//! Shared test-support code for the chaos engineering & re-org simulation
//! suite (Issue #1178). Lives under `tests/common/mod.rs` (not
//! `tests/common.rs`) specifically so Cargo does not treat it as its own
//! test binary — see the [integration test guidelines] in the Rust book.
//!
//! [integration test guidelines]: https://doc.rust-lang.org/book/ch11-03-test-organization.html#submodules-in-integration-tests

pub mod chaos;

use sqlx::PgPool;

/// Connect to the database named by `DATABASE_URL`, or return `None` with a
/// message on stderr.
///
/// Mirrors the skip-gracefully convention already used by
/// `server/tests/auth_integration.rs` so these tests behave identically in
/// environments (local dev without Docker, some CI runners) that don't wire
/// up Postgres.
pub async fn test_pool(test_name: &str) -> Option<PgPool> {
    let Ok(database_url) = std::env::var("DATABASE_URL") else {
        eprintln!("skipping {test_name}: DATABASE_URL not set");
        return None;
    };

    match PgPool::connect(&database_url).await {
        Ok(pool) => Some(pool),
        Err(e) => {
            eprintln!("skipping {test_name}: failed to connect to DATABASE_URL: {e}");
            None
        }
    }
}

/// Ledger base far above any plausible real Stellar ledger sequence (mainnet
/// sits in the tens of millions as of 2026 — Stellar closes a ledger roughly
/// every 5 seconds and has run since 2015). Chaos/re-org tests that write
/// into `indexer_ledger_events` anchor every synthetic ledger number at or
/// above this constant so `CheckpointStore::rollback_to`'s
/// `DELETE FROM indexer_ledger_events WHERE ledger >= $1` — which is not
/// scoped by chain — can never delete real indexed data if these tests run
/// against a shared, populated database.
pub const SYNTHETIC_LEDGER_BASE: u32 = 3_900_000_000;

/// A synthetic ledger base randomized within `[SYNTHETIC_LEDGER_BASE,
/// SYNTHETIC_LEDGER_BASE + 300_000_000)`.
///
/// `CheckpointStore::rollback_to`'s buffer purge (`DELETE FROM
/// indexer_ledger_events WHERE ledger >= $1`) is not scoped by chain or
/// contract, and `cargo test` runs multiple `#[tokio::test]` functions —
/// even across different integration-test binaries — concurrently against
/// the same database. Randomizing each test's ledger slice on top of the
/// already-far-from-real `SYNTHETIC_LEDGER_BASE` keeps concurrently running
/// chaos/re-org tests from purging each other's buffered rows, while still
/// leaving comfortable headroom below `u32::MAX` (~4.29B) for a small test
/// window.
pub fn random_synthetic_ledger_base() -> u32 {
    use rand::Rng;
    SYNTHETIC_LEDGER_BASE + rand::thread_rng().gen_range(0..300_000_000)
}

/// Create a synthetic organizer + event pair for tests that need a valid FK
/// target for `tickets.event_id`. Returns `(organizer_id, event_id)`.
///
/// Deleting the organizer row cascades to the event and every ticket
/// attached to it (`events.organizer_id` and `tickets.event_id` are both
/// `ON DELETE CASCADE`), so callers only need to clean up the organizer.
pub async fn seed_organizer_and_event(pool: &PgPool, label: &str) -> (uuid::Uuid, uuid::Uuid) {
    let organizer_id = uuid::Uuid::new_v4();
    let event_id = uuid::Uuid::new_v4();

    sqlx::query(
        "INSERT INTO organizers (id, name, contact_email) VALUES ($1, $2, 'chaos-suite@agora.test')",
    )
    .bind(organizer_id)
    .bind(format!("Chaos Suite Organizer ({label})"))
    .execute(pool)
    .await
    .expect("seed organizer");

    sqlx::query(
        "INSERT INTO events (id, organizer_id, title, location, start_time)
         VALUES ($1, $2, $3, 'Chaos Suite Venue', NOW())",
    )
    .bind(event_id)
    .bind(organizer_id)
    .bind(format!("Chaos Suite Event ({label})"))
    .execute(pool)
    .await
    .expect("seed event");

    (organizer_id, event_id)
}

/// Delete an organizer created by [`seed_organizer_and_event`] (cascades to
/// its event and tickets).
pub async fn cleanup_organizer(pool: &PgPool, organizer_id: uuid::Uuid) {
    let _ = sqlx::query("DELETE FROM organizers WHERE id = $1")
        .bind(organizer_id)
        .execute(pool)
        .await;
}
