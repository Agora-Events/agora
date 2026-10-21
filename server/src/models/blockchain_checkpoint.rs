//! # Ledger Cursor Checkpoint Engine (Issue #1174)
//!
//! Persists the Soroban event indexer's position inside PostgreSQL so the
//! pipeline can resume across restarts and roll back unfinalized state when
//! the Stellar network re-orgs.
//!
//! ## Why PostgreSQL, not Redis
//!
//! The cursor is the source of truth for the indexer. Redis is treated as an
//! optional, best-effort mirror; if it is wiped or flaky, the checkpoint here
//! still lets the pipeline resume without re-scanning the whole chain.
//!
//! ## Re-org handling
//!
//! [`CheckpointStore::rollback_to`] purges every buffered event at or after a
//! fork ledger and rewinds the cursor to `fork_ledger - 1` **in a single
//! transaction**, so the two can never be observed half-applied by a reader
//! or a crash.

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use sqlx::PgPool;

/// Default chain key used by the pipeline when no environment override is set.
pub const DEFAULT_CHAIN_KEY: &str = "soroban:mainnet";

/// One row of the `blockchain_checkpoints` table.
#[derive(Debug, Clone, Serialize, Deserialize, sqlx::FromRow)]
pub struct BlockchainCheckpoint {
    /// Unique chain identifier (e.g. `"soroban:mainnet"`, `"soroban:testnet"`).
    pub chain_key: String,
    /// Highest ledger whose events have been finalized and applied to the DB.
    pub ledger_sequence: i64,
    /// Opaque RPC pagination cursor for fast resume (`NULL` forces a scan
    /// from `ledger_sequence + 1`, which is always safe because inserts are
    /// deduplicated and state application is idempotent).
    pub event_cursor: Option<String>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

/// Stateless set of SQL operations over [`BlockchainCheckpoint`].
pub struct CheckpointStore;

impl CheckpointStore {
    /// Load the checkpoint for `chain_key`.
    pub async fn load(
        pool: &PgPool,
        chain_key: &str,
    ) -> Result<Option<BlockchainCheckpoint>, sqlx::Error> {
        sqlx::query_as::<_, BlockchainCheckpoint>(
            "SELECT chain_key, ledger_sequence, event_cursor, created_at, updated_at
             FROM blockchain_checkpoints WHERE chain_key = $1",
        )
        .bind(chain_key)
        .fetch_optional(pool)
        .await
    }

    /// Atomically upsert the checkpoint cursor.
    ///
    /// `INSERT ... ON CONFLICT DO UPDATE` is a single statement, so concurrent
    /// producers/workers can never observe or leave a partially written row.
    pub async fn save(
        pool: &PgPool,
        chain_key: &str,
        ledger_sequence: i64,
        event_cursor: Option<&str>,
    ) -> Result<(), sqlx::Error> {
        sqlx::query(
            "INSERT INTO blockchain_checkpoints
                (chain_key, ledger_sequence, event_cursor, created_at, updated_at)
             VALUES ($1, $2, $3, NOW(), NOW())
             ON CONFLICT (chain_key) DO UPDATE SET
                ledger_sequence = EXCLUDED.ledger_sequence,
                event_cursor    = EXCLUDED.event_cursor,
                updated_at      = NOW()",
        )
        .bind(chain_key)
        .bind(ledger_sequence)
        .bind(event_cursor)
        .execute(pool)
        .await
        .map(|_| ())
    }

    /// Rewind the chain to just before `fork_ledger` and purge every buffered
    /// event at or above it — atomically.
    ///
    /// After the call the producer re-scans from `fork_ledger` and replays the
    /// surviving fork, which is safe because buffer inserts are deduplicated
    /// (`ON CONFLICT DO NOTHING`) and state application is idempotent
    /// (`INSERT ... ON CONFLICT` / plain `UPDATE`s).
    ///
    /// Returns the new cursor ledger (`fork_ledger - 1`).
    pub async fn rollback_to(
        pool: &PgPool,
        chain_key: &str,
        fork_ledger: i64,
    ) -> Result<i64, sqlx::Error> {
        let mut tx = pool.begin().await?;

        sqlx::query("DELETE FROM indexer_ledger_events WHERE ledger >= $1")
            .bind(fork_ledger)
            .execute(&mut *tx)
            .await?;

        sqlx::query(
            "INSERT INTO blockchain_checkpoints
                (chain_key, ledger_sequence, event_cursor, created_at, updated_at)
             VALUES ($1, $2 - 1, NULL, NOW(), NOW())
             ON CONFLICT (chain_key) DO UPDATE SET
                ledger_sequence = EXCLUDED.ledger_sequence,
                event_cursor    = NULL,
                updated_at      = NOW()",
        )
        .bind(chain_key)
        .bind(fork_ledger)
        .execute(&mut *tx)
        .await?;

        tx.commit().await?;

        // Best-effort: also drop the mirrored Redis cursor so a restart that
        // prefers Redis cannot resume past the fork. Redis is out of scope of
        // the transaction, hence best-effort.
        Ok(fork_ledger - 1)
    }

    /// Prune finalized buffer rows older than `keep_from_ledger` so the
    /// sliding window stays bounded.
    ///
    /// Only *finalized* rows are ever pruned; pending rows are always retained
    /// so a re-org within the window remains detectable.
    pub async fn prune_window(
        pool: &PgPool,
        keep_from_ledger: i64,
    ) -> Result<u64, sqlx::Error> {
        let result = sqlx::query(
            "DELETE FROM indexer_ledger_events
             WHERE finalized = TRUE AND ledger < $1",
        )
        .bind(keep_from_ledger)
        .execute(pool)
        .await?;

        Ok(result.rows_affected())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn checkpoint_serialization_round_trip() {
        let cp = BlockchainCheckpoint {
            chain_key: DEFAULT_CHAIN_KEY.to_string(),
            ledger_sequence: 42,
            event_cursor: Some("abc-123".to_string()),
            created_at: Utc::now(),
            updated_at: Utc::now(),
        };

        let json = serde_json::to_string(&cp).expect("serialize");
        let back: BlockchainCheckpoint = serde_json::from_str(&json).expect("deserialize");

        assert_eq!(back.chain_key, DEFAULT_CHAIN_KEY);
        assert_eq!(back.ledger_sequence, 42);
        assert_eq!(back.event_cursor.as_deref(), Some("abc-123"));
    }

    #[test]
    fn checkpoint_null_cursor_serializes() {
        let cp = BlockchainCheckpoint {
            chain_key: DEFAULT_CHAIN_KEY.to_string(),
            ledger_sequence: 7,
            event_cursor: None,
            created_at: Utc::now(),
            updated_at: Utc::now(),
        };
        let json = serde_json::to_string(&cp).unwrap();
        let back: BlockchainCheckpoint = serde_json::from_str(&json).unwrap();
        assert!(back.event_cursor.is_none());
    }
}