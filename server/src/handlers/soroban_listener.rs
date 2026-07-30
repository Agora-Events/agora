//! # Soroban Event Listener & Redis Background Job Queue
//!
//! A background service that long-polls the Stellar RPC node for `ContractEvent`
//! objects emitted by the `ticket_payment` and `event_registry` contracts.
//!
//! ## Architecture
//! - Polling loop fetches contract events and pushes raw RPC events into a Redis-backed
//!   background job queue (`soroban:job_queue`).
//! - A worker pool consumes jobs asynchronously from the queue.
//! - Failed jobs are retried with exponential backoff.
//! - Jobs exceeding `MAX_JOB_RETRIES` are pushed to a Dead-Letter Queue (`soroban:dead_letter_queue`)
//!   and logged at `error!` level.

use serde::{Deserialize, Serialize};
use serde_json::Value;
use sqlx::PgPool;
use std::sync::Arc;
use std::time::Duration;
use tokio::sync::mpsc;
use tokio::time::sleep;

/// Minimum ledger confirmations before an event is considered final.
const MIN_CONFIRMATIONS: u32 = 2;

/// How often to poll the RPC node for new events (base interval).
const POLL_INTERVAL: Duration = Duration::from_secs(5);

/// Maximum back-off delay between retries after consecutive RPC failures.
const MAX_BACKOFF: Duration = Duration::from_secs(300);

/// Maximum events to fetch per poll cycle.
const MAX_EVENTS_PER_POLL: u32 = 100;

/// Redis key for persisting the last processed event cursor.
#[allow(dead_code)]
pub const CURSOR_CACHE_KEY: &str = "soroban:event_cursor";

/// Redis key for the main job queue.
pub const JOB_QUEUE_KEY: &str = "soroban:job_queue";

/// Redis key for the dead-letter queue.
pub const DEAD_LETTER_QUEUE_KEY: &str = "soroban:dead_letter_queue";

/// Default maximum retry attempts for failed jobs.
pub const MAX_JOB_RETRIES: u32 = 5;

/// Number of concurrent background worker tasks.
pub const WORKER_POOL_SIZE: usize = 4;

// ---------------------------------------------------------------------------
// RPC & Job Queue types
// ---------------------------------------------------------------------------

/// Stellar RPC `getEvents` request body.
#[derive(Debug, Serialize)]
struct GetEventsRequest {
    jsonrpc: &'static str,
    id: u32,
    method: &'static str,
    params: GetEventsParams,
}

#[derive(Debug, Serialize)]
struct GetEventsParams {
    #[serde(rename = "startLedger")]
    start_ledger: Option<u32>,
    filters: Vec<EventFilter>,
    pagination: EventPagination,
}

#[derive(Debug, Serialize)]
struct EventFilter {
    #[serde(rename = "type")]
    event_type: &'static str,
    #[serde(rename = "contractIds")]
    contract_ids: Vec<String>,
}

#[derive(Debug, Serialize)]
struct EventPagination {
    limit: u32,
    cursor: Option<String>,
}

/// Stellar RPC `getEvents` response.
#[derive(Debug, Deserialize)]
struct GetEventsResponse {
    result: Option<GetEventsResult>,
    error: Option<Value>,
}

#[derive(Debug, Deserialize)]
struct GetEventsResult {
    events: Vec<SorobanEvent>,
    #[serde(rename = "latestLedger")]
    latest_ledger: u32,
}

/// A single Soroban contract event from the RPC.
#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct SorobanEvent {
    /// Opaque pagination cursor for this event.
    pub id: String,
    /// Ledger sequence number where this event was emitted.
    #[serde(rename = "ledger")]
    pub ledger: u32,
    /// Contract that emitted the event.
    #[serde(rename = "contractId")]
    pub contract_id: String,
    /// XDR-encoded topic array (base64).
    pub topic: Vec<String>,
    /// XDR-encoded event data (base64).
    pub value: Value,
    /// Ledger close time (Unix timestamp).
    #[serde(rename = "ledgerClosedAt")]
    pub ledger_closed_at: Option<String>,
}

/// A background job wrapping a Soroban event for asynchronous processing.
#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct SorobanJob {
    pub id: String,
    pub event: SorobanEvent,
    pub retry_count: u32,
    pub max_retries: u32,
    pub created_at: i64,
}

impl SorobanJob {
    pub fn new(event: SorobanEvent) -> Self {
        let job_id = format!("job_{}_{}", event.id, event.ledger);
        Self {
            id: job_id,
            event,
            retry_count: 0,
            max_retries: MAX_JOB_RETRIES,
            created_at: chrono::Utc::now().timestamp(),
        }
    }

    /// Calculate exponential backoff duration for retries: 2^retry_count seconds (capped at 60s).
    pub fn backoff_duration(&self) -> Duration {
        let secs = 2u64.pow(self.retry_count).min(60);
        Duration::from_secs(secs)
    }
}

// ---------------------------------------------------------------------------
// Listener state & configuration
// ---------------------------------------------------------------------------

/// Configuration for the Soroban event listener.
#[derive(Clone)]
pub struct ListenerConfig {
    /// Stellar RPC endpoint URL.
    pub rpc_url: String,
    /// Contract ID of the `ticket_payment` contract.
    pub ticket_payment_contract_id: String,
    /// Contract ID of the `event_registry` contract.
    pub event_registry_contract_id: String,
    /// Ledger to start scanning from (used only on first run).
    pub start_ledger: u32,
    /// Optional Redis URL for job queue persistence.
    pub redis_url: Option<String>,
}

impl ListenerConfig {
    /// Build from environment variables with sensible defaults.
    pub fn from_env() -> Self {
        Self {
            rpc_url: std::env::var("SOROBAN_RPC_URL")
                .unwrap_or_else(|_| "https://soroban-testnet.stellar.org".to_string()),
            ticket_payment_contract_id: std::env::var("TICKET_PAYMENT_CONTRACT_ID")
                .unwrap_or_default(),
            event_registry_contract_id: std::env::var("EVENT_REGISTRY_CONTRACT_ID")
                .unwrap_or_default(),
            start_ledger: std::env::var("SOROBAN_START_LEDGER")
                .ok()
                .and_then(|v| v.parse().ok())
                .unwrap_or(0),
            redis_url: std::env::var("REDIS_URL").ok(),
        }
    }
}

// ---------------------------------------------------------------------------
// Main listener loop & worker pool
// ---------------------------------------------------------------------------

/// Spawn the Soroban event listener as a background task along with the worker pool.
pub fn spawn_listener(pool: PgPool, config: ListenerConfig) {
    tokio::spawn(async move {
        run_listener(pool, redis, config).await;
    });
}

async fn run_listener(pool: PgPool, config: ListenerConfig) {
    if config.ticket_payment_contract_id.is_empty() && config.event_registry_contract_id.is_empty()
    {
        tracing::info!(
            "Soroban listener: no contract IDs configured, skipping. \
             Set TICKET_PAYMENT_CONTRACT_ID and/or EVENT_REGISTRY_CONTRACT_ID to enable."
        );
        return;
    }

    let (job_tx, job_rx) = mpsc::channel::<SorobanJob>(1000);
    let job_rx = Arc::new(tokio::sync::Mutex::new(job_rx));

    // Initialize Redis client if REDIS_URL is configured
    let redis_client = config
        .redis_url
        .as_ref()
        .and_then(|url| redis::Client::open(url.as_str()).ok());

    // Spawn Worker Pool
    for worker_id in 0..WORKER_POOL_SIZE {
        let pool = pool.clone();
        let config = config.clone();
        let rx = Arc::clone(&job_rx);
        let redis = redis_client.clone();

        tokio::spawn(async move {
            run_worker(worker_id, pool, config, rx, redis).await;
        });
    }

    let http = reqwest::Client::new();
    let mut cursor: Option<String> = None;

    if let Some(ref mut r) = redis {
        if let Ok(Some(cached_cursor)) = r.get::<String>(CURSOR_CACHE_KEY).await {
            tracing::info!("Loaded Soroban event cursor from Redis: {}", cached_cursor);
            cursor = Some(cached_cursor);
        }
    }

    let mut start_ledger = if cursor.is_none() { Some(config.start_ledger) } else { None };
    let mut current_backoff = POLL_INTERVAL;

    tracing::info!(
        "Soroban listener started with Redis job queue & {} workers. RPC={} poll_interval={:?}",
        WORKER_POOL_SIZE,
        config.rpc_url,
        POLL_INTERVAL
    );

    loop {
        match poll_events(&http, &config, cursor.clone(), start_ledger).await {
            Ok(Some(result)) => {
                let latest_ledger = result.latest_ledger;

                for event in &result.events {
                    if latest_ledger.saturating_sub(event.ledger) < MIN_CONFIRMATIONS {
                        tracing::debug!(
                            "Skipping event {} (ledger {} not yet confirmed, latest={})",
                            event.id,
                            event.ledger,
                            latest_ledger
                        );
                        continue;
                    }

                    let job = SorobanJob::new(event.clone());
                    push_job(&job, &redis_client, &job_tx).await;
                }

                if let Some(last) = result.events.last() {
                    cursor = Some(last.id.clone());
                    start_ledger = None;

                    if let Some(ref mut r) = redis {
                        if let Err(e) = r.set(CURSOR_CACHE_KEY, &last.id, Duration::from_secs(86400 * 30)).await {
                            tracing::warn!("Failed to persist Soroban event cursor to Redis: {:?}", e);
                        }
                    }
                }

                current_backoff = POLL_INTERVAL;

                // Full page means more events may be waiting — fetch next page
                // immediately instead of waiting for POLL_INTERVAL.
                if should_immediately_fetch_next_page(result.events.len()) {
                    continue;
                }
            }
            Ok(None) => {
                current_backoff = POLL_INTERVAL;
            }
            Err(e) => {
                tracing::error!(
                    "Soroban listener poll error (retrying in {:?}): {:?}",
                    current_backoff,
                    e
                );
                sleep(current_backoff).await;
                current_backoff = (current_backoff * 2).min(MAX_BACKOFF);
                continue;
            }
        }

        sleep(POLL_INTERVAL).await;
    }
}

/// Push a job into Redis (if available) or into the fallback MPSC channel.
async fn push_job(
    job: &SorobanJob,
    redis_client: &Option<redis::Client>,
    job_tx: &mpsc::Sender<SorobanJob>,
) {
    if let Some(client) = redis_client {
        if let Ok(mut conn) = client.get_tokio_connection().await {
            if let Ok(json) = serde_json::to_string(job) {
                let res: Result<(), redis::RedisError> = redis::cmd("RPUSH")
                    .arg(JOB_QUEUE_KEY)
                    .arg(json)
                    .query_async(&mut conn)
                    .await;
                if res.is_ok() {
                    tracing::debug!("Pushed job {} to Redis queue {}", job.id, JOB_QUEUE_KEY);
                    return;
                }
            }
        }
    }

    // Fallback to MPSC channel
    if let Err(e) = job_tx.send(job.clone()).await {
        tracing::error!("Failed to enqueue job {} to internal channel: {:?}", job.id, e);
    }
}

/// Worker consumer task that pops jobs, executes event processing, and handles retries / DLQ.
async fn run_worker(
    worker_id: usize,
    pool: PgPool,
    config: ListenerConfig,
    job_rx: Arc<tokio::sync::Mutex<mpsc::Receiver<SorobanJob>>>,
    redis_client: Option<redis::Client>,
) {
    tracing::debug!("Soroban queue worker {} started", worker_id);

    loop {
        let mut popped_job: Option<SorobanJob> = None;

        // 1. Try popping from Redis BLPOP if connected
        if let Some(ref client) = redis_client {
            if let Ok(mut conn) = client.get_tokio_connection().await {
                let res: Result<Option<(String, String)>, redis::RedisError> = redis::cmd("BLPOP")
                    .arg(JOB_QUEUE_KEY)
                    .arg(2) // 2 second timeout
                    .query_async(&mut conn)
                    .await;

                if let Ok(Some((_, payload))) = res {
                    if let Ok(job) = serde_json::from_str::<SorobanJob>(&payload) {
                        popped_job = Some(job);
                    }
                }
            }
        }

        // 2. If Redis had no item or is disabled, try MPSC channel
        if popped_job.is_none() {
            let mut rx = job_rx.lock().await;
            if let Ok(job) = tokio::time::timeout(Duration::from_millis(500), rx.recv()).await {
                popped_job = job;
            }
        }

        let mut job = match popped_job {
            Some(j) => j,
            None => continue,
        };

        // 3. Execute event persistence logic
        match process_event(&pool, &job.event, &config).await {
            Ok(()) => {
                tracing::info!("Worker {} successfully processed job {}", worker_id, job.id);
            }
            Err(err_msg) => {
                tracing::warn!(
                    "Worker {} failed processing job {} (attempt {}/{}): {}",
                    worker_id,
                    job.id,
                    job.retry_count + 1,
                    job.max_retries,
                    err_msg
                );

                if job.retry_count < job.max_retries {
                    job.retry_count += 1;
                    let backoff = job.backoff_duration();
                    tracing::info!(
                        "Retrying job {} in {:?} (attempt {})",
                        job.id,
                        backoff,
                        job.retry_count
                    );
                    sleep(backoff).await;

                    // Re-enqueue for retry
                    if let Some(ref client) = redis_client {
                        if let Ok(mut conn) = client.get_tokio_connection().await {
                            if let Ok(json) = serde_json::to_string(&job) {
                                let _: Result<(), _> = redis::cmd("RPUSH")
                                    .arg(JOB_QUEUE_KEY)
                                    .arg(json)
                                    .query_async(&mut conn)
                                    .await;
                            }
                        }
                    }
                } else {
                    // Exceeded max retries -> Move to Dead-Letter Queue
                    tracing::error!(
                        "Job {} for event {} exceeded max retries ({}), moving to dead-letter queue. Error: {}",
                        job.id,
                        job.event.id,
                        job.max_retries,
                        err_msg
                    );

                    if let Some(ref client) = redis_client {
                        if let Ok(mut conn) = client.get_tokio_connection().await {
                            if let Ok(json) = serde_json::to_string(&job) {
                                let _: Result<(), _> = redis::cmd("RPUSH")
                                    .arg(DEAD_LETTER_QUEUE_KEY)
                                    .arg(json)
                                    .query_async(&mut conn)
                                    .await;
                            }
                        }
                    }
                }
            }
        }
    }
/// Returns true when a poll returned a full page, indicating more events may
/// be available beyond the current cursor and the listener should retry
/// immediately without waiting for [`POLL_INTERVAL`].
fn should_immediately_fetch_next_page(events_returned: usize) -> bool {
    events_returned == MAX_EVENTS_PER_POLL as usize
}

/// Poll the Stellar RPC node for new contract events.
async fn poll_events(
    http: &reqwest::Client,
    config: &ListenerConfig,
    cursor: Option<String>,
    start_ledger: Option<u32>,
) -> Result<Option<GetEventsResult>, String> {
    let mut contract_ids = Vec::new();
    if !config.ticket_payment_contract_id.is_empty() {
        contract_ids.push(config.ticket_payment_contract_id.clone());
    }
    if !config.event_registry_contract_id.is_empty() {
        contract_ids.push(config.event_registry_contract_id.clone());
    }

    let request = GetEventsRequest {
        jsonrpc: "2.0",
        id: 1,
        method: "getEvents",
        params: GetEventsParams {
            start_ledger,
            filters: vec![EventFilter {
                event_type: "contract",
                contract_ids,
            }],
            pagination: EventPagination {
                limit: MAX_EVENTS_PER_POLL,
                cursor,
            },
        },
    };

    let response = http
        .post(&config.rpc_url)
        .json(&request)
        .timeout(Duration::from_secs(8))
        .send()
        .await
        .map_err(|e| format!("HTTP request failed: {e}"))?;

    let rpc_response: GetEventsResponse = response
        .json()
        .await
        .map_err(|e| format!("Failed to parse RPC response: {e}"))?;

    if let Some(err) = rpc_response.error {
        return Err(format!("RPC error: {err}"));
    }

    match rpc_response.result {
        Some(result) if !result.events.is_empty() => Ok(Some(result)),
        _ => Ok(None),
    }
}

// ---------------------------------------------------------------------------
// Event processing logic
// ---------------------------------------------------------------------------

async fn process_event(
    pool: &PgPool,
    event: &SorobanEvent,
    config: &ListenerConfig,
) -> Result<(), String> {
    let event_name = event.topic.first().map(|t| t.as_str()).unwrap_or("unknown");

    tracing::debug!(
        "Processing event: contract={} name={} ledger={}",
        event.contract_id,
        event_name,
        event.ledger
    );

    if event.contract_id == config.ticket_payment_contract_id {
        match event_name {
            "ticket_purchased" | "purchase_confirmed" => {
                handle_ticket_purchased(pool, event).await?;
            }
            "ticket_refunded" => {
                handle_ticket_refunded(pool, event).await?;
            }
            _ => {
                tracing::debug!("Unhandled ticket_payment event: {}", event_name);
            }
        }
    } else if event.contract_id == config.event_registry_contract_id {
        match event_name {
            "event_registered" => {
                handle_event_registered(pool, event).await?;
            }
            "event_status_updated" | "event_cancelled" => {
                handle_event_status_updated(pool, event).await?;
            }
            "collateral_staked" | "CollateralStaked" => {
                handle_collateral_staked(pool, event).await?;
            }
            "collateral_unstaked" | "CollateralUnstaked" => {
                handle_collateral_unstaked(pool, event).await?;
            }
            _ => {
                tracing::debug!("Unhandled event_registry event: {}", event_name);
            }
        }
    }

    Ok(())
}

async fn handle_ticket_purchased(pool: &PgPool, event: &SorobanEvent) -> Result<(), String> {
    let data = &event.value;

    let event_id = data
        .get("event_id")
        .and_then(|v| v.as_str())
        .unwrap_or_default();
    let buyer_wallet = data
        .get("buyer")
        .and_then(|v| v.as_str())
        .unwrap_or_default();
    let owner_wallet = data
        .get("owner")
        .and_then(|v| v.as_str())
        .unwrap_or(buyer_wallet);
    let stellar_id = data
        .get("stellar_id")
        .and_then(|v| v.as_str())
        .unwrap_or(&event.id);

    if event_id.is_empty() || buyer_wallet.is_empty() {
        tracing::warn!(
            "ticket_purchased event {} missing required fields, skipping",
            event.id
        );
        return Ok(());
    }

    match sqlx::query(
        r#"
        INSERT INTO tickets (stellar_id, event_id, buyer_wallet, owner_wallet, status)
        VALUES ($1, $2::uuid, $3, $4, 'Unused')
        ON CONFLICT (stellar_id) DO NOTHING
        "#,
    )
    .bind(stellar_id)
    .bind(event_id)
    .bind(buyer_wallet)
    .bind(owner_wallet)
    .execute(pool)
    .await
    {
        Ok(result) => {
            if result.rows_affected() > 0 {
                tracing::info!(
                    "Synced on-chain ticket purchase: stellar_id={} event_id={} buyer={}",
                    stellar_id,
                    event_id,
                    buyer_wallet
                );
            }
            Ok(())
        }
        Err(e) => Err(format!("DB error upserting ticket: {e}")),
    }
}

async fn handle_ticket_refunded(pool: &PgPool, event: &SorobanEvent) -> Result<(), String> {
    let stellar_id = event
        .value
        .get("stellar_id")
        .and_then(|v| v.as_str())
        .unwrap_or(&event.id);

    match sqlx::query("UPDATE tickets SET status = 'Revoked' WHERE stellar_id = $1")
        .bind(stellar_id)
        .execute(pool)
        .await
    {
        Ok(_) => {
            tracing::info!(
                "Marked ticket {} as cancelled (on-chain refund)",
                stellar_id
            );
            Ok(())
        }
        Err(e) => Err(format!("DB error cancelling ticket: {e}")),
    }
}

async fn handle_event_registered(_pool: &PgPool, event: &SorobanEvent) -> Result<(), String> {
    let data = &event.value;
    let on_chain_event_id = data
        .get("event_id")
        .and_then(|v| v.as_str())
        .unwrap_or_default();

    if on_chain_event_id.is_empty() {
        return Ok(());
    }

    if let Ok(uuid) = uuid::Uuid::parse_str(on_chain_event_id) {
        sqlx::query("UPDATE events SET updated_at = NOW() WHERE id = $1")
            .bind(uuid)
            .execute(pool)
            .await
            .map_err(|e| format!("DB error updating registered event: {e}"))?;
    }

    tracing::info!(
        "On-chain event registered: event_id={} ledger={}",
        on_chain_event_id,
        event.ledger
    );
    Ok(())
}

async fn handle_event_status_updated(_pool: &PgPool, event: &SorobanEvent) -> Result<(), String> {
    let data = &event.value;
    let on_chain_event_id = data
        .get("event_id")
        .and_then(|v| v.as_str())
        .unwrap_or_default();
    let new_status = data
        .get("status")
        .and_then(|v| v.as_str())
        .unwrap_or("cancelled");

    if on_chain_event_id.is_empty() {
        return Ok(());
    }

    if let Ok(uuid) = uuid::Uuid::parse_str(on_chain_event_id) {
        sqlx::query("UPDATE events SET updated_at = NOW() WHERE id = $1")
            .bind(uuid)
            .execute(pool)
            .await
            .map_err(|e| format!("DB error updating event status: {e}"))?;
    }

    tracing::info!(
        "On-chain event status update: event_id={} status={} ledger={}",
        on_chain_event_id,
        new_status,
        event.ledger
    );
    Ok(())
}

async fn handle_collateral_staked(pool: &PgPool, event: &SorobanEvent) -> Result<(), String> {
    let data = &event.value;
    let organizer_wallet = data
        .get("organizer")
        .and_then(|v| v.as_str())
        .unwrap_or_default();
    let is_verified = data
        .get("is_verified")
        .and_then(|v| v.as_bool())
        .unwrap_or_default();

    if !organizer_wallet.is_empty() {
        sqlx::query(
            "UPDATE organizers SET is_verified = $1, updated_at = NOW() WHERE wallet_address = $2",
        )
        .bind(is_verified)
        .bind(organizer_wallet)
        .execute(pool)
        .await
        .map_err(|e| format!("Failed to update organizer verified status: {}", e))?;
    }

    Ok(())
}

async fn handle_collateral_unstaked(pool: &PgPool, event: &SorobanEvent) -> Result<(), String> {
    let data = &event.value;
    let organizer_wallet = data
        .get("organizer")
        .and_then(|v| v.as_str())
        .unwrap_or_default();

    if !organizer_wallet.is_empty() {
        sqlx::query("UPDATE organizers SET is_verified = FALSE, updated_at = NOW() WHERE wallet_address = $1")
            .bind(organizer_wallet)
            .execute(pool)
            .await
            .map_err(|e| format!("Failed to reset organizer verified status: {}", e))?;
    }

    Ok(())
}

// ---------------------------------------------------------------------------
// Tests
// ---------------------------------------------------------------------------

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_job_creation_and_exponential_backoff() {
        let event = SorobanEvent {
            id: "evt-123".to_string(),
            ledger: 1000,
            contract_id: "CTEST".to_string(),
            topic: vec!["ticket_purchased".to_string()],
            value: serde_json::json!({ "event_id": "e1", "buyer": "G1" }),
            ledger_closed_at: None,
        };

        let mut job = SorobanJob::new(event);
        assert_eq!(job.retry_count, 0);
        assert_eq!(job.backoff_duration(), Duration::from_secs(1));

        job.retry_count = 1;
        assert_eq!(job.backoff_duration(), Duration::from_secs(2));

        job.retry_count = 2;
        assert_eq!(job.backoff_duration(), Duration::from_secs(4));

        job.retry_count = 3;
        assert_eq!(job.backoff_duration(), Duration::from_secs(8));

        job.retry_count = 4;
        assert_eq!(job.backoff_duration(), Duration::from_secs(16));

        job.retry_count = 6;
        assert_eq!(job.backoff_duration(), Duration::from_secs(60));
    }

    #[test]
    fn test_job_serialization() {
        let event = SorobanEvent {
            id: "evt-999".to_string(),
            ledger: 500,
            contract_id: "CPAY".to_string(),
            topic: vec!["event_registered".to_string()],
            value: serde_json::json!({ "event_id": "e99" }),
            ledger_closed_at: None,
        };

        let job = SorobanJob::new(event);
        let serialized = serde_json::to_string(&job).unwrap();
        let deserialized: SorobanJob = serde_json::from_str(&serialized).unwrap();

        assert_eq!(deserialized.id, job.id);
        assert_eq!(deserialized.event.contract_id, "CPAY");
        assert_eq!(deserialized.max_retries, MAX_JOB_RETRIES);
    }

    #[test]
    fn test_immediate_retry_when_full_page_returned() {
        assert!(should_immediately_fetch_next_page(MAX_EVENTS_PER_POLL as usize));
        assert!(!should_immediately_fetch_next_page(
            (MAX_EVENTS_PER_POLL as usize) - 1
        ));
        assert!(!should_immediately_fetch_next_page(0));
        assert!(!should_immediately_fetch_next_page(1));
    }
}
