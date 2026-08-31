//! # Webhook Dispatcher (Issue #1339)
//!
//! Fires signed HTTP POST payloads to organiser-registered endpoints whenever
//! key events occur (`PaymentProcessed`, `EventCreated`, `TicketScanned`).
//!
//! - Payload envelope: `{ id, event, timestamp, data }`
//! - Signature: HMAC-SHA256 over the raw JSON body using the endpoint secret,
//!   delivered in the `X-Agora-Signature: sha256=<hex>` header.
//! - Retries: 3 attempts with exponential backoff (1s, 2s, 4s).
//! - Every attempt is recorded in `webhook_delivery_logs` for debugging.

use chrono::Utc;
use hmac::{Hmac, Mac};
use sha2::Sha256;
use sqlx::PgPool;
use uuid::Uuid;

use crate::models::webhook::WebhookEndpoint;

type HmacSha256 = Hmac<Sha256>;

/// Number of delivery attempts per endpoint (including the first try).
const MAX_ATTEMPTS: u32 = 3;

/// Base backoff delay in seconds; doubled each retry.
const BASE_BACKOFF_SECS: u64 = 1;

/// Default delivery timeout per attempt.
const DISPATCH_TIMEOUT_SECS: u64 = 5;

/// Compute `sha256=<hex>` for the given body and secret.
pub fn sign_payload(secret: &str, body: &[u8]) -> String {
    let mut mac =
        HmacSha256::new_from_slice(secret.as_bytes()).expect("HMAC accepts any key length");
    mac.update(body);
    format!("sha256={}", hex::encode(mac.finalize().into_bytes()))
}

/// Resolve the owning organiser for an event (used by dispatch call sites).
pub async fn organizer_for_event(pool: &PgPool, event_id: Uuid) -> Option<Uuid> {
    sqlx::query_scalar::<_, Uuid>("SELECT organizer_id FROM events WHERE id = $1")
        .bind(event_id)
        .fetch_optional(pool)
        .await
        .ok()
        .flatten()
}

/// Fetch the active endpoints subscribed to `event` for an organiser.
async fn endpoints_for(pool: &PgPool, organizer_id: Uuid, event: &str) -> Vec<WebhookEndpoint> {
    sqlx::query_as::<_, WebhookEndpoint>(
        r#"
        SELECT * FROM webhook_endpoints
        WHERE organizer_id = $1
          AND is_active = TRUE
          AND $2 = ANY(events)
        "#,
    )
    .bind(organizer_id)
    .bind(event)
    .fetch_all(pool)
    .await
    .unwrap_or_else(|e| {
        tracing::warn!(
            "webhook dispatch: failed to load endpoints for organizer {}: {:?}",
            organizer_id,
            e
        );
        Vec::new()
    })
}

async fn log_delivery(
    pool: &PgPool,
    endpoint_id: Uuid,
    event: &str,
    payload: &serde_json::Value,
    attempt: u32,
    status_code: Option<i32>,
    error: Option<&str>,
) {
    if let Err(e) = sqlx::query(
        r#"
        INSERT INTO webhook_delivery_logs
            (endpoint_id, event, payload, attempt, status_code, error, delivered_at)
        VALUES ($1, $2, $3, $4, $5, $6, CASE WHEN $5 IS NOT NULL THEN NOW() ELSE NULL END)
        "#,
    )
    .bind(endpoint_id)
    .bind(event)
    .bind(payload)
    .bind(attempt as i32)
    .bind(status_code)
    .bind(error)
    .execute(pool)
    .await
    {
        tracing::warn!("webhook dispatch: failed to write delivery log: {:?}", e);
    }
}

/// Deliver `payload` to one endpoint with retries and backoff.
async fn deliver_to_endpoint(
    client: &reqwest::Client,
    pool: &PgPool,
    endpoint: &WebhookEndpoint,
    event: &str,
    payload: serde_json::Value,
) {
    // Bound the request body so huge payloads surface early rather than timing out.
    let body = serde_json::to_vec(&payload).unwrap_or_else(|_| b"{}".to_vec());
    let signature = sign_payload(&endpoint.secret, &body);

    for attempt in 1..=MAX_ATTEMPTS {
        let outcome = tokio::time::timeout(
            std::time::Duration::from_secs(DISPATCH_TIMEOUT_SECS),
            client
                .post(&endpoint.url)
                .header("Content-Type", "application/json")
                .header("X-Agora-Signature", &signature)
                .header("X-Agora-Event", event)
                .body(body.clone())
                .send(),
        )
        .await;

        match outcome {
            Ok(Ok(response)) => {
                let status = response.status().as_u16() as i32;
                log_delivery(
                    pool,
                    endpoint.id,
                    event,
                    &payload,
                    attempt,
                    Some(status),
                    None,
                )
                .await;

                // 2xx (and 3xx redirects followed by reqwest) count as delivered;
                // anything else retries.
                if response.status().is_success() {
                    tracing::info!(
                        endpoint_id = %endpoint.id,
                        event,
                        status,
                        attempt,
                        "Webhook delivered"
                    );
                    return;
                }
                tracing::warn!(
                    endpoint_id = %endpoint.id,
                    status,
                    attempt,
                    "Webhook returned non-2xx, will retry"
                );
            }
            Ok(Err(e)) => {
                log_delivery(
                    pool,
                    endpoint.id,
                    event,
                    &payload,
                    attempt,
                    None,
                    Some(&e.to_string()),
                )
                .await;
                tracing::warn!(
                    endpoint_id = %endpoint.id,
                    attempt,
                    error = %e,
                    "Webhook request failed, will retry"
                );
            }
            Err(_) => {
                log_delivery(
                    pool,
                    endpoint.id,
                    event,
                    &payload,
                    attempt,
                    None,
                    Some("delivery timed out"),
                )
                .await;
                tracing::warn!(
                    endpoint_id = %endpoint.id,
                    attempt,
                    "Webhook request timed out, will retry"
                );
            }
        }

        if attempt < MAX_ATTEMPTS {
            let backoff = BASE_BACKOFF_SECS.saturating_mul(2u64.pow(attempt - 1));
            tokio::time::sleep(std::time::Duration::from_secs(backoff)).await;
        }
    }
}

/// Enqueue delivery of `event` to every matching endpoint for an organiser.
///
/// Spawns a background task per event so callers never block on outbound HTTP;
/// delivery is retried (3 attempts, exponential backoff) entirely in the task.
pub fn dispatch_webhooks(pool: PgPool, organizer_id: Uuid, event: &str, data: serde_json::Value) {
    let client = reqwest::Client::new();
    let event = event.to_string();
    tokio::spawn(async move {
        let payload = serde_json::json!({
            "id": Uuid::new_v4(),
            "event": event,
            "timestamp": Utc::now().to_rfc3339(),
            "data": data,
        });

        let endpoints = endpoints_for(&pool, organizer_id, &event).await;
        if endpoints.is_empty() {
            tracing::debug!(
                organizer_id = %organizer_id,
                event,
                "No webhook endpoints subscribed"
            );
            return;
        }

        for endpoint in endpoints {
            deliver_to_endpoint(&client, &pool, &endpoint, &event, payload.clone()).await;
        }
    });
}
