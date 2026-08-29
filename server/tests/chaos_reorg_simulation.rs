//! # Simulated Stellar Chain Re-org Runner (Issue #1178)
//!
//! Drives the real re-org machinery in `agora_server::services::indexer`
//! (`CheckpointStore::rollback_to`, `buffer_insert`, `apply_event_state`) —
//! the same functions the live Soroban event indexer uses — through a
//! simulated chain fork, without touching the network. This is the "Stellar
//! ledger re-org rollback" runner called out in the issue: it rolls back a
//! simulated chain by 5 ledgers and verifies the database recovers without
//! leaking invalid state.
//!
//! ## Running
//! ```bash
//! DATABASE_URL=postgres://user:password@localhost:5432/agora cargo test --test chaos_reorg_simulation
//! ```
//! Skips gracefully (matching `server/tests/auth_integration.rs`) when
//! `DATABASE_URL` is unset.
//!
//! ## Safety
//! Every row this suite writes is scoped to a synthetic, randomized ledger
//! range far above any real Stellar ledger height (see
//! [`common::random_synthetic_ledger_base`]) and a `chaos-test:` prefixed
//! `chain_key`, so it is safe to run against a shared, populated dev
//! database. On an assertion failure mid-test some `chaos-test:` rows may be
//! left behind; they are inert and safe to delete manually
//! (`DELETE FROM blockchain_checkpoints WHERE chain_key LIKE 'chaos-test:%'`).
//!
//! ## A discovered limitation (see `deep_reorg_beyond_confirmation_window_*`)
//! `CheckpointStore::rollback_to` rewinds the cursor and purges the
//! *unfinalized buffer* atomically, but it never reverses state already
//! written to `tickets` / `events` by `apply_event_state`. That's safe as
//! long as a re-org can never reach deeper than `INDEXER_CONFIRMATIONS`
//! ledgers (the whole point of the confirmation window) — but the issue
//! explicitly asks this suite to simulate rolling back 5 ledgers, which is
//! deeper than the default confirmation depth of 2. One test below pins
//! down what currently happens in that case as an honest regression guard,
//! not as a claim that the behavior is desirable.

mod common;

use agora_server::models::blockchain_checkpoint::CheckpointStore;
use agora_server::models::indexer_event::IndexedEvent;
use agora_server::services::indexer::{apply_event_state, buffer_insert, IndexerConfig};
use common::{cleanup_organizer, random_synthetic_ledger_base, seed_organizer_and_event, test_pool};
use serde_json::json;
use sqlx::PgPool;
use uuid::Uuid;

const PAYMENT_PROCESSED_TOPIC: &str = "PaymentProcessed";

fn test_indexer_config(ticket_payment_contract_id: &str) -> IndexerConfig {
    IndexerConfig {
        rpc_urls: vec!["https://rpc.invalid".to_string()],
        ticket_payment_contract_id: ticket_payment_contract_id.to_string(),
        event_registry_contract_id: "CREG-UNUSED".to_string(),
        start_ledger: 0,
        window_ledgers: 100,
        confirmations: 2,
        workers: 1,
        redis_url: None,
    }
}

/// Build, buffer-insert, and (optionally) finalize+apply a synthetic
/// `PaymentProcessed` event at `ledger`, minting ticket `stellar_id` owned by
/// `owner_wallet` against `event_id` when `apply` is true.
async fn emit_purchase(
    pool: &PgPool,
    config: &IndexerConfig,
    ledger: u32,
    latest_ledger: u32,
    event_id: Uuid,
    stellar_id: &str,
    owner_wallet: &str,
    apply: bool,
) {
    let value = json!({
        "payment_id": stellar_id,
        "event_id": event_id.to_string(),
        "buyer": owner_wallet,
        "owner": owner_wallet,
        "amount": 25_000_000i64,
    });

    let event = IndexedEvent::decode(
        format!("rpc-id-{stellar_id}"),
        ledger,
        config.ticket_payment_contract_id.clone(),
        &[PAYMENT_PROCESSED_TOPIC.to_string()],
        &value,
        latest_ledger,
    );

    let inserted = buffer_insert(pool, &event).await.expect("buffer_insert");
    assert!(inserted > 0, "buffer_insert must not report a duplicate for a fresh id");

    if apply {
        apply_event_state(pool, config, None, &event)
            .await
            .expect("apply_event_state");
        sqlx::query("UPDATE indexer_ledger_events SET finalized = TRUE WHERE id = $1")
            .bind(&event.id)
            .execute(pool)
            .await
            .expect("mark finalized");
    }
}

async fn ticket_owner(pool: &PgPool, stellar_id: &str) -> Option<(String, String)> {
    sqlx::query_as::<_, (String, String)>(
        "SELECT owner_wallet, status FROM tickets WHERE stellar_id = $1",
    )
    .bind(stellar_id)
    .fetch_optional(pool)
    .await
    .expect("query ticket")
}

async fn buffered_ledgers_at_or_above(pool: &PgPool, contract_id: &str, ledger: i64) -> i64 {
    sqlx::query_scalar::<_, i64>(
        "SELECT COUNT(*) FROM indexer_ledger_events WHERE contract_id = $1 AND ledger >= $2",
    )
    .bind(contract_id)
    .bind(ledger)
    .fetch_one(pool)
    .await
    .expect("count buffered rows")
}

// ---------------------------------------------------------------------------
// 1. The safe case: reorg confined to the unconfirmed window
// ---------------------------------------------------------------------------

/// Rolls back 5 ledgers' worth of *unconfirmed* (buffered but never applied)
/// events — the case the indexer's confirmation window exists to make safe.
/// Finalized/applied state below the fork must survive untouched, and every
/// buffered row at or after the fork must be purged.
#[tokio::test]
async fn five_ledger_reorg_within_unconfirmed_window_recovers_cleanly() {
    let Some(pool) = test_pool("five_ledger_reorg_within_unconfirmed_window_recovers_cleanly").await else {
        return;
    };

    let contract_id = format!("CPAY-{}", Uuid::new_v4());
    let config = test_indexer_config(&contract_id);
    let chain_key = format!("chaos-test:reorg:{}", Uuid::new_v4());
    let base = random_synthetic_ledger_base();
    let (organizer_id, event_id) =
        seed_organizer_and_event(&pool, "five-ledger-safe-reorg").await;

    // Ledgers [base, base+4]: finalized and applied — these must survive.
    for i in 0..5u32 {
        let stellar_id = format!("chaos-safe-pre-{base}-{i}");
        emit_purchase(
            &pool,
            &config,
            base + i,
            base + 9,
            event_id,
            &stellar_id,
            "GPRE0000000000000000000000000000000000000",
            true,
        )
        .await;
    }
    CheckpointStore::save(&pool, &chain_key, (base + 4) as i64, None)
        .await
        .expect("save checkpoint after finalizing pre-fork ledgers");

    // Ledgers [base+5, base+9]: buffered but NOT YET finalized/applied —
    // exactly the window a re-org is allowed to invalidate.
    for i in 5..10u32 {
        let stellar_id = format!("chaos-safe-post-{base}-{i}");
        emit_purchase(
            &pool,
            &config,
            base + i,
            base + 9,
            event_id,
            &stellar_id,
            "GPOST000000000000000000000000000000000000",
            false,
        )
        .await;
    }

    // Fork detected at base+5 → "rolling back 5 ledgers" (base+5..base+9).
    let fork_ledger = (base + 5) as i64;
    let new_checkpoint = CheckpointStore::rollback_to(&pool, &chain_key, fork_ledger)
        .await
        .expect("rollback_to");
    assert_eq!(new_checkpoint, (base + 4) as i64);

    let checkpoint = CheckpointStore::load(&pool, &chain_key)
        .await
        .expect("load checkpoint")
        .expect("checkpoint row must exist");
    assert_eq!(checkpoint.ledger_sequence, (base + 4) as i64);
    assert!(checkpoint.event_cursor.is_none(), "rollback clears the opaque cursor");

    assert_eq!(
        buffered_ledgers_at_or_above(&pool, &contract_id, fork_ledger).await,
        0,
        "every buffered row at or after the fork ledger must be purged"
    );

    for i in 0..5u32 {
        let stellar_id = format!("chaos-safe-pre-{base}-{i}");
        let ticket = ticket_owner(&pool, &stellar_id).await;
        assert!(
            ticket.is_some(),
            "finalized ticket for ledger {} must survive the reorg (no invalid state leak below the fork)",
            base + i
        );
    }
    for i in 5..10u32 {
        let stellar_id = format!("chaos-safe-post-{base}-{i}");
        assert!(
            ticket_owner(&pool, &stellar_id).await.is_none(),
            "unconfirmed ticket for ledger {} must never have been applied",
            base + i
        );
    }

    cleanup_organizer(&pool, organizer_id).await;
    let _ = sqlx::query("DELETE FROM blockchain_checkpoints WHERE chain_key = $1")
        .bind(&chain_key)
        .execute(&pool)
        .await;
}

// ---------------------------------------------------------------------------
// 2. Atomicity of the checkpoint + buffer rewind
// ---------------------------------------------------------------------------

#[tokio::test]
async fn rollback_moves_checkpoint_and_purges_buffer_atomically() {
    let Some(pool) = test_pool("rollback_moves_checkpoint_and_purges_buffer_atomically").await else {
        return;
    };

    let contract_id = format!("CPAY-{}", Uuid::new_v4());
    let config = test_indexer_config(&contract_id);
    let chain_key = format!("chaos-test:reorg:{}", Uuid::new_v4());
    let base = random_synthetic_ledger_base();
    let (organizer_id, event_id) =
        seed_organizer_and_event(&pool, "atomic-rollback").await;

    for i in 0..3u32 {
        emit_purchase(
            &pool,
            &config,
            base + i,
            base + 3,
            event_id,
            &format!("chaos-atomic-{base}-{i}"),
            "GATOMIC0000000000000000000000000000000000",
            false,
        )
        .await;
    }
    CheckpointStore::save(&pool, &chain_key, (base + 3) as i64, Some("some-opaque-cursor"))
        .await
        .expect("save checkpoint");

    let fork_ledger = base as i64;
    CheckpointStore::rollback_to(&pool, &chain_key, fork_ledger)
        .await
        .expect("rollback_to");

    // Both halves of the rewind must be observable together: nothing left
    // buffered at/after the fork, and the checkpoint reflects exactly one
    // ledger before it — never a state where one moved but not the other.
    let checkpoint = CheckpointStore::load(&pool, &chain_key)
        .await
        .unwrap()
        .unwrap();
    assert_eq!(checkpoint.ledger_sequence, fork_ledger - 1);
    assert_eq!(
        buffered_ledgers_at_or_above(&pool, &contract_id, fork_ledger).await,
        0
    );

    cleanup_organizer(&pool, organizer_id).await;
    let _ = sqlx::query("DELETE FROM blockchain_checkpoints WHERE chain_key = $1")
        .bind(&chain_key)
        .execute(&pool)
        .await;
}

// ---------------------------------------------------------------------------
// 3. The surviving fork replays idempotently after rollback
// ---------------------------------------------------------------------------

#[tokio::test]
async fn surviving_fork_replays_idempotently_after_rollback() {
    let Some(pool) = test_pool("surviving_fork_replays_idempotently_after_rollback").await else {
        return;
    };

    let contract_id = format!("CPAY-{}", Uuid::new_v4());
    let config = test_indexer_config(&contract_id);
    let chain_key = format!("chaos-test:reorg:{}", Uuid::new_v4());
    let base = random_synthetic_ledger_base();
    let (organizer_id, event_id) = seed_organizer_and_event(&pool, "fork-replay").await;
    let stellar_id = format!("chaos-fork-{base}");

    // Original (soon-to-be-orphaned) chain: buffered, never finalized.
    emit_purchase(
        &pool,
        &config,
        base,
        base,
        event_id,
        &stellar_id,
        "GORIGINAL000000000000000000000000000000000",
        false,
    )
    .await;

    CheckpointStore::rollback_to(&pool, &chain_key, base as i64)
        .await
        .expect("rollback_to");

    // The surviving fork re-emits a purchase at the same ledger height for
    // the same ticket, with a new RPC envelope id (as a real re-scan would)
    // but referencing the same on-chain payment — apply must be idempotent.
    for _ in 0..2 {
        emit_purchase(
            &pool,
            &config,
            base,
            base,
            event_id,
            &stellar_id,
            "GFORKOWNER00000000000000000000000000000000",
            true,
        )
        .await;
    }

    let rows = sqlx::query_scalar::<_, i64>("SELECT COUNT(*) FROM tickets WHERE stellar_id = $1")
        .bind(&stellar_id)
        .fetch_one(&pool)
        .await
        .unwrap();
    assert_eq!(
        rows, 1,
        "replaying the fork's purchase twice must not create duplicate tickets"
    );

    let (owner, _status) = ticket_owner(&pool, &stellar_id).await.expect("ticket exists");
    assert_eq!(owner, "GFORKOWNER00000000000000000000000000000000");

    cleanup_organizer(&pool, organizer_id).await;
    let _ = sqlx::query("DELETE FROM blockchain_checkpoints WHERE chain_key = $1")
        .bind(&chain_key)
        .execute(&pool)
        .await;
}

// ---------------------------------------------------------------------------
// 4. Rollback is safe to retry
// ---------------------------------------------------------------------------

#[tokio::test]
async fn rollback_to_same_fork_ledger_twice_is_idempotent() {
    let Some(pool) = test_pool("rollback_to_same_fork_ledger_twice_is_idempotent").await else {
        return;
    };

    let chain_key = format!("chaos-test:reorg:{}", Uuid::new_v4());
    let base = random_synthetic_ledger_base();
    CheckpointStore::save(&pool, &chain_key, base as i64, None)
        .await
        .unwrap();

    let first = CheckpointStore::rollback_to(&pool, &chain_key, base as i64)
        .await
        .expect("first rollback");
    let second = CheckpointStore::rollback_to(&pool, &chain_key, base as i64)
        .await
        .expect("retried rollback must not error");
    assert_eq!(first, second, "retrying the same rollback must be a no-op, not a further rewind");

    let _ = sqlx::query("DELETE FROM blockchain_checkpoints WHERE chain_key = $1")
        .bind(&chain_key)
        .execute(&pool)
        .await;
}

// ---------------------------------------------------------------------------
// 5. Discovered limitation: a reorg deeper than the confirmation window
// ---------------------------------------------------------------------------

/// `INDEXER_CONFIRMATIONS` defaults to 2 ledgers — the assumption baked into
/// `services::indexer` is that a re-org can never reach further back than
/// that, so once an event is "finalized" it's permanently safe to have
/// applied its state. Issue #1178 explicitly asks this suite to simulate
/// rolling back **5** ledgers, which exceeds that assumption.
///
/// This test does not assert the system "recovers correctly" — it pins down
/// what actually happens today: the checkpoint and buffer rewind correctly,
/// but the `tickets` rows written for the now-invalidated ledgers are left
/// behind, because `rollback_to` never reverses `apply_event_state`'s
/// writes. That is a real gap between the confirmation-window assumption
/// and the depth of re-org this suite is asked to simulate — flagged here
/// as a regression guard / follow-up marker rather than silently patched,
/// since fixing it is a production-code decision outside this test suite's
/// scope.
#[tokio::test]
async fn deep_reorg_beyond_confirmation_window_can_leave_applied_state_stale() {
    let Some(pool) =
        test_pool("deep_reorg_beyond_confirmation_window_can_leave_applied_state_stale").await
    else {
        return;
    };

    let contract_id = format!("CPAY-{}", Uuid::new_v4());
    let config = test_indexer_config(&contract_id); // confirmations: 2
    let chain_key = format!("chaos-test:reorg:{}", Uuid::new_v4());
    let base = random_synthetic_ledger_base();
    let (organizer_id, event_id) =
        seed_organizer_and_event(&pool, "deep-reorg-limitation").await;

    // Finalize + apply ledgers [base, base+4] — 5 ledgers deep, i.e. well
    // past the 2-ledger confirmation margin, exactly as the issue's "roll
    // back 5 ledgers" scenario implies.
    for i in 0..5u32 {
        emit_purchase(
            &pool,
            &config,
            base + i,
            base + 4,
            event_id,
            &format!("chaos-deep-{base}-{i}"),
            "GDEEP00000000000000000000000000000000000000",
            true,
        )
        .await;
    }
    CheckpointStore::save(&pool, &chain_key, (base + 4) as i64, None)
        .await
        .unwrap();

    // A fork is now discovered reaching back to base+2 — deeper than what
    // the confirmation window assumed could ever be invalidated.
    let fork_ledger = (base + 2) as i64;
    CheckpointStore::rollback_to(&pool, &chain_key, fork_ledger)
        .await
        .expect("rollback_to");

    let checkpoint = CheckpointStore::load(&pool, &chain_key).await.unwrap().unwrap();
    assert_eq!(
        checkpoint.ledger_sequence,
        fork_ledger - 1,
        "the cursor itself rewinds correctly regardless of confirmation depth"
    );
    assert_eq!(
        buffered_ledgers_at_or_above(&pool, &contract_id, fork_ledger).await,
        0,
        "the buffer is purged correctly regardless of confirmation depth"
    );

    // The gap: tickets for the invalidated ledgers [base+2, base+4] are
    // still present, because rollback_to never touches `tickets`.
    let mut stale_survivors = 0;
    for i in 2..5u32 {
        let stellar_id = format!("chaos-deep-{base}-{i}");
        if ticket_owner(&pool, &stellar_id).await.is_some() {
            stale_survivors += 1;
        }
    }
    assert_eq!(
        stale_survivors, 3,
        "documents the current gap: applied tickets for rolled-back ledgers are not \
         retroactively invalidated by rollback_to. If this assertion ever fails because \
         it dropped to 0, `services::indexer` gained reversal logic — update this test \
         to assert the (now fixed) safe behavior instead of removing the coverage."
    );

    cleanup_organizer(&pool, organizer_id).await;
    let _ = sqlx::query("DELETE FROM blockchain_checkpoints WHERE chain_key = $1")
        .bind(&chain_key)
        .execute(&pool)
        .await;
}
