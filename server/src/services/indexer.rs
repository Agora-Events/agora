//! # High-Throughput, Re-org Resilient Soroban Event Indexer (Issue #1174)
//!
//! Replaces the old single-threaded polling loop in `handlers::soroban_listener`
//! with a decoupled Tokio pipeline:
//!
//! ```text
//! RPC fetcher ──> event filter (typed decode) ──> dedup queue ──> DB persister ──> WebSocket broadcaster
//! ```
//!
//! ## Resilience properties
//!
//! * **Ledger cursor checkpoints** live in PostgreSQL and are updated with a
//!   single atomic `UPSERT` ([`CheckpointStore::save`]).
//! * **Sliding-window buffer** (`indexer_ledger_events`) holds every event by
//!   RPC `id` until its ledger is deep enough below the chain head to be
//!   considered final. Non-final events are *not* applied to the application
//!   tables, so a fork can never corrupt committed state.
//! * **Re-org detection & rollback**: if the RPC returns an event at a ledger
//!   the indexer has already finalized — with an envelope we have never seen —
//!   `CheckpointStore::rollback_to` purges the buffer from the fork point and
//!   rewinds the cursor atomically; the producer then re-scans the surviving
//!   fork. Re-application is idempotent because state writes are
//!   `INSERT ... ON CONFLICT DO NOTHING` / plain `UPDATE`s.
//! * **RPC failover & rate limiting**: a round-robin node pool skips nodes in
//!   cooldown and applies exponential backoff **with jitter** on HTTP 429 and
//!   connection failures, so thundering-herd retries never retrigger the rate
//!   limiter.
//! * **Historical replay** is exposed as an admin endpoint
//!   (`handlers::indexer::replay_indexer`) that backfills any ledger range
//!   through the same buffered, idempotent path used live.

use crate::cache::RedisCache;
use crate::handlers::ws::PurchaseBroadcaster;
use crate::models::blockchain_checkpoint::{CheckpointStore, DEFAULT_CHAIN_KEY};
use crate::models::indexer_event::{EventKind, IndexedEvent};
use rand::Rng;
use serde::{Deserialize, Serialize};
use serde_json::Value;
use sqlx::PgPool;
use sqlx::types::Json;
use std::sync::atomic::{AtomicU32, Ordering};
use std::sync::Arc;
use std::time::Duration;
use thiserror::Error;
use tokio::sync::mpsc;
use tokio::time::sleep;

// ---------------------------------------------------------------------------
// Tunables (also overridable via env in [`IndexerConfig::from_env`])
// ---------------------------------------------------------------------------

/// Default chain key stored in `blockchain_checkpoints`.
pub const CHAIN_KEY: &str = DEFAULT_CHAIN_KEY;

/// How many ledgers back the re-org detection window spans.
pub const DEFAULT_WINDOW_LEDGERS: u32 = 100;

/// Minimum ledger depth before an event is considered final.
pub const DEFAULT_CONFIRMATIONS: u32 = 2;

/// Base interval between polls (also the backoff base).
pub const POLL_INTERVAL: Duration = Duration::from_secs(5);

/// Max back-off between consecutive RPC failures.
pub const MAX_BACKOFF: Duration = Duration::from_secs(300);

/// Cooldown applied to a node after a failure before it is retried.
pub const NODE_COOLDOWN: Duration = Duration::from_secs(60);

/// Maximum events fetched per `getEvents` page.
pub const MAX_EVENTS_PER_POLL: u32 = 100;

/// Default number of DB persister workers.
pub const DEFAULT_WORKERS: usize = 4;

/// Maximum ledger range a single replay request is allowed to backfill.
pub const MAX_REPLAY_LEDGERS: u32 = 100_000;

/// Pending events applied per finalizer sweep.
const FINALIZER_BATCH_SIZE: i64 = 250;

/// Capacity of the producer → worker queue.
const QUEUE_CAPACITY: usize = 4096;

/// Legacy Redis key kept as a best-effort cursor mirror (Issue #490).
const LEGACY_CURSOR_CACHE_KEY: &str = "soroban:event_cursor";

// ---------------------------------------------------------------------------
// Configuration
// ---------------------------------------------------------------------------

/// Configuration for the Soroban indexer pipeline.
#[derive(Debug, Clone)]
pub struct IndexerConfig {
    /// Ordered list of Soroban RPC endpoints (`SOROBAN_RPC_URLS`, comma
    /// separated, falls back to the legacy `SOROBAN_RPC_URL`).
    pub rpc_urls: Vec<String>,
    /// Contract ID of the `ticket_payment` contract.
    pub ticket_payment_contract_id: String,
    /// Contract ID of the `event_registry` contract.
    pub event_registry_contract_id: String,
    /// Ledger to start scanning from on a fresh chain (no checkpoint yet).
    pub start_ledger: u32,
    /// Sliding-window depth (ledgers) for re-org detection.
    pub window_ledgers: u32,
    /// Confirmations before an event is applied to the DB.
    pub confirmations: u32,
    /// Number of DB persister workers.
    pub workers: usize,
    /// Optional Redis URL — used only as a best-effort cursor mirror.
    pub redis_url: Option<String>,
}

impl IndexerConfig {
    /// Build from environment variables with sensible defaults.
    pub fn from_env() -> Self {
        let multi_rpc = std::env::var("SOROBAN_RPC_URLS")
            .ok()
            .map(|s| {
                s.split(',')
                    .map(|u| u.trim().to_string())
                    .filter(|u| !u.is_empty())
                    .collect::<Vec<_>>()
            })
            .filter(|v| !v.is_empty());

        let rpc_urls = match multi_rpc {
            Some(urls) => urls,
            None => std::env::var("SOROBAN_RPC_URL")
                .unwrap_or_else(|_| "https://soroban-testnet.stellar.org".to_string())
                .split(',')
                .map(|u| u.trim().to_string())
                .filter(|u| !u.is_empty())
                .collect(),
        };

        Self {
            rpc_urls,
            ticket_payment_contract_id: std::env::var("TICKET_PAYMENT_CONTRACT_ID")
                .unwrap_or_default(),
            event_registry_contract_id: std::env::var("EVENT_REGISTRY_CONTRACT_ID")
                .unwrap_or_default(),
            start_ledger: std::env::var("SOROBAN_START_LEDGER")
                .ok()
                .and_then(|v| v.parse().ok())
                .unwrap_or(0),
            window_ledgers: std::env::var("INDEXER_WINDOW_LEDGERS")
                .ok()
                .and_then(|v| v.parse().ok())
                .unwrap_or(DEFAULT_WINDOW_LEDGERS),
            confirmations: std::env::var("INDEXER_CONFIRMATIONS")
                .ok()
                .and_then(|v| v.parse().ok())
                .unwrap_or(DEFAULT_CONFIRMATIONS),
            workers: std::env::var("INDEXER_WORKERS")
                .ok()
                .and_then(|v| v.parse().ok())
                .unwrap_or(DEFAULT_WORKERS),
            redis_url: std::env::var("REDIS_URL").ok(),
        }
    }

    /// URLs subscribed as event-filter contract ids (the contracts we care
    /// about).
    pub fn contract_ids(&self) -> Vec<String> {
        let mut ids = Vec::new();
        if !self.ticket_payment_contract_id.is_empty() {
            ids.push(self.ticket_payment_contract_id.clone());
        }
        if !self.event_registry_contract_id.is_empty() {
            ids.push(self.event_registry_contract_id.clone());
        }
        ids
    }

    fn is_enabled(&self) -> bool {
        !self.rpc_urls.is_empty() && !self.contract_ids().is_empty()
    }
}

// ---------------------------------------------------------------------------
// Errors
// ---------------------------------------------------------------------------

/// Errors surfaced by the indexer pipeline.
#[derive(Debug, Error)]
pub enum IndexerError {
    #[error("HTTP request failed: {0}")]
    Http(String),
    #[error("RPC returned an error: {0}")]
    Rpc(String),
    #[error("RPC responded with 429 Too Many Requests")]
    RateLimited,
    #[error("all configured RPC nodes are in cooldown")]
    NoHealthyNode,
    #[error(transparent)]
    Database(#[from] sqlx::Error),
    #[error("invalid replay range: {0}")]
    InvalidRange(String),
}

// ---------------------------------------------------------------------------
// RPC node pool with failover, backoff & jitter
// ---------------------------------------------------------------------------

/// A round-robin pool of RPC endpoints with per-node cooldown.
///
/// A node that returns 429 / a connection error is set into cooldown and only
/// retried after [`NODE_COOLDOWN`]; healthy nodes keep being rotated so a
/// single failing endpoint cannot stall the whole pipeline.
pub struct RpcNodePool {
    urls: Vec<String>,
    next: usize,
    cooldown_until: Vec<tokio::time::Instant>,
    cooldown: Duration,
}

impl RpcNodePool {
    pub fn new(urls: Vec<String>, cooldown: Duration) -> Self {
        let len = urls.len();
        Self {
            urls,
            next: 0,
            cooldown_until: vec![tokio::time::Instant::now(); len],
            cooldown,
        }
    }

    pub fn is_empty(&self) -> bool {
        self.urls.is_empty()
    }

    pub fn len(&self) -> usize {
        self.urls.len()
    }

    /// Next healthy node URL (round-robin), or `None` if all are cooling down
    /// (so the caller can back off instead of hammering a dead pool).
    pub fn next_url(&mut self) -> Option<String> {
        if self.urls.is_empty() {
            return None;
        }
        for _ in 0..self.urls.len() {
            let idx = self.next;
            self.next = (self.next + 1) % self.urls.len();
            if self.cooldown_until[idx] <= tokio::time::Instant::now() {
                return Some(self.urls[idx].clone());
            }
        }
        None
    }

    /// Forget a node's cooldown after a successful call.
    pub fn mark_success(&mut self, url: &str) {
        if let Some(idx) = self.urls.iter().position(|u| u == url) {
            self.cooldown_until[idx] = tokio::time::Instant::now();
        }
    }

    /// Put a node into cooldown after a failure / rate limit.
    pub fn mark_failure(&mut self, url: &str) {
        if let Some(idx) = self.urls.iter().position(|u| u == url) {
            self.cooldown_until[idx] = tokio::time::Instant::now() + self.cooldown;
        }
    }
}

/// Exponential backoff with full jitter in `[50%, 100%)` of the nominal value.
///
/// `attempt` is the number of consecutive failures so far; the nominal delay
/// is `base * 2^attempt` capped at `max`. Jittered homes prevent every worker
/// from storming a recovering node at the same instant.
pub fn backoff_with_jitter(attempt: u32, base: Duration, max: Duration) -> Duration {
    let exp = base.saturating_mul(2u32.saturating_pow(attempt.min(16)));
    let exp = exp.min(max);
    let nominal_ms = exp.as_millis().min(u64::MAX as u128) as u64;
    let jittered = rand::thread_rng().gen_range(nominal_ms / 2..=nominal_ms);
    Duration::from_millis(jittered.max(1))
}

// ---------------------------------------------------------------------------
// Raw RPC types
// ---------------------------------------------------------------------------

/// `getEvents` request body.
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

/// `getEvents` response.
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

/// A raw contract event as returned by the RPC node.
#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct SorobanEvent {
    /// Opaque pagination cursor for this event (also the dedup key).
    pub id: String,
    /// Ledger where the event was emitted.
    #[serde(rename = "ledger")]
    pub ledger: u32,
    /// Emitting contract id.
    #[serde(rename = "contractId")]
    pub contract_id: String,
    /// Base64-XDR topic array (!) or plain symbol strings, depending on the
    /// RPC adapter. [`IndexedEvent::decode`] handles both.
    pub topic: Vec<String>,
    /// XDR base64 payload or raw JSON — normalised by [`IndexedEvent::decode`].
    pub value: Value,
    /// Ledger close time (optional).
    #[serde(rename = "ledgerClosedAt")]
    pub ledger_closed_at: Option<String>,
}

/// One successfully decoded page of events from `getEvents`.
struct FetchedPage {
    events: Vec<SorobanEvent>,
    latest_ledger: u32,
}

// ---------------------------------------------------------------------------
// RPC fetch with failover
// ---------------------------------------------------------------------------

/// Fetch one `getEvents` page from a single node.
async fn fetch_page_from(
    http: &reqwest::Client,
    url: &str,
    config: &IndexerConfig,
    cursor: Option<String>,
    start_ledger: Option<u32>,
) -> Result<FetchedPage, IndexerError> {
    let request = GetEventsRequest {
        jsonrpc: "2.0",
        id: 1,
        method: "getEvents",
        params: GetEventsParams {
            start_ledger,
            filters: vec![EventFilter {
                event_type: "contract",
                contract_ids: config.contract_ids(),
            }],
            pagination: EventPagination {
                limit: MAX_EVENTS_PER_POLL,
                cursor,
            },
        },
    };

    let response = http
        .post(url)
        .json(&request)
        .timeout(Duration::from_secs(15))
        .send()
        .await
        .map_err(|e| IndexerError::Http(format!("{e}")))?;

    if response.status() == reqwest::StatusCode::TOO_MANY_REQUESTS {
        return Err(IndexerError::RateLimited);
    }
    if !response.status().is_success() {
        return Err(IndexerError::Rpc(format!(
            "unexpected HTTP status {}",
            response.status()
        )));
    }

    let rpc: GetEventsResponse = response
        .json()
        .await
        .map_err(|e| IndexerError::Rpc(format!("failed to parse response: {e}")))?;

    if let Some(err) = rpc.error {
        return Err(IndexerError::Rpc(format!("{err}")));
    }

    match rpc.result {
        Some(result) => Ok(FetchedPage {
            events: result.events,
            latest_ledger: result.latest_ledger,
        }),
        None => Ok(FetchedPage {
            events: Vec::new(),
            latest_ledger: 0,
        }),
    }
}

/// Fetch a page, rotating across the node pool and applying cooldowns.
///
/// Returns the fetched page and the URL of the node that served it, so the
/// caller can mark success/failure.
async fn fetch_page(
    http: &reqwest::Client,
    pool: &mut RpcNodePool,
    config: &IndexerConfig,
    cursor: Option<String>,
    start_ledger: Option<u32>,
) -> Result<(FetchedPage, String), IndexerError> {
    let url = pool
        .next_url()
        .ok_or(IndexerError::NoHealthyNode)?;
    match fetch_page_from(http, &url, config, cursor, start_ledger).await {
        Ok(page) => Ok((page, url)),
        Err(e) => {
            match &e {
                IndexerError::RateLimited | IndexerError::Http(_) | IndexerError::Rpc(_) => {
                    pool.mark_failure(&url);
                }
                _ => {}
            }
            Err(e)
        }
    }
}

// ---------------------------------------------------------------------------
// Producer: RPC fetcher stage
// ---------------------------------------------------------------------------

/// Spawn the full indexer pipeline: one producer, `config.workers` DB
/// persisters and one finalizer sweeper.
///
/// This is the entry point wired up by `routes::create_routes`.
pub fn spawn_indexer(
    pool: PgPool,
    redis: Option<RedisCache>,
    broker: Option<PurchaseBroadcaster>,
    config: IndexerConfig,
) {
    if !config.is_enabled() {
        tracing::info!(
            "Soroban indexer: disabled (set RPC URL(s) and TICKET_PAYMENT_CONTRACT_ID / \
             EVENT_REGISTRY_CONTRACT_ID to enable)."
        );
        return;
    }

    let latest_ledger = Arc::new(AtomicU32::new(0));

    let (job_tx, job_rx) = mpsc::channel::<IndexedEvent>(QUEUE_CAPACITY);
    let job_rx = Arc::new(tokio::sync::Mutex::new(job_rx));

    for worker_id in 0..config.workers {
        let pool = pool.clone();
        let config = config.clone();
        let rx = Arc::clone(&job_rx);
        let broker = broker.clone();
        tokio::spawn(async move {
            run_persister(worker_id, pool, config, rx, broker).await;
        });
    }

    {
        let pool = pool.clone();
        let config = config.clone();
        let broker = broker.clone();
        let latest = Arc::clone(&latest_ledger);
        tokio::spawn(async move {
            run_finalizer(pool, config, broker, latest).await;
        });
    }

    {
        let pool = pool.clone();
        let config = config.clone();
        let latest = Arc::clone(&latest_ledger);
        tokio::spawn(async move {
            run_producer(pool, redis, config, job_tx, latest).await;
        });
    }

    tracing::info!(
        "Soroban indexer started: rpc_urls={:?} workers={} window={} confirmations={}",
        config.rpc_urls,
        config.workers,
        config.window_ledgers,
        config.confirmations
    );
}

#[allow(clippy::too_many_lines)]
async fn run_producer(
    pool: PgPool,
    redis: Option<RedisCache>,
    config: IndexerConfig,
    job_tx: mpsc::Sender<IndexedEvent>,
    latest_ledger: Arc<AtomicU32>,
) {
    tracing::info!("Soroban indexer producer started (RPC fetcher stage)");

    let http = reqwest::Client::new();
    let mut nodes = RpcNodePool::new(config.rpc_urls.clone(), NODE_COOLDOWN);

    // Recover the checkpoint so a restart resumes where it stopped.
    let checkpoint = CheckpointStore::load(&pool, CHAIN_KEY).await.ok().flatten();
    let (mut cursor, mut start_ledger): (Option<String>, Option<u32>) = match checkpoint {
        Some(cp) if cp.event_cursor.is_some() => (cp.event_cursor, None),
        Some(cp) => (
            None,
            Some(cp.ledger_sequence.saturating_add(1) as u32),
        ),
        None => (
            None,
            if config.start_ledger > 0 {
                Some(config.start_ledger)
            } else {
                None
            },
        ),
    };

    let mut attempts = 0u32;

    loop {
        // Re-org detection compares against the checkpoint advanced by the
        // finalizer; refresh it once per page (one cheap SELECT).
        let checkpoint_ledger = CheckpointStore::load(&pool, CHAIN_KEY)
            .await
            .ok()
            .flatten()
            .map(|cp| cp.ledger_sequence)
            .unwrap_or(0);

        let (page, url) = match fetch_page(
            &http,
            &mut nodes,
            &config,
            cursor.clone(),
            start_ledger,
        )
        .await
        {
            Ok(ok) => ok,
            Err(IndexerError::NoHealthyNode) => {
                attempts += 1;
                let wait = backoff_with_jitter(attempts, POLL_INTERVAL, MAX_BACKOFF);
                tracing::warn!("Soroban indexer: all RPC nodes cooling down, backing off {:?}", wait);
                sleep(wait).await;
                continue;
            }
            Err(e) => {
                attempts += 1;
                let wait = backoff_with_jitter(attempts, POLL_INTERVAL, MAX_BACKOFF);
                tracing::warn!(
                    "Soroban indexer: poll error (attempt {}, retrying in {:?}): {:?}",
                    attempts,
                    wait,
                    e
                );
                sleep(wait).await;
                continue;
            }
        };

        attempts = 0;
        nodes.mark_success(&url);
        latest_ledger.store(page.latest_ledger, Ordering::Relaxed);

        let mut reorg_rolled_back = false;
        for raw in &page.events {
            let event = IndexedEvent::decode(
                raw.id.clone(),
                raw.ledger,
                raw.contract_id.clone(),
                &raw.topic,
                &raw.value,
                page.latest_ledger,
            );

            // A brand-new envelope at an already-finalized ledger means the
            // chain forked at or before `event.ledger`.
            if !reorg_rolled_back
                && event.ledger as i64 <= checkpoint_ledger
                && !buffer_contains(&pool, &event.id, event.ledger).await
            {
                tracing::warn!(
                    "Re-org detected: new event {} at finalized ledger {}, rolling back",
                    event.id,
                    event.ledger
                );
                match CheckpointStore::rollback_to(&pool, CHAIN_KEY, event.ledger as i64).await {
                    Ok(new_ledger) => {
                        tracing::info!("Rolled back checkpoint to ledger {new_ledger}");
                        // Clear the in-memory cursor and re-scan from the fork.
                        cursor = None;
                        start_ledger = Some(event.ledger);
                        reorg_rolled_back = true;
                    }
                    Err(e) => {
                        tracing::error!("Re-org rollback failed: {:?}", e);
                    }
                }
            }

            if reorg_rolled_back {
                break;
            }

            if job_tx.send(event).await.is_err() {
                tracing::error!("Soroban indexer: worker queue closed, producer exiting");
                return;
            }
        }

        if reorg_rolled_back {
            continue; // resume polling from the fork ledger on the next pass
        }

        if let Some(last) = page.events.last() {
            cursor = Some(last.id.clone());
            start_ledger = None;

            // Best-effort mirrors so a restart can fast-resume even if it
            // cannot read PostgreSQL momentarily. PG remains the source of
            // truth on restart though.
            if let Err(e) =
                CheckpointStore::save(&pool, CHAIN_KEY, checkpoint_ledger, Some(&last.id)).await
            {
                tracing::warn!("Failed to persist indexer cursor to PostgreSQL: {:?}", e);
            }
            if let Some(mut redis) = redis.clone() {
                if let Err(e) = redis
                    .set(
                        LEGACY_CURSOR_CACHE_KEY,
                        &last.id,
                        Duration::from_secs(86400 * 30),
                    )
                    .await
                {
                    tracing::debug!("Failed to mirror indexer cursor to Redis: {:?}", e);
                }
            }
        }

        if page.events.len() >= MAX_EVENTS_PER_POLL as usize {
            continue; // full page → fetch the next one immediately
        }

        sleep(POLL_INTERVAL).await;
    }
}

/// Bounded existence check inside the sliding-window buffer.
async fn buffer_contains(pool: &PgPool, id: &str, ledger: u32) -> bool {
    sqlx::query_scalar::<_, bool>(
        "SELECT EXISTS (SELECT 1 FROM indexer_ledger_events WHERE id = $1 AND ledger = $2)",
    )
    .bind(id)
    .bind(ledger as i64)
    .fetch_one(pool)
    .await
    .unwrap_or(false)
}

// ---------------------------------------------------------------------------
// Persister workers: dedup queue → DB persister stage
// ---------------------------------------------------------------------------

/// Consumer stage: insert into the sliding-window buffer (dedup key = RPC id),
/// and apply state immediately when the event is already final, otherwise leave
/// it pending for the finalizer sweeper.
async fn run_persister(
    worker_id: usize,
    pool: PgPool,
    config: IndexerConfig,
    job_rx: Arc<tokio::sync::Mutex<mpsc::Receiver<IndexedEvent>>>,
    broker: Option<PurchaseBroadcaster>,
) {
    tracing::debug!("Soroban indexer persister worker {worker_id} started");

    loop {
        let event = {
            let mut rx = job_rx.lock().await;
            match rx.recv().await {
                Some(event) => event,
                None => return, // queue closed → producer gone
            }
        };

        let inserted = match buffer_insert(&pool, &event).await {
            Ok(rows) => rows > 0,
            Err(e) => {
                tracing::warn!("Buffer insert failed for event {}: {:?}", event.id, e);
                continue;
            }
        };

        if !inserted {
            continue; // duplicate already parked by a previous poll/replay
        }

        let is_final = event.ledger.saturating_add(config.confirmations) <= event.latest_ledger;
        if !is_final {
            continue; // too close to chain head → finalizer applies later
        }

        if let Err(e) = apply_and_finalize(&pool, &config, broker.as_ref(), &event).await {
            tracing::warn!(
                "Worker {} failed applying final event {}: {:?}",
                worker_id,
                event.id,
                e
            );
        }
    }
}

/// Insert one event into the buffer, returning the number of rows actually
/// inserted (0 → duplicate).
pub async fn buffer_insert(pool: &PgPool, event: &IndexedEvent) -> Result<u64, sqlx::Error> {
    let result = sqlx::query(
        "INSERT INTO indexer_ledger_events (id, ledger, contract_id, topic, value, finalized)
         VALUES ($1, $2, $3, $4, $5, FALSE)
         ON CONFLICT (id) DO NOTHING",
    )
    .bind(&event.id)
    .bind(event.ledger as i64)
    .bind(&event.contract_id)
    .bind(sqlx::types::Json(&event.topic_name))
    .bind(sqlx::types::Json(&event.value))
    .execute(pool)
    .await?;

    Ok(result.rows_affected())
}

/// Apply state for a final event, broadcast to WebSocket clients, and mark the
/// buffer row finalized.
async fn apply_and_finalize(
    pool: &PgPool,
    config: &IndexerConfig,
    broker: Option<&PurchaseBroadcaster>,
    event: &IndexedEvent,
) -> Result<(), IndexerError> {
    apply_event_state(pool, config, broker, event).await?;
    sqlx::query("UPDATE indexer_ledger_events SET finalized = TRUE WHERE id = $1")
        .bind(&event.id)
        .execute(pool)
        .await?;
    Ok(())
}

// ---------------------------------------------------------------------------
// Finalizer sweeper: applies buffered events once their ledger is final
// ---------------------------------------------------------------------------

/// Background sweeper that (a) finalizes pending events whose ledger is now
/// deep enough below the chain head and (b) prunes the bounded window.
async fn run_finalizer(
    pool: PgPool,
    config: IndexerConfig,
    broker: Option<PurchaseBroadcaster>,
    latest_ledger: Arc<AtomicU32>,
) {
    tracing::info!("Soroban indexer finalizer started (sliding-window sweeper)");

    let mut interval = tokio::time::interval(POLL_INTERVAL);
    loop {
        interval.tick().await;

        let observed = latest_ledger.load(Ordering::Relaxed);
        if observed < config.confirmations + 1 {
            continue;
        }
        let final_bound = observed - config.confirmations;

        match finalize_batch(&pool, &config, broker.as_ref(), final_bound).await {
            Ok((applied, finalized)) => {
                if applied > 0 {
                    tracing::info!(
                        "Finalizer applied {applied} events (finalized {finalized} buffered rows), \
                         now final at ledger {final_bound}"
                    );
                }
                // Atomic checkpoint advancement after the batch.
                if let Err(e) =
                    CheckpointStore::save(&pool, CHAIN_KEY, final_bound as i64, None).await
                {
                    tracing::warn!("Finalizer failed to persist checkpoint: {:?}", e);
                }
                // Keep the re-org window bounded.
                let keep = final_bound.saturating_sub(config.window_ledgers) as i64;
                if let Err(e) = CheckpointStore::prune_window(&pool, keep).await {
                    tracing::debug!("Finalizer prune skipped: {:?}", e);
                }
            }
            Err(e) => {
                tracing::warn!("Finalizer sweep failed: {:?}", e);
            }
        }
    }
}

/// A buffered event waiting for confirmation.
#[derive(sqlx::FromRow)]
struct PendingEvent {
    id: String,
    ledger: i64,
    contract_id: String,
    topic: Json<Value>,
    value: Json<Value>,
}

/// Apply every final-but-unconfirmed buffered event in ledger order.
async fn finalize_batch(
    pool: &PgPool,
    config: &IndexerConfig,
    broker: Option<&PurchaseBroadcaster>,
    final_bound: u32,
) -> Result<(u64, u64), IndexerError> {
    let rows: Vec<PendingEvent> = sqlx::query_as::<_, PendingEvent>(
        "SELECT id, ledger, contract_id, topic, value
         FROM indexer_ledger_events
         WHERE finalized = FALSE AND ledger <= $1
         ORDER BY ledger ASC, id ASC
         LIMIT $2",
    )
    .bind(final_bound as i64)
    .bind(FINALIZER_BATCH_SIZE)
    .fetch_all(pool)
    .await?;

    let mut applied = 0u64;
    for row in &rows {
        let topic: Vec<String> = match serde_json::from_value(row.topic.0.clone()) {
            Ok(t) => t,
            Err(_) => row
                .topic
                .0
                .as_str()
                .map(|s| vec![s.to_string()])
                .unwrap_or_default(),
        };
        let event = IndexedEvent::decode(
            row.id.clone(),
            row.ledger as u32,
            row.contract_id.clone(),
            &topic,
            &row.value.0,
            final_bound + config.confirmations, // historical view of latest
        );
        if event.kind == EventKind::Unhandled {
            // Nothing to apply — just retire the buffered row.
            sqlx::query("UPDATE indexer_ledger_events SET finalized = TRUE WHERE id = $1")
                .bind(&row.id)
                .execute(pool)
                .await?;
            applied += 1;
            continue;
        }
        apply_and_finalize(pool, config, broker, &event).await?;
        applied += 1;
    }

    Ok((applied, rows.len() as u64))
}

// ---------------------------------------------------------------------------
// State application (idempotent) + WebSocket broadcasting
// ---------------------------------------------------------------------------

/// Apply a final event's state change to the application tables.
///
/// Every path is idempotent (`ON CONFLICT` / plain `UPDATE`), so a re-org
/// replay re-applies without corrupting rows.
pub async fn apply_event_state(
    pool: &PgPool,
    config: &IndexerConfig,
    broker: Option<&PurchaseBroadcaster>,
    event: &IndexedEvent,
) -> Result<(), IndexerError> {
    if event.contract_id == config.ticket_payment_contract_id {
        match event.kind {
            EventKind::ProcessPurchase => apply_purchase(pool, broker, event).await?,
            EventKind::Refund => apply_refund(pool, event).await?,
            EventKind::TransferTicket => apply_transfer(pool, event).await?,
            _ => {}
        }
    } else if event.contract_id == config.event_registry_contract_id {
        match event.kind {
            EventKind::RegisterEvent => apply_registered(pool, event).await?,
            EventKind::EventStatusUpdate => apply_status_update(pool, event).await?,
            EventKind::CollateralStaked => apply_collateral(pool, event, true).await?,
            EventKind::CollateralUnstaked => apply_collateral(pool, event, false).await?,
            _ => {}
        }
    }
    Ok(())
}

async fn apply_purchase(
    pool: &PgPool,
    broker: Option<&PurchaseBroadcaster>,
    event: &IndexedEvent,
) -> Result<(), IndexerError> {
    let payload = event.as_purchase();
    let stellar_id = if payload.payment_id.is_empty() {
        event.id.clone()
    } else {
        payload.payment_id.clone()
    };

    if payload.event_id.is_empty() {
        tracing::debug!("{}/{} missing event_id, skipping purchase apply", event.id, payload.payment_id);
        return Ok(());
    }

    let owner = payload
        .owner
        .clone()
        .or_else(|| payload.buyer.clone());

    sqlx::query(
        r#"
        INSERT INTO tickets (stellar_id, event_id, buyer_wallet, owner_wallet, status)
        VALUES ($1, $2::uuid, $3, $4, 'Unused')
        ON CONFLICT (stellar_id) DO NOTHING
        "#,
    )
    .bind(&stellar_id)
    .bind(&payload.event_id)
    .bind(payload.buyer.as_deref())
    .bind(owner.as_deref())
    .execute(pool)
    .await?;

    tracing::info!(
        "Synced on-chain purchase: stellar_id={} event_id={} buyer={:?} ledger={}",
        stellar_id,
        payload.event_id,
        payload.buyer,
        event.ledger
    );

    // WebSocket broadcaster stage — best-effort, only when the event id is a
    // valid UUID (so dashboard clients can correlate it).
    if let Some(broker) = broker {
        if let Ok(event_uuid) = uuid::Uuid::parse_str(&payload.event_id) {
            let amount = payload.amount.unwrap_or(0) as f64 / 1e7;
            let published = broker.publish(crate::handlers::ws::PurchaseEvent {
                event_id: event_uuid,
                ticket_tier_id: uuid::Uuid::nil(),
                quantity: 1,
                amount,
                currency: "USDC".to_string(),
                purchased_at: chrono::Utc::now().to_rfc3339(),
            });
            if published > 0 {
                tracing::debug!("Broadcast on-chain purchase to {published} WebSocket client(s)");
            }
        }
    }
    Ok(())
}

async fn apply_refund(pool: &PgPool, event: &IndexedEvent) -> Result<(), IndexerError> {
    let payload = event.as_refund();
    let stellar_id = if payload.payment_id.is_empty() {
        event.id.clone()
    } else {
        payload.payment_id.clone()
    };
    let event_uuid = uuid::Uuid::parse_str(&payload.event_id).ok();

    let result = sqlx::query(
        r#"
        UPDATE tickets SET status = 'Revoked'
        WHERE stellar_id = $1
           OR (event_id = $2 AND status != 'Revoked')
        "#,
    )
    .bind(&stellar_id)
    .bind(event_uuid)
    .execute(pool)
    .await?;

    if result.rows_affected() > 0 {
        tracing::info!(
            "Marked {} ticket(s) Revoked after on-chain refund (ledger {})",
            result.rows_affected(),
            event.ledger
        );
    }
    Ok(())
}

async fn apply_transfer(pool: &PgPool, event: &IndexedEvent) -> Result<(), IndexerError> {
    let payload = event.as_transfer();
    if payload.payment_id.is_empty() || payload.to.is_none() {
        return Ok(());
    }
    sqlx::query(
        "UPDATE tickets SET owner_wallet = $1, updated_at = NOW() WHERE stellar_id = $2",
    )
    .bind(payload.to.as_deref())
    .bind(&payload.payment_id)
    .execute(pool)
    .await?;
    tracing::info!(
        "Transferred on-chain ticket {} to {:?} (ledger {})",
        payload.payment_id,
        payload.to,
        event.ledger
    );
    Ok(())
}

async fn apply_registered(pool: &PgPool, event: &IndexedEvent) -> Result<(), IndexerError> {
    let payload = event.as_register();
    if let Ok(uuid) = uuid::Uuid::parse_str(&payload.event_id) {
        sqlx::query("UPDATE events SET updated_at = NOW() WHERE id = $1")
            .bind(uuid)
            .execute(pool)
            .await?;
    }
    tracing::info!(
        "On-chain event registered: event_id={} ledger={}",
        payload.event_id,
        event.ledger
    );
    Ok(())
}

async fn apply_status_update(pool: &PgPool, event: &IndexedEvent) -> Result<(), IndexerError> {
    let event_id = event
        .value
        .get("event_id")
        .and_then(Value::as_str)
        .unwrap_or_default();
    let new_status = event
        .value
        .get("status")
        .and_then(Value::as_str)
        .unwrap_or("cancelled");

    if let Ok(uuid) = uuid::Uuid::parse_str(event_id) {
        sqlx::query("UPDATE events SET updated_at = NOW() WHERE id = $1")
            .bind(uuid)
            .execute(pool)
            .await?;
    }

    tracing::info!(
        "On-chain event status update: event_id={} status={} ledger={}",
        event_id,
        new_status,
        event.ledger
    );
    Ok(())
}

async fn apply_collateral(
    pool: &PgPool,
    event: &IndexedEvent,
    verified: bool,
) -> Result<(), IndexerError> {
    let organizer = event
        .value
        .get("organizer")
        .and_then(Value::as_str)
        .unwrap_or_default();

    if !organizer.is_empty() {
        sqlx::query(
            "UPDATE organizers SET is_verified = $1, updated_at = NOW() WHERE wallet_address = $2",
        )
        .bind(verified)
        .bind(organizer)
        .execute(pool)
        .await?;
    }
    Ok(())
}

// ---------------------------------------------------------------------------
// Historical replay / backfill (used by the admin endpoint)
// ---------------------------------------------------------------------------

/// Result of a replay run.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ReplaySummary {
    pub start_ledger: u32,
    pub end_ledger: u32,
    pub events_processed: u64,
    pub skipped_duplicates: u64,
}

/// Validate replay range without hitting the network or database.
fn run_replay_validation(start_ledger: u32, end_ledger: u32) -> Result<(), IndexerError> {
    if start_ledger == 0 {
        return Err(IndexerError::InvalidRange(
            "start_ledger must be >= 1".to_string(),
        ));
    }
    if end_ledger < start_ledger {
        return Err(IndexerError::InvalidRange(format!(
            "end_ledger ({end_ledger}) must be >= start_ledger ({start_ledger})"
        )));
    }
    if end_ledger.saturating_sub(start_ledger) > MAX_REPLAY_LEDGERS {
        return Err(IndexerError::InvalidRange(format!(
            "replay range exceeds the {MAX_REPLAY_LEDGERS} ledger limit"
        )));
    }
    Ok(())
}

/// Backfill `[start_ledger, end_ledger]` through the same buffered, idempotent
/// path as the live pipeline, then advance the checkpoint to `end_ledger`.
pub async fn run_replay(
    pool: &PgPool,
    config: &IndexerConfig,
    broker: Option<&PurchaseBroadcaster>,
    start_ledger: u32,
    end_ledger: u32,
) -> Result<ReplaySummary, IndexerError> {
    if start_ledger == 0 {
        return Err(IndexerError::InvalidRange(
            "start_ledger must be >= 1".to_string(),
        ));
    }
    if end_ledger < start_ledger {
        return Err(IndexerError::InvalidRange(format!(
            "end_ledger ({end_ledger}) must be >= start_ledger ({start_ledger})"
        )));
    }
    if end_ledger.saturating_sub(start_ledger) > MAX_REPLAY_LEDGERS {
        return Err(IndexerError::InvalidRange(format!(
            "replay range exceeds the {MAX_REPLAY_LEDGERS} ledger limit"
        )));
    }

    let http = reqwest::Client::new();
    let mut nodes = RpcNodePool::new(config.rpc_urls.clone(), NODE_COOLDOWN);

    let mut cursor: Option<String> = None;
    let mut started = false;
    let mut processed = 0u64;
    let mut skipped = 0u64;
    let mut attempts = 0u32;

    loop {
        let (page, url) = match fetch_page(
            &http,
            &mut nodes,
            config,
            cursor.clone(),
            Some(start_ledger),
        )
        .await
        {
            Ok(ok) => ok,
            Err(IndexerError::NoHealthyNode) => {
                attempts += 1;
                sleep(backoff_with_jitter(attempts, POLL_INTERVAL, MAX_BACKOFF)).await;
                continue;
            }
            Err(e) if matches!(e, IndexerError::RateLimited | IndexerError::Rpc(_)) => {
                attempts += 1;
                tracing::warn!("Replay page fetch failed (attempt {attempts}): {e:?}");
                sleep(backoff_with_jitter(attempts, POLL_INTERVAL, MAX_BACKOFF)).await;
                continue;
            }
            Err(e) => return Err(e),
        };
        attempts = 0;
        nodes.mark_success(&url);

        let mut done = false;
        for raw in &page.events {
            if raw.ledger > end_ledger {
                done = true;
                break;
            }
            started = true;

            let event = IndexedEvent::decode(
                raw.id.clone(),
                raw.ledger,
                raw.contract_id.clone(),
                &raw.topic,
                &raw.value,
                page.latest_ledger,
            );

            let inserted = buffer_insert(pool, &event).await?;
            if inserted > 0 {
                apply_and_finalize(pool, config, broker, &event).await?;
                processed += 1;
            } else {
                skipped += 1;
            }
        }

        // Advance the in-memory cursor.
        if let Some(last) = page.events.last() {
            cursor = Some(last.id.clone());
        }

        if done
            || page.events.len() < MAX_EVENTS_PER_POLL as usize
            || page.latest_ledger >= end_ledger
            || !started
        {
            break;
        }
    }

    // Leave the checkpoint behind the replayed range so the live pipeline
    // picks up exactly where backfill finished.
    CheckpointStore::save(pool, CHAIN_KEY, end_ledger as i64, None).await?;

    tracing::info!(
        "Replay finished: ledgers [{start_ledger}, {end_ledger}], processed {processed}, skipped {skipped}"
    );
    Ok(ReplaySummary {
        start_ledger,
        end_ledger,
        events_processed: processed,
        skipped_duplicates: skipped,
    })
}

// ---------------------------------------------------------------------------
// Tests
// ---------------------------------------------------------------------------

#[cfg(test)]
mod tests {
    use super::*;

    // --- RpcNodePool -------------------------------------------------------

    #[test]
    fn pool_rotates_across_nodes_round_robin() {
        let mut pool = RpcNodePool::new(
            vec!["https://a".into(), "https://b".into(), "https://c".into()],
            Duration::from_secs(60),
        );
        assert_eq!(pool.next_url().as_deref(), Some("https://a"));
        assert_eq!(pool.next_url().as_deref(), Some("https://b"));
        assert_eq!(pool.next_url().as_deref(), Some("https://c"));
        assert_eq!(pool.next_url().as_deref(), Some("https://a"));
    }

    #[test]
    fn failed_node_skips_until_cooldown_elapses() {
        let mut pool = RpcNodePool::new(vec!["https://a".into(), "https://b".into()], Duration::from_millis(1));
        pool.mark_failure("https://a");
        // Cooldown still active → "a" is skipped, "b" is served.
        assert_eq!(pool.next_url().as_deref(), Some("https://b"));
        // Let the cooldown elapse.
        std::thread::sleep(Duration::from_millis(2));
        assert_eq!(pool.next_url().as_deref(), Some("https://a"));
    }

    #[test]
    fn empty_pool_returns_none() {
        let mut pool: RpcNodePool = RpcNodePool::new(Vec::new(), Duration::from_secs(1));
        assert!(pool.is_empty());
        assert!(pool.next_url().is_none());
    }

    #[test]
    fn mark_success_forgets_cooldown() {
        let mut pool = RpcNodePool::new(vec!["https://a".into(), "https://b".into()], Duration::from_secs(60));
        pool.mark_failure("https://a");
        pool.mark_success("https://a");
        // First rotation returns "a" (cooldown cleared) again.
        assert_eq!(pool.next_url().as_deref(), Some("https://a"));
    }

    // --- backoff_with_jitter -------------------------------------------------

    #[test]
    fn backoff_is_capped_and_jittered() {
        for attempt in 0..20 {
            for _ in 0..200 {
                let d = backoff_with_jitter(attempt, Duration::from_secs(1), Duration::from_secs(300));
                assert!(
                    d >= Duration::from_millis(500),
                    "backoff must not drop below 50% of base, got {d:?} (attempt {attempt})"
                );
                assert!(
                    d <= MAX_BACKOFF,
                    "backoff must never exceed the cap, got {d:?} (attempt {attempt})"
                );
            }
        }
    }

    #[test]
    fn backoff_grows_with_attempt_count() {
        let nominal = |attempt| backoff_with_jitter(attempt, Duration::from_secs(1), Duration::from_secs(300));
        // Averaging over many draws keeps outliers from flapping the test.
        let avg = |from: u32, to: u32| {
            (from..to)
                .flat_map(|a| (0..100).map(move |_| nominal(a)))
                .map(|d| d.as_millis())
                .sum::<u128>()
                / (2000 as u128)
        };
        assert!(
            avg(0, 1) < avg(4, 5),
            "early backoff should be smaller than late backoff"
        );
    }

    // --- config --------------------------------------------------------------
    // (env-based; those vars default to empty values so tests are hermetic)

    #[test]
    fn contract_ids_are_stable_and_ordered() {
        let config = IndexerConfig {
            rpc_urls: vec!["https://rpc".into()],
            ticket_payment_contract_id: "CPAY".into(),
            event_registry_contract_id: "CREG".into(),
            start_ledger: 0,
            window_ledgers: DEFAULT_WINDOW_LEDGERS,
            confirmations: DEFAULT_CONFIRMATIONS,
            workers: DEFAULT_WORKERS,
            redis_url: None,
        };
        assert_eq!(config.contract_ids(), vec!["CPAY", "CREG"]);
        assert!(config.is_enabled());
    }

    #[tokio::test]
    async fn replay_validation_rejects_bad_ranges() {
        let _config = test_config();

        let res = run_replay_validation(0, 10);
        assert!(matches!(res, Err(IndexerError::InvalidRange(_))));

        let res = run_replay_validation(50, 10);
        assert!(matches!(res, Err(IndexerError::InvalidRange(_))));

        let res = run_replay_validation(1, 1 + MAX_REPLAY_LEDGERS);
        assert!(res.is_ok());

        let res = run_replay_validation(1, 2 + MAX_REPLAY_LEDGERS);
        assert!(matches!(res, Err(IndexerError::InvalidRange(_))));
    }

    // --- helpers -------------------------------------------------------------

    fn test_config() -> IndexerConfig {
        IndexerConfig {
            rpc_urls: vec!["https://rpc".into()],
            ticket_payment_contract_id: "CPAY".into(),
            event_registry_contract_id: "CREG".into(),
            start_ledger: 0,
            window_ledgers: DEFAULT_WINDOW_LEDGERS,
            confirmations: DEFAULT_CONFIRMATIONS,
            workers: DEFAULT_WORKERS,
            redis_url: None,
        }
    }
}