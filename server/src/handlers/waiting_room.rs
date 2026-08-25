//! # Waiting Room Handlers (Issue #1187)
//!
//! Bot-resistant virtual queue API that gates checkout during high-traffic
//! ticket releases.
//!
//! ## Endpoints
//!
//! | Method | Path | Purpose |
//! |---|---|---|
//! | `POST` | `/api/v1/waiting-room/challenge` | Issue a single-use SHA-256 proof-of-work challenge |
//! | `POST` | `/api/v1/waiting-room/join` | Verify PoW, enqueue the client, return position / grant |
//! | `GET` | `/api/v1/waiting-room/status` | Current position or grant for a client |
//! | `GET` | `/api/v1/waiting-room/stream` | SSE stream pushing live position + admission grant |
//!
//! ## Admission flow
//!
//! 1. Client asks for a challenge (`challenge` endpoint).
//! 2. Client brute-forces a `nonce` where `SHA-256(challenge || nonce)` starts
//!    with `difficulty` hex zeros (see `services::pow`).
//! 3. Client calls `join` with the solution. The handler verifies the PoW,
//!    derives an admission rate from the venue's remaining inventory and adds
//!    the client to the Redis ZSET queue (see `services::queue`).
//! 4. A background Tokio worker admits clients at the token-bucket rate and
//!    issues a cryptographically signed `checkout_access` grant JWT.
//! 5. The client watches the SSE stream; when the `admitted` event arrives
//!    with the grant token, they may proceed to checkout.

use axum::{
    extract::{Query, State},
    response::{
        sse::{Event, KeepAlive, Sse},
        IntoResponse, Response,
    },
    Json,
};
use futures_util::stream::Stream;
use serde::{Deserialize, Serialize};
use sqlx::PgPool;
use std::{convert::Infallible, sync::Arc, time::Duration};
use uuid::Uuid;

use crate::services::pow::{self, DEFAULT_CHALLENGE_TTL_SECONDS, DEFAULT_DIFFICULTY};
use crate::services::queue::{
    admission_rate_for_inventory, pow_challenge_key, QueueEngine, QueueStatusKind,
    DEFAULT_ADMISSION_RATE_PER_MINUTE, DEFAULT_GRANT_TTL_MINUTES,
};
use crate::utils::error::{ApiError, AppError};
use crate::utils::response::success;

// ---------------------------------------------------------------------------
// State & configuration
// ---------------------------------------------------------------------------

/// Shared state for the waiting-room routes.
#[derive(Clone)]
pub struct WaitingRoomState {
    pub engine: Arc<QueueEngine>,
    pub pool: PgPool,
    pub config: WaitingRoomConfig,
}

/// Tuning knobs for the waiting room, read from the environment once.
#[derive(Clone)]
pub struct WaitingRoomConfig {
    /// Leading hex zeros required by the PoW challenge (default 4 = 16 bits).
    pub pow_difficulty: u32,
    /// Maximum checkout admission rate (clients per minute) — scaled down by
    /// remaining venue inventory at join time.
    pub admission_rate_per_minute: u64,
    /// Lifetime of an issued checkout access grant (minutes).
    pub grant_ttl_minutes: i64,
    /// Lifetime of an issued PoW challenge (seconds).
    pub challenge_ttl_seconds: u64,
    /// Background admission worker tick interval (ms).
    pub tick_interval_ms: u64,
}

impl WaitingRoomConfig {
    pub fn from_env() -> Self {
        let env_u64 = |name: &str, default: u64| -> u64 {
            std::env::var(name)
                .ok()
                .and_then(|v| v.parse().ok())
                .unwrap_or(default)
        };
        Self {
            pow_difficulty: std::env::var("WAITING_ROOM_POW_DIFFICULTY")
                .ok()
                .and_then(|v| v.parse().ok())
                .unwrap_or(DEFAULT_DIFFICULTY),
            admission_rate_per_minute: env_u64(
                "WAITING_ROOM_ADMISSION_RATE_PER_MINUTE",
                DEFAULT_ADMISSION_RATE_PER_MINUTE,
            ),
            grant_ttl_minutes: env_u64(
                "WAITING_ROOM_GRANT_TTL_MINUTES",
                DEFAULT_GRANT_TTL_MINUTES as u64,
            ) as i64,
            challenge_ttl_seconds: env_u64(
                "WAITING_ROOM_CHALLENGE_TTL_SECONDS",
                DEFAULT_CHALLENGE_TTL_SECONDS,
            ),
            tick_interval_ms: env_u64("WAITING_ROOM_TICK_INTERVAL_MS", 1000),
        }
    }
}

// ---------------------------------------------------------------------------
// Request / response types
// ---------------------------------------------------------------------------

#[derive(Debug, Deserialize)]
pub struct ChallengeRequest {
    pub event_id: String,
}

#[derive(Debug, Serialize)]
pub struct ChallengeResponse {
    pub challenge: String,
    pub difficulty: u32,
    /// Seconds until the challenge expires.
    pub expires_in: u64,
}

#[derive(Debug, Deserialize)]
pub struct JoinRequest {
    pub event_id: String,
    /// Wallet address / device id that will be queued.
    pub client_id: String,
    pub challenge: String,
    /// PoW solution such that SHA-256(challenge || nonce) has `difficulty` leading zeros.
    pub nonce: String,
}

#[derive(Debug, Deserialize)]
pub struct StatusQuery {
    pub event_id: String,
    pub client_id: String,
}

// ---------------------------------------------------------------------------
// Handlers
// ---------------------------------------------------------------------------

/// `POST /api/v1/waiting-room/challenge`
///
/// Issues a fresh, single-use SHA-256 proof-of-work challenge bound to the
/// event. The challenge is stored in Redis with a short TTL; `join` consumes
/// it so a solved nonce cannot be replayed.
pub async fn request_challenge(
    State(state): State<WaitingRoomState>,
    Json(payload): Json<ChallengeRequest>,
) -> Response {
    if payload.event_id.trim().is_empty() {
        return ApiError::new(axum::http::StatusCode::BAD_REQUEST, "event_id is required")
            .into_response();
    }

    let challenge = pow::generate_challenge();
    let mut conn = state.engine.redis_connection();

    let stored: Result<(), redis::RedisError> = redis::cmd("SET")
        .arg(pow_challenge_key(&challenge))
        .arg(&payload.event_id)
        .arg("EX")
        .arg(state.config.challenge_ttl_seconds)
        .query_async(&mut conn)
        .await;

    if let Err(e) = stored {
        tracing::warn!(error = %e, "Waiting room: failed to persist PoW challenge");
        return AppError::ExternalServiceError(format!("Redis error: {e}")).into_response();
    }

    success(
        ChallengeResponse {
            challenge,
            difficulty: state.config.pow_difficulty,
            expires_in: state.config.challenge_ttl_seconds,
        },
        "Proof-of-work challenge issued",
    )
    .into_response()
}

/// `POST /api/v1/waiting-room/join`
///
/// Verifies the PoW solution, derives the admission rate from the venue's
/// remaining inventory and enqueues the client. Returns either a `waiting`
/// status with position / estimated wait, or an `admitted` status with the
/// signed checkout access grant when the client was already admitted.
pub async fn join_queue(
    State(state): State<WaitingRoomState>,
    Json(payload): Json<JoinRequest>,
) -> Response {
    if payload.event_id.trim().is_empty()
        || payload.client_id.trim().is_empty()
        || payload.challenge.trim().is_empty()
        || payload.nonce.trim().is_empty()
    {
        return ApiError::new(
            axum::http::StatusCode::BAD_REQUEST,
            "event_id, client_id, challenge, and nonce are all required",
        )
        .into_response();
    }

    if !is_valid_client_id(&payload.client_id) {
        return ApiError::new(
            axum::http::StatusCode::BAD_REQUEST,
            "client_id must be 1-128 characters and contain only letters, digits, '-', '_', '.', ':'",
        )
        .into_response();
    }

    // Consume the challenge (single-use) — it must exist and match the event.
    let mut conn = state.engine.redis_connection();
    let stored_event: Result<Option<String>, redis::RedisError> = redis::cmd("GET")
        .arg(pow_challenge_key(&payload.challenge))
        .query_async(&mut conn)
        .await;

    let challenge_ok = match stored_event {
        Ok(Some(event_id)) if event_id == payload.event_id => {
            let _: Result<i64, redis::RedisError> = redis::cmd("DEL")
                .arg(pow_challenge_key(&payload.challenge))
                .query_async(&mut conn)
                .await;
            true
        }
        _ => false,
    };

    if !challenge_ok {
        return ApiError::new(
            axum::http::StatusCode::UNPROCESSABLE_ENTITY,
            "Invalid, expired, or already-used proof-of-work challenge",
        )
        .into_response();
    }

    if !pow::verify_pow(
        &payload.challenge,
        &payload.nonce,
        state.config.pow_difficulty,
    ) {
        return ApiError::new(
            axum::http::StatusCode::UNPROCESSABLE_ENTITY,
            "Proof-of-work solution is incorrect",
        )
        .into_response();
    }

    // Admission rate is derived from remaining inventory (Issue #1187).
    let rate = match admission_rate_from_db(
        &state.pool,
        &payload.event_id,
        state.config.admission_rate_per_minute,
    )
    .await
    {
        Ok(r) => r,
        Err(e) => return e.into_response(),
    };

    if rate == 0 {
        return AppError::Conflict(
            "This event is sold out; the waiting room is closed".to_string(),
        )
        .into_response();
    }

    match state
        .engine
        .join(&payload.event_id, &payload.client_id, rate)
        .await
    {
        Ok(status) => success(status, "Joined the queue").into_response(),
        Err(e) => e.into_response(),
    }
}

/// `GET /api/v1/waiting-room/status?event_id=&client_id=`
///
/// Returns the client's current position / estimated wait, or the signed
/// grant once admitted. 404 when the client is not in the queue.
pub async fn queue_status(
    State(state): State<WaitingRoomState>,
    Query(query): Query<StatusQuery>,
) -> Response {
    if query.event_id.trim().is_empty() || query.client_id.trim().is_empty() {
        return ApiError::new(
            axum::http::StatusCode::BAD_REQUEST,
            "event_id and client_id query parameters are required",
        )
        .into_response();
    }

    match state.engine.status(&query.event_id, &query.client_id).await {
        Ok(Some(status)) => success(status, "Queue status").into_response(),
        Ok(None) => ApiError::new(
            axum::http::StatusCode::NOT_FOUND,
            "You are not in the queue for this event",
        )
        .into_response(),
        Err(e) => e.into_response(),
    }
}

/// `GET /api/v1/waiting-room/stream?event_id=&client_id=`
///
/// Server-Sent Events stream that pushes a `position` event every 2 seconds
/// ("You are #142 in line") and finally an `admitted` event carrying the
/// cryptographically signed checkout access grant, after which the stream
/// closes so the client can redirect to checkout.
pub async fn queue_stream(
    State(state): State<WaitingRoomState>,
    Query(query): Query<StatusQuery>,
) -> Response {
    if query.event_id.trim().is_empty() || query.client_id.trim().is_empty() {
        return ApiError::new(
            axum::http::StatusCode::BAD_REQUEST,
            "event_id and client_id query parameters are required",
        )
        .into_response();
    }

    let stream = position_stream(state, query.event_id, query.client_id);
    Sse::new(stream)
        .keep_alive(
            KeepAlive::new()
                .interval(Duration::from_secs(15))
                .text(": keepalive"),
        )
        .into_response()
}

/// Build the SSE position stream for a client.
fn position_stream(
    state: WaitingRoomState,
    event_id: String,
    client_id: String,
) -> impl Stream<Item = Result<Event, Infallible>> {
    async_stream::stream! {
        let mut interval = tokio::time::interval(Duration::from_secs(2));
        interval.set_missed_tick_behavior(tokio::time::MissedTickBehavior::Skip);

        loop {
            interval.tick().await;
            match state.engine.status(&event_id, &client_id).await {
                Ok(Some(status)) => {
                    if status.status == QueueStatusKind::Admitted {
                        if let Some(token) = status.grant_token {
                            let payload = serde_json::json!({
                                "type": "admitted",
                                "event_id": event_id,
                                "grant_token": token,
                            });
                            tracing::info!(event_id = %event_id, client_id = %client_id, "Waiting room: client admitted, closing stream");
                            yield Ok(Event::default().data(payload.to_string()));
                            break;
                        }
                    } else {
                        let payload = serde_json::json!({
                            "type": "position",
                            "position": status.position,
                            "queue_size": status.queue_size,
                            "estimated_wait_seconds": status.estimated_wait_seconds,
                        });
                        yield Ok(Event::default().data(payload.to_string()));
                    }
                }
                Ok(None) => {
                    let payload = serde_json::json!({
                        "type": "not-in-queue",
                        "event_id": event_id,
                    });
                    yield Ok(Event::default().data(payload.to_string()));
                    break;
                }
                Err(e) => {
                    let payload = serde_json::json!({
                        "type": "error",
                        "message": e.public_message(),
                    });
                    yield Ok(Event::default().data(payload.to_string()));
                    break;
                }
            }
        }
    }
}

// ---------------------------------------------------------------------------
// Helpers
// ---------------------------------------------------------------------------

/// Admission rate for an event based on its remaining inventory.
async fn admission_rate_from_db(
    pool: &PgPool,
    event_id: &str,
    max_rate_per_minute: u64,
) -> Result<u64, AppError> {
    let event_uuid = Uuid::parse_str(event_id)
        .map_err(|_| AppError::ValidationError("event_id must be a valid UUID".to_string()))?;

    let row = sqlx::query_as::<_, (i64, i64)>(
        "SELECT total_tickets, minted_tickets FROM events WHERE id = $1",
    )
    .bind(event_uuid)
    .fetch_optional(pool)
    .await?;

    match row {
        Some((total, minted)) => Ok(admission_rate_for_inventory(
            total,
            minted,
            max_rate_per_minute,
        )),
        None => Err(AppError::NotFound(format!("Event '{event_id}' not found"))),
    }
}

/// Restrict client ids to safe, Redis-key-friendly values.
fn is_valid_client_id(client_id: &str) -> bool {
    let len = client_id.len();
    (1..=128).contains(&len)
        && client_id
            .chars()
            .all(|c| c.is_ascii_alphanumeric() || matches!(c, '-' | '_' | '.' | ':'))
}

// ---------------------------------------------------------------------------
// Tests
// ---------------------------------------------------------------------------

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_client_id_validation() {
        assert!(is_valid_client_id("GABCDEFGHIJKLMNOPQRSTUVWXYZ1234567890"));
        assert!(is_valid_client_id("550e8400-e29b-41d4-a716-446655440000"));
        assert!(is_valid_client_id("a"));
        assert!(is_valid_client_id("device_42:ios"));
        assert!(!is_valid_client_id(""));
        assert!(!is_valid_client_id("has space"));
        assert!(!is_valid_client_id("bad/char"));
        assert!(!is_valid_client_id(&"x".repeat(129)));
    }

    #[test]
    fn test_waiting_room_config_defaults() {
        // Ensure no env vars leak into the defaults test.
        for name in [
            "WAITING_ROOM_POW_DIFFICULTY",
            "WAITING_ROOM_ADMISSION_RATE_PER_MINUTE",
            "WAITING_ROOM_GRANT_TTL_MINUTES",
            "WAITING_ROOM_CHALLENGE_TTL_SECONDS",
            "WAITING_ROOM_TICK_INTERVAL_MS",
        ] {
            std::env::remove_var(name);
        }
        let config = WaitingRoomConfig::from_env();
        assert_eq!(config.pow_difficulty, DEFAULT_DIFFICULTY);
        assert_eq!(
            config.admission_rate_per_minute,
            DEFAULT_ADMISSION_RATE_PER_MINUTE
        );
        assert_eq!(config.grant_ttl_minutes, DEFAULT_GRANT_TTL_MINUTES);
        assert_eq!(config.challenge_ttl_seconds, DEFAULT_CHALLENGE_TTL_SECONDS);
        assert_eq!(config.tick_interval_ms, 1000);
    }
}
