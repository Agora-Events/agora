//! # Webhook Management Handlers (Issue #1339)
//!
//! `POST /api/v1/webhooks` — register a new endpoint (max 5 per organiser).
//! `GET  /api/v1/webhooks?organizer_id=` — list an organiser's endpoints.
//! `DELETE /api/v1/webhooks/:id` — deactivate (soft delete) an endpoint.

use axum::{
    extract::{Path, Query, State},
    response::{IntoResponse, Response},
};
use serde::{Deserialize, Serialize};
use sqlx::PgPool;
use utoipa::{IntoParams, ToSchema};
use uuid::Uuid;

pub use crate::models::webhook::{
    CreateWebhookRequest, WebhookEndpoint,
};
use crate::models::webhook::{
    validate_create_webhook, MAX_WEBHOOK_ENDPOINTS_PER_ORGANIZER,
};
use crate::utils::error::AppError;
use crate::utils::response::success;

/// Application state for webhook handlers.
#[derive(Clone)]
pub struct WebhookState {
    pub pool: PgPool,
}

/// Query parameters for `GET /api/v1/webhooks`.
#[derive(Debug, Deserialize, Default, IntoParams, ToSchema)]
pub struct ListWebhooksParams {
    /// Organiser id whose endpoints should be listed.
    pub organizer_id: Option<Uuid>,
}

/// Response envelope for a newly created webhook endpoint.
#[derive(Debug, Serialize, ToSchema)]
pub struct CreateWebhookResponse {
    pub endpoint: WebhookEndpoint,
    /// Registered endpoints remaining for this organiser.
    pub remaining_slots: usize,
}

/// Create a webhook endpoint for an organiser.
///
/// # Endpoint
/// POST `/api/v1/webhooks`
#[utoipa::path(
    post,
    path = "/webhooks",
    request_body = CreateWebhookRequest,
    responses(
        (status = 200, description = "Webhook endpoint created", body = CreateWebhookResponse),
        (status = 400, description = "Invalid webhook configuration"),
    ),
    tag = "Webhooks"
)]
pub async fn create_webhook(
    State(state): State<WebhookState>,
    axum::extract::Json(payload): axum::extract::Json<CreateWebhookRequest>,
) -> Response {
    let events = match validate_create_webhook(&payload) {
        Ok(events) => events,
        Err(message) => return AppError::ValidationError(message).into_response(),
    };

    let mut tx = match state.pool.begin().await {
        Ok(tx) => tx,
        Err(e) => return AppError::DatabaseError(e).into_response(),
    };

    // Guarded INSERT enforces the 5-endpoint cap atomically even when several
    // requests race for the same organiser.
    let endpoint = match sqlx::query_as::<_, WebhookEndpoint>(
        r#"
        INSERT INTO webhook_endpoints (organizer_id, url, secret, events, is_active)
        SELECT $1, $2, $3, $4, TRUE
        WHERE (SELECT COUNT(*) FROM webhook_endpoints WHERE organizer_id = $1 AND is_active = TRUE) < $5
        RETURNING *
        "#,
    )
    .bind(payload.organizer_id)
    .bind(payload.url.trim())
    .bind(payload.secret.trim())
    .bind(&events)
    .bind(MAX_WEBHOOK_ENDPOINTS_PER_ORGANIZER as i64)
    .fetch_optional(&mut *tx)
    .await
    {
        Ok(Some(endpoint)) => endpoint,
        Ok(None) => {
            let _ = tx.rollback().await;
            return AppError::ValidationError(format!(
                "an organiser may register at most {MAX_WEBHOOK_ENDPOINTS_PER_ORGANIZER} webhook endpoints"
            ))
            .into_response();
        }
        Err(e) => {
            let _ = tx.rollback().await;
            return AppError::DatabaseError(e).into_response();
        }
    };

    let remaining = match sqlx::query_scalar::<_, i64>(
        "SELECT MAX(0, $1::bigint - COUNT(*)) FROM webhook_endpoints WHERE organizer_id = $2 AND is_active = TRUE",
    )
    .bind(MAX_WEBHOOK_ENDPOINTS_PER_ORGANIZER as i64)
    .bind(payload.organizer_id)
    .fetch_one(&mut *tx)
    .await
    {
        Ok(remaining) => remaining as usize,
        Err(e) => {
            let _ = tx.rollback().await;
            return AppError::DatabaseError(e).into_response();
        }
    };

    if let Err(e) = tx.commit().await {
        return AppError::DatabaseError(e).into_response();
    }

    success(
        CreateWebhookResponse {
            endpoint,
            remaining_slots: remaining,
        },
        "Webhook endpoint created",
    )
    .into_response()
}

/// List an organiser's webhook endpoints.
///
/// # Endpoint
/// GET `/api/v1/webhooks?organizer_id=...`
#[utoipa::path(
    get,
    path = "/webhooks",
    params(ListWebhooksParams),
    responses(
        (status = 200, description = "Organizer webhook endpoints", body = Vec<WebhookEndpoint>),
        (status = 400, description = "organizer_id is required"),
    ),
    tag = "Webhooks"
)]
pub async fn list_webhooks(
    State(state): State<WebhookState>,
    Query(params): Query<ListWebhooksParams>,
) -> Response {
    let Some(organizer_id) = params.organizer_id else {
        return AppError::ValidationError("organizer_id is required".to_string()).into_response();
    };

    match sqlx::query_as::<_, WebhookEndpoint>(
        "SELECT * FROM webhook_endpoints WHERE organizer_id = $1 ORDER BY created_at DESC",
    )
    .bind(organizer_id)
    .fetch_all(&state.pool)
    .await
    {
        Ok(endpoints) => success(endpoints, "Webhook endpoints retrieved").into_response(),
        Err(e) => AppError::DatabaseError(e).into_response(),
    }
}

/// Deactivate a webhook endpoint (soft delete).
///
/// # Endpoint
/// DELETE `/api/v1/webhooks/:id`
#[utoipa::path(
    delete,
    path = "/webhooks/{endpoint_id}",
    params(("endpoint_id" = Uuid, Path, description = "Webhook endpoint identifier")),
    responses(
        (status = 200, description = "Webhook endpoint deactivated"),
        (status = 404, description = "Webhook endpoint not found"),
    ),
    tag = "Webhooks"
)]
pub async fn delete_webhook(
    State(state): State<WebhookState>,
    Path(endpoint_id): Path<Uuid>,
) -> Response {
    match sqlx::query("UPDATE webhook_endpoints SET is_active = FALSE WHERE id = $1")
        .bind(endpoint_id)
        .execute(&state.pool)
        .await
    {
        Ok(result) => {
            if result.rows_affected() == 0 {
                return AppError::NotFound(format!("Webhook endpoint '{}' not found", endpoint_id))
                    .into_response();
            }
            success((), "Webhook endpoint deactivated").into_response()
        }
        Err(e) => AppError::DatabaseError(e).into_response(),
    }
}

/// Response payload for `POST /api/v1/webhooks/:id/test` (Issue #1604).
#[derive(Debug, Serialize, Deserialize, ToSchema, PartialEq)]
pub struct TestWebhookResponse {
    pub endpoint_id: Uuid,
    pub url: String,
    pub event: String,
    pub status_code: Option<u16>,
    pub latency_ms: u64,
    pub success: bool,
    pub error: Option<String>,
}

/// Test a registered webhook endpoint by sending a signed Ping event payload.
///
/// # Endpoint
/// POST `/api/v1/webhooks/:id/test`
pub async fn test_webhook(
    State(state): State<WebhookState>,
    Path(endpoint_id): Path<Uuid>,
) -> Response {
    // 1. Fetch endpoint by id
    let endpoint = match sqlx::query_as::<_, WebhookEndpoint>(
        "SELECT * FROM webhook_endpoints WHERE id = $1",
    )
    .bind(endpoint_id)
    .fetch_optional(&state.pool)
    .await
    {
        Ok(Some(ep)) => ep,
        Ok(None) => {
            return AppError::NotFound(format!("Webhook endpoint '{}' not found", endpoint_id))
                .into_response();
        }
        Err(e) => return AppError::DatabaseError(e).into_response(),
    };

    if !endpoint.is_active {
        return AppError::ValidationError("Cannot test an inactive webhook endpoint".to_string())
            .into_response();
    }

    // 2. Build test Ping payload
    let ping_payload = serde_json::json!({
        "id": Uuid::new_v4(),
        "event": "Ping",
        "timestamp": chrono::Utc::now().to_rfc3339(),
        "data": {
            "endpoint_id": endpoint.id,
            "message": "Agora webhook connectivity test ping",
        }
    });

    let body = serde_json::to_vec(&ping_payload).unwrap_or_else(|_| b"{}".to_vec());
    let signature = crate::services::webhook_dispatcher::sign_payload(&endpoint.secret, &body);

    // 3. Dispatch HTTP POST to the target URL with timing and timeout
    let client = reqwest::Client::builder()
        .timeout(std::time::Duration::from_secs(10))
        .build()
        .unwrap_or_else(|_| reqwest::Client::new());

    let start = std::time::Instant::now();
    let dispatch_result = client
        .post(&endpoint.url)
        .header("Content-Type", "application/json")
        .header("X-Agora-Signature", &signature)
        .header("X-Agora-Event", "Ping")
        .body(body)
        .send()
        .await;
    let latency_ms = start.elapsed().as_millis() as u64;

    let response = match dispatch_result {
        Ok(res) => {
            let status = res.status();
            let status_u16 = status.as_u16();
            let is_success = status.is_success();
            let error_msg = if !is_success {
                Some(format!("Target server returned HTTP {}", status_u16))
            } else {
                None
            };

            TestWebhookResponse {
                endpoint_id: endpoint.id,
                url: endpoint.url,
                event: "Ping".to_string(),
                status_code: Some(status_u16),
                latency_ms,
                success: is_success,
                error: error_msg,
            }
        }
        Err(err) => TestWebhookResponse {
            endpoint_id: endpoint.id,
            url: endpoint.url,
            event: "Ping".to_string(),
            status_code: None,
            latency_ms,
            success: false,
            error: Some(err.to_string()),
        },
    };

    success(response, "Webhook test executed").into_response()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn validation_rejects_non_https_url() {
        let req = CreateWebhookRequest {
            organizer_id: Uuid::new_v4(),
            url: "http://insecure.example.com/hook".to_string(),
            secret: "s3cret".to_string(),
            events: vec!["PaymentProcessed".to_string()],
        };
        assert!(validate_create_webhook(&req).is_err());
    }

    #[test]
    fn validation_rejects_unknown_event() {
        let req = CreateWebhookRequest {
            organizer_id: Uuid::new_v4(),
            url: "https://example.com/hook".to_string(),
            secret: "s3cret".to_string(),
            events: vec!["RefundInitiated".to_string()],
        };
        assert!(validate_create_webhook(&req).is_err());
    }

    #[test]
    fn validation_deduplicates_and_normalises_events() {
        let req = CreateWebhookRequest {
            organizer_id: Uuid::new_v4(),
            url: "https://example.com/hook".to_string(),
            secret: "s3cret".to_string(),
            events: vec![
                "PaymentProcessed".to_string(),
                "PaymentProcessed".to_string(),
                "EventCreated".to_string(),
            ],
        };
        let events = validate_create_webhook(&req).unwrap();
        assert_eq!(events, vec!["PaymentProcessed", "EventCreated"]);
    }

    #[test]
    fn validation_rejects_empty_events() {
        let req = CreateWebhookRequest {
            organizer_id: Uuid::new_v4(),
            url: "https://example.com/hook".to_string(),
            secret: "s3cret".to_string(),
            events: vec![],
        };
        assert!(validate_create_webhook(&req).is_err());
    }

    #[test]
    fn test_webhook_response_serialization() {
        let resp = TestWebhookResponse {
            endpoint_id: Uuid::nil(),
            url: "https://example.com/webhook".to_string(),
            event: "Ping".to_string(),
            status_code: Some(200),
            latency_ms: 45,
            success: true,
            error: None,
        };
        let json = serde_json::to_value(&resp).expect("serializes");
        assert_eq!(json["event"], "Ping");
        assert_eq!(json["status_code"], 200);
        assert_eq!(json["latency_ms"], 45);
        assert_eq!(json["success"], true);
    }
}
