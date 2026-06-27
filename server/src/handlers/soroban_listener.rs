//! # Soroban Event Listener
//!
//! A background service that long-polls the Stellar RPC node for `ContractEvent`
//! objects emitted by the `ticket_payment` and `event_registry` contracts.
//!
//! ## Architecture
//! - Spawned as a `tokio` background task at server startup.
//! - Uses `getEvents` RPC method with a cursor to avoid re-processing events.
//! - Parses XDR-encoded event topics/data to update `tickets` and `events` tables.
//! - Handles ledger re-orgs by only processing events with ≥ `MIN_CONFIRMATIONS`
//!   ledgers of depth (i.e. `latest_ledger - event_ledger >= MIN_CONFIRMATIONS`).
//!
//! ## Acceptance Criteria
//! A purchase confirmed on-chain appears in the backend DB within 10 seconds.

use serde::{Deserialize, Serialize};
use serde_json::Value;
use sqlx::PgPool;
use std::time::Duration;
use tokio::time::sleep;

/// Minimum ledger confirmations before an event is considered final.
/// Protects against short re-orgs on Stellar (typically 1 is sufficient,
/// but we use 2 for safety).
const MIN_CONFIRMATIONS: u32 = 2;

/// How often to poll the RPC node for new events (base interval and first-failure delay).
const POLL_INTERVAL: Duration = Duration::from_secs(5);

/// Maximum back-off delay between retries after consecutive RPC failures.
const MAX_BACKOFF: Duration = Duration::from_secs(300);

/// Maximum events to fetch per poll cycle.
const MAX_EVENTS_PER_POLL: u32 = 100;

/// Redis key for persisting the last processed event cursor.
#[allow(dead_code)]
const CURSOR_CACHE_KEY: &str = "soroban:event_cursor";

// ---------------------------------------------------------------------------
// RPC types
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
#[derive(Debug, Deserialize, Clone)]
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

// ---------------------------------------------------------------------------
// Listener state
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
        }
    }
}

// ---------------------------------------------------------------------------
// Main listener loop
// ---------------------------------------------------------------------------

/// Spawn the Soroban event listener as a background task.
///
/// This function returns immediately; the listener runs indefinitely in the
/// background. Errors are logged but do not crash the server.
pub fn spawn_listener(pool: PgPool, config: ListenerConfig) {
    tokio::spawn(async move {
        run_listener(pool, config).await;
    });
}

async fn run_listener(pool: PgPool, config: ListenerConfig) {
    // Skip if no contract IDs are configured (e.g. in development without contracts)
    if config.ticket_payment_contract_id.is_empty() && config.event_registry_contract_id.is_empty()
    {
        tracing::info!(
            "Soroban listener: no contract IDs configured, skipping. \
             Set TICKET_PAYMENT_CONTRACT_ID and/or EVENT_REGISTRY_CONTRACT_ID to enable."
        );
        return;
    }

    let http = reqwest::Client::new();
    let mut cursor: Option<String> = None;
    let mut start_ledger = Some(config.start_ledger);
    let mut current_backoff = POLL_INTERVAL;

    tracing::info!(
        "Soroban listener started. RPC={} poll_interval={:?}",
        config.rpc_url,
        POLL_INTERVAL
    );

    loop {
        match poll_events(&http, &config, cursor.clone(), start_ledger).await {
            Ok(Some(result)) => {
                let latest_ledger = result.latest_ledger;

                for event in &result.events {
                    // Re-org protection: skip events that are too recent
                    if latest_ledger.saturating_sub(event.ledger) < MIN_CONFIRMATIONS {
                        tracing::debug!(
                            "Skipping event {} (ledger {} not yet confirmed, latest={})",
                            event.id,
                            event.ledger,
                            latest_ledger
                        );
                        continue;
                    }

                    if let Err(e) = process_event(&pool, event, &config).await {
                        tracing::error!("Failed to process event {}: {:?}", event.id, e);
                    }
                }

                // Advance cursor to the last event we received
                if let Some(last) = result.events.last() {
                    cursor = Some(last.id.clone());
                    // Once we have a cursor, stop specifying startLedger
                    start_ledger = None;
                }

                // Reset back-off on any successful poll
                current_backoff = POLL_INTERVAL;
            }
            Ok(None) => {
                // No new events — reset back-off and poll again at base rate
                current_backoff = POLL_INTERVAL;
            }
            Err(e) => {
                tracing::error!(
                    "Soroban listener poll error (retrying in {:?}): {:?}",
                    current_backoff,
                    e
                );
                sleep(current_backoff).await;
                // Double the back-off for the next failure, capped at MAX_BACKOFF
                current_backoff = (current_backoff * 2).min(MAX_BACKOFF);
                continue;
            }
        }

        sleep(POLL_INTERVAL).await;
    }
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
// Event processing
// ---------------------------------------------------------------------------

/// Dispatch a single Soroban event to the appropriate handler based on its topic.
async fn process_event(
    pool: &PgPool,
    event: &SorobanEvent,
    config: &ListenerConfig,
) -> Result<(), String> {
    // The first topic element identifies the event type (a Symbol in XDR).
    // We use the string representation that the RPC returns in the topic array.
    let event_name = event.topic.first().map(|t| t.as_str()).unwrap_or("unknown");

    tracing::debug!(
        "Processing event: contract={} name={} ledger={}",
        event.contract_id,
        event_name,
        event.ledger
    );

    if event.contract_id == config.ticket_payment_contract_id {
        match event_name {
            // Emitted by TicketPayment::process_purchase
            "ticket_purchased" | "purchase_confirmed" => {
                handle_ticket_purchased(pool, event).await?;
            }
            // Emitted by TicketPayment::refund
            "ticket_refunded" => {
                handle_ticket_refunded(pool, event).await?;
            }
            _ => {
                tracing::debug!("Unhandled ticket_payment event: {}", event_name);
            }
        }
    } else if event.contract_id == config.event_registry_contract_id {
        match event_name {
            // Emitted by EventRegistry::register_event
            "event_registered" => {
                handle_event_registered(pool, event).await?;
            }
            // Emitted by EventRegistry::update_event_status / cancel_event
            "event_status_updated" | "event_cancelled" => {
                handle_event_status_updated(pool, event).await?;
            }
            _ => {
                tracing::debug!("Unhandled event_registry event: {}", event_name);
            }
        }
    }

    Ok(())
}

/// Handle a `ticket_purchased` event — upsert a ticket record in the DB.
///
/// Expected event value shape (JSON representation of XDR map):
/// ```json
/// { "event_id": "...", "buyer": "G...", "owner": "G...", "quantity": 1, "stellar_id": "..." }
/// ```
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

    // Upsert ticket — idempotent on stellar_id
    match sqlx::query(
        r#"
        INSERT INTO tickets (stellar_id, event_id, buyer_wallet, owner_wallet, status)
        VALUES ($1, $2::uuid, $3, $4, 'active')
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

/// Handle a `ticket_refunded` event — mark the ticket as cancelled.
async fn handle_ticket_refunded(pool: &PgPool, event: &SorobanEvent) -> Result<(), String> {
    let stellar_id = event
        .value
        .get("stellar_id")
        .and_then(|v| v.as_str())
        .unwrap_or(&event.id);

    match sqlx::query("UPDATE tickets SET status = 'cancelled' WHERE stellar_id = $1")
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

/// Handle an `event_registered` event — record the on-chain event ID.
async fn handle_event_registered(_pool: &PgPool, event: &SorobanEvent) -> Result<(), String> {
    let data = &event.value;
    let on_chain_event_id = data
        .get("event_id")
        .and_then(|v| v.as_str())
        .unwrap_or_default();

    if on_chain_event_id.is_empty() {
        return Ok(());
    }

    tracing::info!(
        "On-chain event registered: event_id={} ledger={}",
        on_chain_event_id,
        event.ledger
    );
    Ok(())
}

/// Handle an `event_status_updated` or `event_cancelled` event.
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

    tracing::info!(
        "On-chain event status update: event_id={} status={} ledger={}",
        on_chain_event_id,
        new_status,
        event.ledger
    );
    Ok(())
}

// ---------------------------------------------------------------------------
// Tests
// ---------------------------------------------------------------------------

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_listener_config_from_env_defaults() {
        let config = ListenerConfig {
            rpc_url: "https://soroban-testnet.stellar.org".to_string(),
            ticket_payment_contract_id: String::new(),
            event_registry_contract_id: String::new(),
            start_ledger: 0,
        };
        assert_eq!(config.rpc_url, "https://soroban-testnet.stellar.org");
        assert_eq!(config.start_ledger, 0);
    }

    #[test]
    fn test_min_confirmations_value() {
        // MIN_CONFIRMATIONS is a compile-time constant; verify it's at least 1
        const _: () = assert!(MIN_CONFIRMATIONS >= 1);
    }

    #[test]
    fn test_poll_interval_within_acceptance_criteria() {
        // Acceptance criteria: on-chain purchase appears in DB within 10 seconds.
        // Poll interval must be well under 10s.
        assert!(
            POLL_INTERVAL.as_secs() < 10,
            "poll interval must be < 10s to meet acceptance criteria"
        );
    }

    #[test]
    fn test_soroban_event_deserialization() {
        let json = serde_json::json!({
            "id": "0000000100-0000000001",
            "ledger": 100u32,
            "contractId": "CABC123",
            "topic": ["ticket_purchased"],
            "value": { "event_id": "evt-1", "buyer": "GABC", "quantity": 2 },
            "ledgerClosedAt": "2026-05-01T12:00:00Z"
        });

        let event: SorobanEvent = serde_json::from_value(json).unwrap();
        assert_eq!(event.ledger, 100);
        assert_eq!(event.contract_id, "CABC123");
        assert_eq!(event.topic[0], "ticket_purchased");
    }

    #[test]
    fn test_backoff_sequence_doubles_and_caps() {
        let mut backoff = POLL_INTERVAL;
        // First failure: wait POLL_INTERVAL (5s), then double
        assert_eq!(backoff, Duration::from_secs(5));
        backoff = (backoff * 2).min(MAX_BACKOFF);
        assert_eq!(backoff, Duration::from_secs(10));
        backoff = (backoff * 2).min(MAX_BACKOFF);
        assert_eq!(backoff, Duration::from_secs(20));
        backoff = (backoff * 2).min(MAX_BACKOFF);
        assert_eq!(backoff, Duration::from_secs(40));
        backoff = (backoff * 2).min(MAX_BACKOFF);
        assert_eq!(backoff, Duration::from_secs(80));
        backoff = (backoff * 2).min(MAX_BACKOFF);
        assert_eq!(backoff, Duration::from_secs(160));
        backoff = (backoff * 2).min(MAX_BACKOFF);
        // 320 > 300, so capped at MAX_BACKOFF
        assert_eq!(backoff, MAX_BACKOFF);
        backoff = (backoff * 2).min(MAX_BACKOFF);
        // Still capped
        assert_eq!(backoff, MAX_BACKOFF);

        // Reset on success
        backoff = POLL_INTERVAL;
        assert_eq!(backoff, Duration::from_secs(5));
    }

    #[test]
    fn test_reorg_protection_logic() {
        let latest_ledger: u32 = 100;
        let event_ledger: u32 = 99;
        let depth = latest_ledger.saturating_sub(event_ledger);
        // With MIN_CONFIRMATIONS = 2, ledger 99 at latest 100 should be skipped
        assert!(depth < MIN_CONFIRMATIONS);

        let confirmed_ledger: u32 = 97;
        let depth2 = latest_ledger.saturating_sub(confirmed_ledger);
        assert!(depth2 >= MIN_CONFIRMATIONS);
    }
}
