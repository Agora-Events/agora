//! # Redis-backed Virtual Waiting Room Queue Engine (Issue #1187)
//!
//! A fair, bot-resistant virtual queue that protects checkout from
//! connection-pool exhaustion during high-traffic ticket releases.
//!
//! ## Design
//!
//! - **Fair FIFO queue** — one Redis sorted set per event
//!   (`agora:waiting:{event_id}:zset`). Members are scored with an atomic
//!   per-event sequence counter, so admission order is exactly arrival order
//!   regardless of how many workers or server replicas are running.
//! - **Token-bucket admission** — each event has a Redis token bucket whose
//!   refill rate is derived from the venue inventory at join time
//!   (`rate_per_minute = min(remaining_inventory, configured_max)`). A Lua
//!   script atomically refills the bucket and pops the queue head, so
//!   admissions per minute can never exceed the configured rate even under
//!   concurrency.
//! - **Signed access grants** — admitted clients receive a short-lived JWT
//!   (`purpose = "checkout_access"`) stored in Redis with a TTL. Handlers and
//!   future checkout endpoints can verify it with [`verify_grant_token`].
//! - **Background admission worker** — a Tokio interval task drains every
//!   active event queue at the bucket rate. Clients observe their admission
//!   through the SSE position stream (`handlers::waiting_room::queue_stream`).

use crate::cache::RedisCache;
use crate::handlers::auth::jwt_secret;
use crate::utils::error::AppError;
use chrono::{Duration, Utc};
use jsonwebtoken::{decode, encode, Algorithm, DecodingKey, EncodingKey, Header, Validation};
use serde::{Deserialize, Serialize};
use std::time::Duration as StdDuration;
use uuid::Uuid;

// ---------------------------------------------------------------------------
// Redis key helpers
// ---------------------------------------------------------------------------

/// Prefix for every waiting-room key.
pub const WAITING_ROOM_KEY_PREFIX: &str = "agora:waiting";

/// Set of event ids that currently have a non-empty queue (worker sweep list).
pub const ACTIVE_EVENTS_KEY: &str = "agora:waiting:active";

/// Prefix for single-use proof-of-work challenges.
pub const POW_CHALLENGE_PREFIX: &str = "agora:pow";

/// Default admission rate (clients per minute) when no env override is set.
pub const DEFAULT_ADMISSION_RATE_PER_MINUTE: u64 = 60;

/// Default lifetime of a checkout access grant in minutes.
pub const DEFAULT_GRANT_TTL_MINUTES: i64 = 10;

/// The queue zset member score is a per-event sequence number, giving strict FIFO order.
fn queue_zset_key(event_id: &str) -> String {
    format!("{WAITING_ROOM_KEY_PREFIX}:{event_id}:zset")
}

fn queue_seq_key(event_id: &str) -> String {
    format!("{WAITING_ROOM_KEY_PREFIX}:{event_id}:seq")
}

fn queue_config_key(event_id: &str) -> String {
    format!("{WAITING_ROOM_KEY_PREFIX}:{event_id}:config")
}

fn queue_bucket_key(event_id: &str) -> String {
    format!("{WAITING_ROOM_KEY_PREFIX}:{event_id}:bucket")
}

fn grant_key(event_id: &str, client_id: &str) -> String {
    format!("{WAITING_ROOM_KEY_PREFIX}:{event_id}:grant:{client_id}")
}

/// Key under which a single-use PoW challenge is stored (value = event_id).
pub fn pow_challenge_key(challenge: &str) -> String {
    format!("{POW_CHALLENGE_PREFIX}:{challenge}")
}

// ---------------------------------------------------------------------------
// Types
// ---------------------------------------------------------------------------

/// Queue membership state reported to clients.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize)]
#[serde(rename_all = "lowercase")]
pub enum QueueStatusKind {
    /// Client is in the line, waiting for admission.
    Waiting,
    /// Client has been admitted and holds a checkout access grant.
    Admitted,
}

/// Position / admission payload returned by `join` and `status`.
#[derive(Debug, Clone, Serialize)]
pub struct QueueStatus {
    pub status: QueueStatusKind,
    /// 1-based position in line, or `None` when admitted / not in queue.
    pub position: Option<u64>,
    /// Total number of clients currently in line.
    pub queue_size: u64,
    /// Estimated seconds until admission (`position / rate * 60`).
    pub estimated_wait_seconds: Option<u64>,
    /// Signed checkout access grant — present only when `status == admitted`.
    pub grant_token: Option<String>,
}

/// Claims embedded in the signed checkout access grant (JWT).
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct GrantClaims {
    /// Client identifier (e.g. wallet address / device id).
    pub sub: String,
    /// Event the grant admits the client to.
    pub event_id: String,
    /// Always `"checkout_access"` — allows callers to reject cross-purpose tokens.
    pub purpose: String,
    /// Issued-at timestamp (Unix seconds).
    pub iat: i64,
    /// Expiry timestamp (Unix seconds).
    pub exp: i64,
    /// Unique grant id (prevents token reuse confusion).
    pub jti: String,
}

/// The purpose string embedded in every checkout access grant.
pub const GRANT_PURPOSE: &str = "checkout_access";

// ---------------------------------------------------------------------------
// Pure helpers (unit-testable without Redis)
// ---------------------------------------------------------------------------

/// Admission rate (clients per minute) for an event based on its inventory.
///
/// * `total_tickets <= 0` – the event has no inventory cap → use `max_rate`.
/// * `total_tickets - minted_tickets <= 0` – sold out → no admissions.
/// * otherwise `min(remaining, max_rate)` so we never admit faster than the
///   venue can actually sell.
pub fn admission_rate_for_inventory(
    total_tickets: i64,
    minted_tickets: i64,
    max_rate_per_minute: u64,
) -> u64 {
    if total_tickets <= 0 {
        return max_rate_per_minute;
    }
    let remaining = total_tickets - minted_tickets;
    match remaining {
        r if r <= 0 => 0,
        r => (r as u64).min(max_rate_per_minute),
    }
}

/// Estimated wait (seconds) for a 1-based position at a given admission rate.
pub fn estimated_wait_seconds(position: u64, rate_per_minute: u64) -> u64 {
    if rate_per_minute == 0 {
        return 0;
    }
    ((position as f64) / (rate_per_minute as f64) * 60.0).ceil() as u64
}

// ---------------------------------------------------------------------------
// Grant token issuance / verification
// ---------------------------------------------------------------------------

/// Issue a cryptographically signed checkout access grant for `client_id`.
pub fn issue_grant_token(
    client_id: &str,
    event_id: &str,
    ttl_minutes: i64,
) -> Result<String, AppError> {
    let now = Utc::now();
    let claims = GrantClaims {
        sub: client_id.to_string(),
        event_id: event_id.to_string(),
        purpose: GRANT_PURPOSE.to_string(),
        iat: now.timestamp(),
        exp: (now + Duration::minutes(ttl_minutes)).timestamp(),
        jti: Uuid::new_v4().to_string(),
    };
    encode(
        &Header::default(),
        &claims,
        &EncodingKey::from_secret(jwt_secret().as_bytes()),
    )
    .map_err(|e| AppError::InternalServerError(format!("Failed to issue checkout grant: {e}")))
}

/// Verify a checkout access grant token and return its claims.
pub fn verify_grant_token(token: &str) -> Result<GrantClaims, AppError> {
    let mut validation = Validation::new(Algorithm::HS256);
    validation.validate_exp = true;
    decode::<GrantClaims>(
        token,
        &DecodingKey::from_secret(jwt_secret().as_bytes()),
        &validation,
    )
    .map(|data| data.claims)
    .map_err(|e| AppError::AuthError(format!("Invalid or expired checkout grant: {e}")))
}

// ---------------------------------------------------------------------------
// Queue engine
// ---------------------------------------------------------------------------

/// Lua script that atomically refills an event's admission token bucket and
/// pops the head of its queue. Returns the admitted client ids.
///
/// KEYS[1] = bucket hash key (fields: `tokens`, `last_refill`)
/// KEYS[2] = queue zset key
/// ARGV[1] = now (ms)
/// ARGV[2] = refill rate (tokens per second)
/// ARGV[3] = bucket capacity (max stored tokens)
/// ARGV[4] = max clients to admit in this call
const ADMIT_LUA: &str = r#"
local tokens = redis.call('HGET', KEYS[1], 'tokens')
local last = redis.call('HGET', KEYS[1], 'last_refill')
if not tokens then tokens = 0 else tokens = tonumber(tokens) end
if not last then last = tonumber(ARGV[1]) else last = tonumber(last) end
local elapsed = (tonumber(ARGV[1]) - last) / 1000.0
tokens = tokens + elapsed * tonumber(ARGV[2])
if tokens > tonumber(ARGV[3]) then tokens = tonumber(ARGV[3]) end
if tokens < 1 then
  redis.call('HSET', KEYS[1], 'tokens', tokens, 'last_refill', ARGV[1])
  return {}
end
local take = math.min(math.floor(tokens), tonumber(ARGV[4]))
local members = redis.call('ZRANGE', KEYS[2], 0, take - 1)
local admitted = {}
if #members > 0 then
  redis.call('ZREM', KEYS[2], unpack(members))
  admitted = members
end
redis.call('HSET', KEYS[1], 'tokens', tokens - #members, 'last_refill', ARGV[1])
return admitted
"#;

/// The virtual waiting room engine.
///
/// Cheap to clone (`RedisCache` wraps a multiplexed connection manager), so it
/// can be shared across handlers and background tasks.
#[derive(Clone)]
pub struct QueueEngine {
    redis: RedisCache,
}

impl QueueEngine {
    pub fn new(redis: RedisCache) -> Self {
        Self { redis }
    }

    /// Clone the underlying Redis connection for ad-hoc commands (e.g. PoW
    /// challenge persistence in the handlers).
    pub fn redis_connection(&self) -> redis::aio::ConnectionManager {
        self.redis.connection()
    }

    // -- public API ---------------------------------------------------------

    /// Add `client_id` to the FIFO queue for `event_id` (idempotent) and
    /// report their position. Returns an `Admitted` status immediately when
    /// the client already holds a grant.
    pub async fn join(
        &self,
        event_id: &str,
        client_id: &str,
        rate_per_minute: u64,
    ) -> Result<QueueStatus, AppError> {
        if rate_per_minute == 0 {
            return Err(AppError::Conflict(
                "This event is sold out; the waiting room is closed".to_string(),
            ));
        }

        let mut conn = self.redis.connection();

        // Already admitted → hand back the grant instead of re-queueing.
        if let Some(token) = self.grant_for(event_id, client_id).await? {
            return Ok(QueueStatus {
                status: QueueStatusKind::Admitted,
                position: None,
                queue_size: self.queue_size(event_id).await?,
                estimated_wait_seconds: Some(0),
                grant_token: Some(token),
            });
        }

        // FIFO enqueue: score with an atomic per-event sequence number.
        let seq: i64 = redis::cmd("INCR")
            .arg(queue_seq_key(event_id))
            .query_async(&mut conn)
            .await
            .map_err(redis_err)?;

        let _added: i64 = redis::cmd("ZADD")
            .arg(queue_zset_key(event_id))
            .arg("NX")
            .arg(seq)
            .arg(client_id)
            .query_async(&mut conn)
            .await
            .map_err(redis_err)?;

        // Keep the worker sweeping this event and persist its admission rate
        // (first join wins; later inventory changes don't yank the rate).
        let _: i64 = redis::cmd("SADD")
            .arg(ACTIVE_EVENTS_KEY)
            .arg(event_id)
            .query_async(&mut conn)
            .await
            .map_err(redis_err)?;

        let _: bool = redis::cmd("HSETNX")
            .arg(queue_config_key(event_id))
            .arg("rate_per_minute")
            .arg(rate_per_minute)
            .query_async(&mut conn)
            .await
            .map_err(redis_err)?;

        // `_added` is 0 on idempotent re-join (member already queued).

        let rank: Option<i64> = redis::cmd("ZRANK")
            .arg(queue_zset_key(event_id))
            .arg(client_id)
            .query_async(&mut conn)
            .await
            .map_err(redis_err)?;

        let position = rank.map(|r| (r + 1) as u64).unwrap_or(1);
        let queue_size = self.queue_size(event_id).await?;

        Ok(QueueStatus {
            status: QueueStatusKind::Waiting,
            position: Some(position),
            queue_size,
            estimated_wait_seconds: Some(estimated_wait_seconds(position, rate_per_minute)),
            grant_token: None,
        })
    }

    /// Current queue status for `client_id`; `None` if they are not in the
    /// queue and hold no grant.
    pub async fn status(
        &self,
        event_id: &str,
        client_id: &str,
    ) -> Result<Option<QueueStatus>, AppError> {
        let mut conn = self.redis.connection();

        if let Some(token) = self.grant_for(event_id, client_id).await? {
            return Ok(Some(QueueStatus {
                status: QueueStatusKind::Admitted,
                position: None,
                queue_size: self.queue_size(event_id).await?,
                estimated_wait_seconds: Some(0),
                grant_token: Some(token),
            }));
        }

        let rank: Option<i64> = redis::cmd("ZRANK")
            .arg(queue_zset_key(event_id))
            .arg(client_id)
            .query_async(&mut conn)
            .await
            .map_err(redis_err)?;

        let Some(rank) = rank else {
            return Ok(None);
        };

        let position = (rank + 1) as u64;
        let rate = self.rate_per_minute(event_id).await?;
        let queue_size = self.queue_size(event_id).await?;

        Ok(Some(QueueStatus {
            status: QueueStatusKind::Waiting,
            position: Some(position),
            queue_size,
            estimated_wait_seconds: Some(estimated_wait_seconds(position, rate)),
            grant_token: None,
        }))
    }

    /// Spawn the background admission worker. Runs `tick`-periodically,
    /// draining every active event queue at its token-bucket rate and issuing
    /// signed grants with the given TTL.
    pub fn spawn_admission_worker(
        engine: std::sync::Arc<Self>,
        tick: StdDuration,
        grant_ttl_minutes: i64,
    ) {
        tokio::spawn(async move {
            let mut interval = tokio::time::interval(tick);
            interval.set_missed_tick_behavior(tokio::time::MissedTickBehavior::Skip);
            loop {
                interval.tick().await;
                match engine.admit_all(grant_ttl_minutes).await {
                    Ok(total) => {
                        if total > 0 {
                            tracing::info!(admitted = total, "Waiting room: admitted clients");
                        }
                    }
                    Err(e) => {
                        tracing::error!(error = %e, "Waiting room admission worker error");
                    }
                }
            }
        });
    }

    // -- internals ----------------------------------------------------------

    /// Admit clients from every active event queue.
    async fn admit_all(&self, grant_ttl_minutes: i64) -> Result<usize, AppError> {
        let mut conn = self.redis.connection();
        let event_ids: Vec<String> = redis::cmd("SMEMBERS")
            .arg(ACTIVE_EVENTS_KEY)
            .query_async(&mut conn)
            .await
            .map_err(redis_err)?;

        let mut total = 0usize;
        for event_id in event_ids {
            total += self.admit_for_event(&event_id, grant_ttl_minutes).await?;
        }
        Ok(total)
    }

    /// Admit up to the token-bucket allowance for one event, issuing grants.
    async fn admit_for_event(
        &self,
        event_id: &str,
        grant_ttl_minutes: i64,
    ) -> Result<usize, AppError> {
        let mut conn = self.redis.connection();

        let rate = self.rate_per_minute(event_id).await?;
        if rate == 0 {
            return Ok(0);
        }

        let now_ms = Utc::now().timestamp_millis();
        let refill_per_sec = rate as f64 / 60.0;
        let capacity = rate as f64;
        let max_admit = capacity.ceil() as i64;

        let admitted: Vec<String> = redis::cmd("EVAL")
            .arg(ADMIT_LUA)
            .arg(2) // numkeys
            .arg(queue_bucket_key(event_id))
            .arg(queue_zset_key(event_id))
            .arg(now_ms)
            .arg(refill_per_sec)
            .arg(capacity)
            .arg(max_admit)
            .query_async(&mut conn)
            .await
            .map_err(redis_err)?;

        for client_id in &admitted {
            let token = issue_grant_token(client_id, event_id, grant_ttl_minutes)?;
            let _: () = redis::cmd("SET")
                .arg(grant_key(event_id, client_id))
                .arg(&token)
                .arg("EX")
                .arg((grant_ttl_minutes * 60).max(60))
                .query_async(&mut conn)
                .await
                .map_err(redis_err)?;
            tracing::info!(
                event_id = %event_id,
                client_id = %client_id,
                "Waiting room: issued checkout access grant"
            );
        }

        // Stop sweeping the event once its queue is empty.
        let remaining: i64 = redis::cmd("ZCARD")
            .arg(queue_zset_key(event_id))
            .query_async(&mut conn)
            .await
            .map_err(redis_err)?;
        if remaining == 0 {
            let _: i64 = redis::cmd("SREM")
                .arg(ACTIVE_EVENTS_KEY)
                .arg(event_id)
                .query_async(&mut conn)
                .await
                .map_err(redis_err)?;
        }

        Ok(admitted.len())
    }

    /// The configured admission rate for an event (fallback: default).
    async fn rate_per_minute(&self, event_id: &str) -> Result<u64, AppError> {
        let mut conn = self.redis.connection();
        let entries: Vec<(String, String)> = redis::cmd("HGETALL")
            .arg(queue_config_key(event_id))
            .query_async(&mut conn)
            .await
            .map_err(redis_err)?;

        Ok(entries
            .into_iter()
            .find(|(k, _)| k == "rate_per_minute")
            .and_then(|(_, v)| v.parse().ok())
            .unwrap_or(DEFAULT_ADMISSION_RATE_PER_MINUTE))
    }

    /// Fetch a stored grant token for a client, if any.
    async fn grant_for(&self, event_id: &str, client_id: &str) -> Result<Option<String>, AppError> {
        let mut conn = self.redis.connection();
        let token: Option<String> = redis::cmd("GET")
            .arg(grant_key(event_id, client_id))
            .query_async(&mut conn)
            .await
            .map_err(redis_err)?;
        Ok(token)
    }

    /// Number of clients currently in the queue for an event.
    async fn queue_size(&self, event_id: &str) -> Result<u64, AppError> {
        let mut conn = self.redis.connection();
        let size: i64 = redis::cmd("ZCARD")
            .arg(queue_zset_key(event_id))
            .query_async(&mut conn)
            .await
            .map_err(redis_err)?;
        Ok(size.max(0) as u64)
    }
}

fn redis_err(e: redis::RedisError) -> AppError {
    AppError::ExternalServiceError(format!("Redis error: {e}"))
}

// ---------------------------------------------------------------------------
// Tests
// ---------------------------------------------------------------------------

#[cfg(test)]
mod tests {
    use super::*;

    fn ensure_test_jwt_secret() {
        std::env::set_var("JWT_SECRET", "test-secret-for-waiting-room-unit-tests-32b");
    }

    #[test]
    fn test_admission_rate_scales_with_inventory() {
        // No inventory cap → max rate.
        assert_eq!(admission_rate_for_inventory(0, 0, 60), 60);
        // Plenty of inventory → max rate.
        assert_eq!(admission_rate_for_inventory(500, 10, 60), 60);
        // Limited inventory → rate clamped to what remains.
        assert_eq!(admission_rate_for_inventory(100, 85, 60), 15);
        assert_eq!(admission_rate_for_inventory(100, 99, 60), 1);
        // Sold out → no admissions.
        assert_eq!(admission_rate_for_inventory(100, 100, 60), 0);
        assert_eq!(admission_rate_for_inventory(100, 120, 60), 0);
        // Small venue but large configured max → still clamped.
        assert_eq!(admission_rate_for_inventory(3, 0, 600), 3);
    }

    #[test]
    fn test_estimated_wait_seconds() {
        // #1 at 60/min → ~1s.
        assert_eq!(estimated_wait_seconds(1, 60), 1);
        // #142 at 60/min → ceil(142/60*60) = 142s.
        assert_eq!(estimated_wait_seconds(142, 60), 142);
        // #30 at 120/min → ceil(15) = 15s.
        assert_eq!(estimated_wait_seconds(30, 120), 15);
        // Zero rate → 0 (defensive).
        assert_eq!(estimated_wait_seconds(10, 0), 0);
    }

    #[test]
    fn test_grant_token_roundtrip() {
        ensure_test_jwt_secret();
        let token = issue_grant_token("GCLIENT123", "550e8400-e29b-41d4-a716-446655440000", 10)
            .expect("grant should be issued");
        let claims = verify_grant_token(&token).expect("grant should verify");
        assert_eq!(claims.sub, "GCLIENT123");
        assert_eq!(claims.purpose, GRANT_PURPOSE);
        assert_eq!(claims.event_id, "550e8400-e29b-41d4-a716-446655440000");
        assert!(claims.exp > claims.iat);
        assert!(!claims.jti.is_empty());
    }

    #[test]
    fn test_grant_token_rejects_garbage() {
        ensure_test_jwt_secret();
        assert!(verify_grant_token("not.a.grant").is_err());
    }

    #[test]
    fn test_key_builders_are_namespaced() {
        assert_eq!(
            queue_zset_key("e1"),
            format!("{WAITING_ROOM_KEY_PREFIX}:e1:zset")
        );
        assert_eq!(
            grant_key("e1", "c1"),
            format!("{WAITING_ROOM_KEY_PREFIX}:e1:grant:c1")
        );
        assert_eq!(
            pow_challenge_key("abc"),
            format!("{POW_CHALLENGE_PREFIX}:abc")
        );
    }
}
