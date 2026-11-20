//! # Webhook Management Handlers (Issue #1339)
//!
//! `POST /api/v1/webhooks` — register a new endpoint (max 5 per organiser).
//! `GET  /api/v1/webhooks` — list the authenticated organiser's endpoints.
//! `DELETE /api/v1/webhooks/:id` — deactivate (soft delete) an endpoint.

use axum::{
    extract::{Path, Query, State},
    http::HeaderMap,
    response::{IntoResponse, Response},
};
use serde::{Deserialize, Serialize};
use sqlx::PgPool;
use utoipa::{IntoParams, ToSchema};
use uuid::Uuid;

use crate::models::webhook::{validate_create_webhook, MAX_WEBHOOK_ENDPOINTS_PER_ORGANIZER};
pub use crate::models::webhook::{CreateWebhookRequest, WebhookDeliveryFailure, WebhookEndpoint};
use crate::utils::error::AppError;
use crate::utils::response::success;

/// Application state for webhook handlers.
#[derive(Clone)]
pub struct WebhookState {
    pub pool: PgPool,
}

/// Response envelope for a newly created webhook endpoint.
#[derive(Debug, Serialize, ToSchema)]
pub struct CreateWebhookResponse {
    pub endpoint: WebhookEndpoint,
    /// Registered endpoints remaining for this organiser.
    pub remaining_slots: usize,
}

/// Result of sending a test event to a webhook endpoint.
#[derive(Debug, Serialize, ToSchema)]
pub struct TestWebhookResponse {
    pub endpoint_id: Uuid,
    pub url: String,
    pub event: String,
    pub status_code: Option<u16>,
    pub latency_ms: u64,
    pub success: bool,
    pub error: Option<String>,
}

async fn authenticated_organizer_id(pool: &PgPool, headers: &HeaderMap) -> Result<Uuid, Response> {
    let wallet_address =
        crate::handlers::auth::extract_auth(headers).map_err(|error| error.into_response())?;

    match sqlx::query_scalar::<_, Uuid>("SELECT id FROM organizers WHERE wallet_address = $1")
        .bind(wallet_address)
        .fetch_optional(pool)
        .await
    {
        Ok(Some(organizer_id)) => Ok(organizer_id),
        Ok(None) => Err(AppError::NotFound("Organizer not found".to_string()).into_response()),
        Err(error) => Err(AppError::DatabaseError(error).into_response()),
    }
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
        (status = 401, description = "Authentication required"),
    ),
    security(("bearer_auth" = [])),
    tag = "Webhooks"
)]
pub async fn create_webhook(
    State(state): State<WebhookState>,
    headers: HeaderMap,
    axum::extract::Json(payload): axum::extract::Json<CreateWebhookRequest>,
) -> Response {
    let organizer_id = match authenticated_organizer_id(&state.pool, &headers).await {
        Ok(organizer_id) => organizer_id,
        Err(response) => return response,
    };

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
    .bind(organizer_id)
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
    .bind(organizer_id)
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
/// GET `/api/v1/webhooks`
#[utoipa::path(
    get,
    path = "/webhooks",
    responses(
        (status = 200, description = "Organizer webhook endpoints", body = Vec<WebhookEndpoint>),
        (status = 401, description = "Authentication required"),
    ),
    security(("bearer_auth" = [])),
    tag = "Webhooks"
)]
pub async fn list_webhooks(State(state): State<WebhookState>, headers: HeaderMap) -> Response {
    let organizer_id = match authenticated_organizer_id(&state.pool, &headers).await {
        Ok(organizer_id) => organizer_id,
        Err(response) => return response,
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
        (status = 401, description = "Authentication required"),
    ),
    security(("bearer_auth" = [])),
    tag = "Webhooks"
)]
pub async fn delete_webhook(
    State(state): State<WebhookState>,
    Path(endpoint_id): Path<Uuid>,
    headers: HeaderMap,
) -> Response {
    let organizer_id = match authenticated_organizer_id(&state.pool, &headers).await {
        Ok(organizer_id) => organizer_id,
        Err(response) => return response,
    };

    match sqlx::query(
        "UPDATE webhook_endpoints SET is_active = FALSE WHERE id = $1 AND organizer_id = $2",
    )
    .bind(endpoint_id)
    .bind(organizer_id)
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

/// Query parameters for `GET /api/v1/webhooks/failures`.
#[derive(Debug, Deserialize, Default, IntoParams, ToSchema)]
pub struct ListWebhookFailuresParams {
    /// Filter failures by endpoint id.
    pub endpoint_id: Option<Uuid>,
}

/// List webhook delivery failures from Dead-Letter Queue (DLQ).
///
/// # Endpoint
/// GET `/api/v1/webhooks/failures`
#[utoipa::path(
    get,
    path = "/webhooks/failures",
    params(ListWebhookFailuresParams),
    responses(
        (status = 200, description = "Webhook DLQ failures retrieved", body = Vec<WebhookDeliveryFailure>),
        (status = 404, description = "Webhook endpoint not found"),
        (status = 401, description = "Authentication required"),
    ),
    security(("bearer_auth" = [])),
    tag = "Webhooks"
)]
pub async fn list_webhook_failures(
    State(state): State<WebhookState>,
    headers: HeaderMap,
    Query(params): Query<ListWebhookFailuresParams>,
) -> Response {
    let organizer_id = match authenticated_organizer_id(&state.pool, &headers).await {
        Ok(organizer_id) => organizer_id,
        Err(response) => return response,
    };

    if let Some(endpoint_id) = params.endpoint_id {
        let endpoint_exists = match sqlx::query_scalar::<_, bool>(
            "SELECT EXISTS(SELECT 1 FROM webhook_endpoints WHERE id = $1 AND organizer_id = $2)",
        )
        .bind(endpoint_id)
        .bind(organizer_id)
        .fetch_one(&state.pool)
        .await
        {
            Ok(exists) => exists,
            Err(error) => return AppError::DatabaseError(error).into_response(),
        };
        if !endpoint_exists {
            return AppError::NotFound(format!("Webhook endpoint '{}' not found", endpoint_id))
                .into_response();
        }
    }

    let result = if let Some(endpoint_id) = params.endpoint_id {
        sqlx::query_as::<_, WebhookDeliveryFailure>(
            r#"
            SELECT f.* FROM webhook_delivery_failures f
            JOIN webhook_endpoints e ON f.endpoint_id = e.id
            WHERE e.id = $1 AND e.organizer_id = $2
            ORDER BY f.failed_at DESC
            "#,
        )
        .bind(endpoint_id)
        .bind(organizer_id)
        .fetch_all(&state.pool)
        .await
    } else {
        sqlx::query_as::<_, WebhookDeliveryFailure>(
            r#"
            SELECT f.* FROM webhook_delivery_failures f
            JOIN webhook_endpoints e ON f.endpoint_id = e.id
            WHERE e.organizer_id = $1
            ORDER BY f.failed_at DESC
            LIMIT 100
            "#,
        )
        .bind(organizer_id)
        .fetch_all(&state.pool)
        .await
    };

    match result {
        Ok(failures) => success(failures, "Webhook failures retrieved").into_response(),
        Err(e) => AppError::DatabaseError(e).into_response(),
    }
}

/// Retry a permanently failed webhook delivery from the Dead-Letter Queue (DLQ).
///
/// # Endpoint
/// POST `/api/v1/webhooks/failures/:id/retry`
#[utoipa::path(
    post,
    path = "/webhooks/failures/{id}/retry",
    params(("id" = Uuid, Path, description = "Webhook failure identifier")),
    responses(
        (status = 200, description = "Webhook redelivered successfully", body = WebhookDeliveryFailure),
        (status = 404, description = "Failure record or endpoint not found"),
        (status = 502, description = "Webhook redelivery attempt failed"),
        (status = 401, description = "Authentication required"),
    ),
    security(("bearer_auth" = [])),
    tag = "Webhooks"
)]
pub async fn retry_webhook_failure(
    State(state): State<WebhookState>,
    Path(failure_id): Path<Uuid>,
    headers: HeaderMap,
) -> Response {
    let organizer_id = match authenticated_organizer_id(&state.pool, &headers).await {
        Ok(organizer_id) => organizer_id,
        Err(response) => return response,
    };

    match crate::services::webhook_dispatcher::retry_failed_delivery(
        &state.pool,
        failure_id,
        organizer_id,
    )
    .await
    {
        Ok(failure) => success(failure, "Webhook redelivered successfully").into_response(),
        Err((status, msg)) => crate::utils::error::ApiError::new(status, msg).into_response(),
    }
}

/// Send a test event to a webhook endpoint.
#[utoipa::path(
    post,
    path = "/webhooks/{endpoint_id}/test",
    params(("endpoint_id" = Uuid, Path, description = "Webhook endpoint identifier")),
    responses(
        (status = 200, description = "Test delivery result", body = TestWebhookResponse),
        (status = 404, description = "Webhook endpoint not found"),
        (status = 401, description = "Authentication required"),
    ),
    security(("bearer_auth" = [])),
    tag = "Webhooks"
)]
pub async fn test_webhook(
    State(state): State<WebhookState>,
    Path(endpoint_id): Path<Uuid>,
    headers: HeaderMap,
) -> Response {
    let organizer_id = match authenticated_organizer_id(&state.pool, &headers).await {
        Ok(organizer_id) => organizer_id,
        Err(response) => return response,
    };

    let endpoint = match sqlx::query_as::<_, WebhookEndpoint>(
        "SELECT * FROM webhook_endpoints WHERE id = $1 AND organizer_id = $2",
    )
    .bind(endpoint_id)
    .bind(organizer_id)
    .fetch_optional(&state.pool)
    .await
    {
        Ok(Some(endpoint)) => endpoint,
        Ok(None) => {
            return AppError::NotFound(format!("Webhook endpoint '{}' not found", endpoint_id))
                .into_response()
        }
        Err(error) => return AppError::DatabaseError(error).into_response(),
    };

    let event = "Ping";
    let payload = serde_json::json!({
        "id": Uuid::new_v4(),
        "event": event,
        "timestamp": chrono::Utc::now().to_rfc3339(),
        "data": { "message": "Test webhook delivery" },
    });
    let body = match serde_json::to_vec(&payload) {
        Ok(body) => body,
        Err(error) => {
            return AppError::InternalServerError(format!(
                "Failed to serialize test webhook payload: {error}"
            ))
            .into_response()
        }
    };
    let signature = crate::services::webhook_dispatcher::sign_payload(&endpoint.secret, &body);
    let started_at = std::time::Instant::now();
    let delivery = tokio::time::timeout(
        std::time::Duration::from_secs(5),
        reqwest::Client::new()
            .post(&endpoint.url)
            .header("Content-Type", "application/json")
            .header("X-Agora-Signature", signature)
            .header("X-Agora-Event", event)
            .body(body)
            .send(),
    )
    .await;

    let (status_code, delivery_succeeded, error) = match delivery {
        Ok(Ok(response)) => {
            let status = response.status();
            (
                Some(status.as_u16()),
                status.is_success(),
                (!status.is_success()).then(|| format!("HTTP status {status}")),
            )
        }
        Ok(Err(error)) => (None, false, Some(error.to_string())),
        Err(_) => (None, false, Some("delivery timed out".to_string())),
    };

    success(
        TestWebhookResponse {
            endpoint_id,
            url: endpoint.url,
            event: event.to_string(),
            status_code,
            latency_ms: started_at.elapsed().as_millis() as u64,
            success: delivery_succeeded,
            error,
        },
        "Webhook test delivery completed",
    )
    .into_response()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn validation_rejects_non_https_url() {
        let req = CreateWebhookRequest {
            url: "http://insecure.example.com/hook".to_string(),
            secret: "s3cret".to_string(),
            events: vec!["PaymentProcessed".to_string()],
        };
        assert!(validate_create_webhook(&req).is_err());
    }

    #[test]
    fn validation_rejects_unknown_event() {
        let req = CreateWebhookRequest {
            url: "https://example.com/hook".to_string(),
            secret: "s3cret".to_string(),
            events: vec!["RefundInitiated".to_string()],
        };
        assert!(validate_create_webhook(&req).is_err());
    }

    #[test]
    fn validation_deduplicates_and_normalises_events() {
        let req = CreateWebhookRequest {
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
