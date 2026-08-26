//! # Admin Indexer Endpoints (Issue #1174)
//!
//! Administrative control surface for the Soroban indexer engine:
//!
//! * `GET /api/v1/admin/indexer/replay?start_ledger=X&end_ledger=Y` — backfills
//!   historical contract state for the ledger range `[X, Y]` in a background
//!   task and returns `202 Accepted`. Replay goes through the same buffered,
//!   idempotent pipeline as live indexing, so duplicate runs are safe.
//!
//! All routes are protected by the existing admin bearer-token + audit
//! middleware attached to `/api/v1/admin`.

use axum::{
    extract::{Query, State},
    http::StatusCode,
    response::{IntoResponse, Response},
    Json,
};
use serde::{Deserialize, Serialize};
use sqlx::PgPool;
use uuid::Uuid;

use crate::handlers::ws::PurchaseBroadcaster;
use crate::services::indexer::{run_replay, IndexerConfig, MAX_REPLAY_LEDGERS};
use crate::utils::error::ApiError;

/// Application state for the admin indexer routes.
#[derive(Clone)]
pub struct IndexerAdminState {
    pub pool: PgPool,
    /// Optional WebSocket broadcaster so replayed purchases also reach the
    /// live dashboard stream.
    pub broker: Option<PurchaseBroadcaster>,
}

/// Query parameters for `GET /api/v1/admin/indexer/replay`.
#[derive(Debug, Deserialize)]
pub struct ReplayQuery {
    /// First ledger to backfill (inclusive, must be `>= 1`).
    pub start_ledger: u32,
    /// Last ledger to backfill (inclusive, must be `>= start_ledger`).
    pub end_ledger: u32,
}

/// The `202 Accepted` body returned by the replay endpoint.
#[derive(Debug, Serialize)]
pub struct ReplayAccepted {
    pub task_id: String,
    pub start_ledger: u32,
    pub end_ledger: u32,
}

/// Kick off a background backfill of `[start_ledger, end_ledger]`.
///
/// The run is spawn-and-forget in the same style as the rest of the
/// background work in this codebase; progress is observable through structured
/// logs and the persisted checkpoint.
pub async fn replay_indexer(
    State(state): State<IndexerAdminState>,
    Query(query): Query<ReplayQuery>,
) -> Result<Response, ApiError> {
    validate_replay_range(query.start_ledger, query.end_ledger)?;

    let config = IndexerConfig::from_env();
    if config.rpc_urls.is_empty() || config.contract_ids().is_empty() {
        return Err(ApiError::new(
            StatusCode::BAD_GATEWAY,
            "Indexer is not configured (missing SOROBAN_RPC_URL(S) / contract IDs)",
        ));
    }

    let pool = state.pool.clone();
    let broker = state.broker.clone();

    tokio::spawn(async move {
        if let Err(e) = run_replay(
            &pool,
            &config,
            broker.as_ref(),
            query.start_ledger,
            query.end_ledger,
        )
        .await
        {
            tracing::error!(
                error = ?e,
                start_ledger = query.start_ledger,
                end_ledger = query.end_ledger,
                "Replay task failed"
            );
        }
    });

    let payload = ReplayAccepted {
        task_id: Uuid::new_v4().to_string(),
        start_ledger: query.start_ledger,
        end_ledger: query.end_ledger,
    };

    Ok((StatusCode::ACCEPTED, Json(payload)).into_response())
}

/// Validate a replay range before dispatching the background task.
fn validate_replay_range(start_ledger: u32, end_ledger: u32) -> Result<(), ApiError> {
    if start_ledger == 0 {
        return Err(ApiError::new(
            StatusCode::BAD_REQUEST,
            "start_ledger must be >= 1",
        ));
    }
    if end_ledger < start_ledger {
        return Err(ApiError::new(
            StatusCode::BAD_REQUEST,
            format!(
                "end_ledger ({end_ledger}) must be >= start_ledger ({start_ledger})"
            ),
        ));
    }
    if end_ledger.saturating_sub(start_ledger) > MAX_REPLAY_LEDGERS {
        return Err(ApiError::new(
            StatusCode::BAD_REQUEST,
            format!("replay range exceeds the {MAX_REPLAY_LEDGERS} ledger limit"),
        ));
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn rejects_zero_start_ledger() {
        let err = validate_replay_range(0, 10).unwrap_err();
        assert_eq!(err.code, StatusCode::BAD_REQUEST.as_u16());
    }

    #[test]
    fn rejects_inverted_range() {
        let err = validate_replay_range(20, 10).unwrap_err();
        assert_eq!(err.code, StatusCode::BAD_REQUEST.as_u16());
        assert!(err.message.contains("end_ledger"));
    }

    #[test]
    fn rejects_huge_ranges() {
        let err = validate_replay_range(1, MAX_REPLAY_LEDGERS + 100).unwrap_err();
        assert_eq!(err.code, StatusCode::BAD_REQUEST.as_u16());
        assert!(err.message.contains("limit"));
    }

    #[test]
    fn accepts_valid_ranges() {
        assert!(validate_replay_range(1, 10).is_ok());
        assert!(validate_replay_range(1, MAX_REPLAY_LEDGERS + 1).is_ok());
    }
}