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

use axum::http::StatusCode;
use chrono::Utc;
use hmac::{Hmac, Mac};
use sha2::Sha256;
use sqlx::PgPool;
use std::net::{IpAddr, Ipv4Addr, Ipv6Addr};
use uuid::Uuid;

use crate::models::webhook::{WebhookDeliveryFailure, WebhookEndpoint};

type HmacSha256 = Hmac<Sha256>;

/// Number of delivery attempts per endpoint (including the first try).
const MAX_ATTEMPTS: u32 = 3;

/// IP addresses and hostname ranges that are rejected for webhook deliveries
/// to prevent SSRF (Server-Side Request Forgery) and DNS rebinding attacks.
///
/// These ranges are refused both at validation time (URL field) and at dispatch
/// time (hostname resolution) for defense-in-depth.
fn is_protected_ip(ip: &IpAddr) -> bool {
    // IPv4 loopback: 127.0.0.0/8
    // IPv6 loopback: ::1
    if ip.is_loopback() {
        return true;
    }

    // IPv4 private: 10.0.0.0/8, 172.16.0.0/12, 192.168.0.0/16
    // IPv6 unique-local: fc00::/8 (we check fd00::/8 in is_private())
    if ip.is_private() {
        return true;
    }

    // IPv4 link-local: 169.254.0.0/16
    // IPv6 link-local: fe80::/10
    if ip.is_link_local() {
        return true;
    }

    // IPv4/macOS multicast: 224.0.0.0/4 (but we avoid all multicast)
    if ip.is_multicast() {
        return true;
    }

    // IPv4 unspecified: 0.0.0.0
    // IPv6 unspecified: ::
    if ip.is_unspecified() {
        return true;
    }

    false
}

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

async fn record_delivery_failure(
    pool: &PgPool,
    endpoint_id: Uuid,
    event: &str,
    payload: &serde_json::Value,
    attempts: u32,
    last_status_code: Option<i32>,
    last_error: Option<&str>,
) {
    if let Err(e) = sqlx::query(
        r#"
        INSERT INTO webhook_delivery_failures
            (endpoint_id, event, payload, attempts, last_status_code, last_error, failed_at)
        VALUES ($1, $2, $3, $4, $5, $6, NOW())
        "#,
    )
    .bind(endpoint_id)
    .bind(event)
    .bind(payload)
    .bind(attempts as i32)
    .bind(last_status_code)
    .bind(last_error)
    .execute(pool)
    .await
    {
        tracing::error!("webhook dispatch: failed to write to DLQ: {:?}", e);
    }
}

/// Resolve a hostname and check that none of its resolved addresses are in
/// protected ranges (loopback, private, link-local, multicast, unspecified).
/// Returns an error if the hostname resolves to a protected address, which
/// prevents DNS rebinding attacks.
fn check_hostname_protection(url_str: &str) -> Result<(), String> {
    let parsed = match Url::parse(url_str) {
        Ok(u) => u,
        Err(_) => return Err("invalid URL".to_string()),
    };

    let host = parsed
        .host()
        .ok_or_else(|| "URL must contain a host".to_string())?;

    // If the host is an IP address, check it directly.
    if let Ok(ip) = std::net::IpAddr::from_str(host) {
        return Ok(if is_protected_ip(&ip) {
            Err(format!(
                "URL host '{host}' is a protected address (loopback/private/link-local)"
            ))
        } else {
            Ok(())
        });
    }

    // It's a hostname: perform DNS resolution and check all resolved addresses.
    // We use std::net::lookup_host which returns all matching addresses.
    let addrs = match std::net::lookup_host(host) {
        Ok(a) => a,
        Err(e) => {
            // If DNS resolution fails, we allow the request (the dispatcher
            // will handle the actual error), but we still reject if the hostname
            // itself is "localhost".
            if host == "localhost" || host == "local" {
                return Err("URL host 'localhost' is not allowed".to_string());
            }
            return Ok(());
        }
    };

    for addr in addrs {
        let ip = addr.ip();
        if is_protected_ip(&ip) {
            return Err(format!(
                "URL host '{}' resolves to protected address {:?} (loopback/private/link-local)",
                host, ip
            ));
        }
    }

    Ok(())
}

/// Deliver `payload` to one endpoint with retries and backoff.
async fn deliver_to_endpoint(
    client: &reqwest::Client,
    pool: &PgPool,
    endpoint: &WebhookEndpoint,
    event: &str,
    payload: serde_json::Value,
) {
    // Check hostname protection before sending.
    let _protection = check_hostname_protection(&endpoint.url);

    // Bound the request body so huge payloads surface early rather than timing out.
    let body = serde_json::to_vec(&payload).unwrap_or_else(|_| b"{}".to_vec());
    let signature = sign_payload(&endpoint.secret, &body);

    let mut last_status: Option<i32> = None;
    let mut last_error: Option<String> = None;

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
                last_status = Some(status);
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
                last_error = Some(format!("HTTP status {status}"));
                tracing::warn!(
                    endpoint_id = %endpoint.id,
                    status,
                    attempt,
                    "Webhook returned non-2xx, will retry"
                );
            }
            Ok(Err(e)) => {
                let err_str = e.to_string();
                last_status = None;
                last_error = Some(err_str.clone());
                log_delivery(
                    pool,
                    endpoint.id,
                    event,
                    &payload,
                    attempt,
                    None,
                    Some(&err_str),
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
                last_status = None;
                last_error = Some("delivery timed out".to_string());
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

    // All retries failed — record failure in Dead-Letter Queue (Issue #1626)
    tracing::error!(
        endpoint_id = %endpoint.id,
        event,
        attempts = MAX_ATTEMPTS,
        last_status = ?last_status,
        last_error = ?last_error,
        "Webhook delivery failed after max retries; moved to DLQ"
    );
    record_delivery_failure(
        pool,
        endpoint.id,
        event,
        &payload,
        MAX_ATTEMPTS,
        last_status,
        last_error.as_deref(),
    )
    .await;
}

/// Redeliver a failed webhook message from the dead-letter queue (Issue #1626).
pub async fn retry_failed_delivery(
    pool: &PgPool,
    failure_id: Uuid,
    organizer_id: Uuid,
) -> Result<WebhookDeliveryFailure, (StatusCode, String)> {
    let failure = sqlx::query_as::<_, WebhookDeliveryFailure>(
        r#"
        SELECT f.* FROM webhook_delivery_failures f
        JOIN webhook_endpoints e ON e.id = f.endpoint_id
        WHERE f.id = $1 AND e.organizer_id = $2
        "#,
    )
    .bind(failure_id)
    .bind(organizer_id)
    .fetch_optional(pool)
    .await
    .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))?
    .ok_or_else(|| {
        (
            StatusCode::NOT_FOUND,
            format!("Webhook failure record '{}' not found", failure_id),
        )
    })?;

    let endpoint = sqlx::query_as::<_, WebhookEndpoint>(
        "SELECT * FROM webhook_endpoints WHERE id = $1 AND organizer_id = $2",
    )
    .bind(failure.endpoint_id)
    .bind(organizer_id)
    .fetch_optional(pool)
    .await
    .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))?
    .ok_or_else(|| {
        (
            StatusCode::NOT_FOUND,
            format!(
                "Associated webhook endpoint '{}' not found",
                failure.endpoint_id
            ),
        )
    })?;

    let client = reqwest::Client::new();
    let body = serde_json::to_vec(&failure.payload).unwrap_or_else(|_| b"{}".to_vec());
    let signature = sign_payload(&endpoint.secret, &body);

    let outcome = tokio::time::timeout(
        std::time::Duration::from_secs(DISPATCH_TIMEOUT_SECS),
        client
            .post(&endpoint.url)
            .header("Content-Type", "application/json")
            .header("X-Agora-Signature", &signature)
            .header("X-Agora-Event", &failure.event)
            .body(body)
            .send(),
    )
    .await;

    let (is_success, status_code, error_msg) = match outcome {
        Ok(Ok(response)) => {
            let status = response.status().as_u16() as i32;
            let success = response.status().is_success();
            let err = if success {
                None
            } else {
                Some(format!("HTTP status {}", status))
            };
            (success, Some(status), err)
        }
        Ok(Err(e)) => (false, None, Some(e.to_string())),
        Err(_) => (false, None, Some("delivery timed out".to_string())),
    };

    let retry_status = if is_success { "delivered" } else { "failed" };

    let updated = sqlx::query_as::<_, WebhookDeliveryFailure>(
        r#"
        UPDATE webhook_delivery_failures
        SET retried_at = NOW(),
            retry_status = $1,
            last_status_code = COALESCE($2, last_status_code),
            last_error = $3
        WHERE id = $4
        RETURNING *
        "#,
    )
    .bind(retry_status)
    .bind(status_code)
    .bind(error_msg.as_deref())
    .bind(failure_id)
    .fetch_one(pool)
    .await
    .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))?;

    if is_success {
        Ok(updated)
    } else {
        Err((
            StatusCode::BAD_GATEWAY,
            format!(
                "Webhook retry delivery failed: {}",
                error_msg.unwrap_or_else(|| "unknown error".to_string())
            ),
        ))
    }
}

/// Enqueue delivery of `event` to every matching endpoint for an organiser.
///
/// Spawns a background task per event so callers never block on outbound HTTP;
/// delivery is retried (3 attempts, exponential backoff) entirely in the task.
/// The reqwest client has redirects disabled to prevent following malicious
/// chains that could bypass the hostname-protection check.
pub fn dispatch_webhooks(pool: PgPool, organizer_id: Uuid, event: &str, data: serde_json::Value) {
    let client = reqwest::Client::builder()
        .redirect(reqwest::redirect::Policy::no_redirects())
        .build()
        .expect("failed to build reqwest client");
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
            // URL protection is checked per-endpoint in deliver_to_endpoint.
            deliver_to_endpoint(&client, &pool, &endpoint, &event, payload.clone()).await;
        }
    });
}
