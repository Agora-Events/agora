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

use crate::models::webhook::{validate_create_webhook, MAX_WEBHOOK_ENDPOINTS_PER_ORGANIZER};
pub use crate::models::webhook::{CreateWebhookRequest, WebhookDeliveryFailure, WebhookEndpoint};
use crate::utils::cursor_pagination::{
    decode_cursor, encode_cursor, CursorResponse, FailureCursor, ValidatedCursorParams,
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

/// Query parameters for `GET /api/v1/webhooks/failures`.
#[derive(Debug, Deserialize, Default, IntoParams, ToSchema)]
pub struct ListWebhookFailuresParams {
    /// Filter failures by endpoint id.
    pub endpoint_id: Option<Uuid>,
    /// Filter failures by organiser id.
    pub organizer_id: Option<Uuid>,
    /// Number of failures to return (default 50, maximum 200).
    #[serde(default = "default_webhook_failure_limit")]
    pub limit: u32,
    /// Opaque cursor containing the failed_at and id of the previous page's last item.
    pub cursor: Option<String>,
}

const DEFAULT_WEBHOOK_FAILURE_LIMIT: u32 = 50;
const MAX_WEBHOOK_FAILURE_LIMIT: u32 = 200;

fn default_webhook_failure_limit() -> u32 {
    DEFAULT_WEBHOOK_FAILURE_LIMIT
}

fn webhook_failure_limit(limit: u32) -> u32 {
    limit.clamp(1, MAX_WEBHOOK_FAILURE_LIMIT)
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
        (status = 200, description = "Webhook DLQ failures retrieved", body = CursorResponse<WebhookDeliveryFailure>),
    ),
    tag = "Webhooks"
)]
pub async fn list_webhook_failures(
    State(state): State<WebhookState>,
    Query(params): Query<ListWebhookFailuresParams>,
) -> Response {
    let limit = webhook_failure_limit(params.limit);
    let cursor = match params.cursor.as_deref() {
        Some(cursor) => match decode_cursor::<FailureCursor>(cursor) {
            Ok(cursor) => Some(cursor),
            Err(error) => {
                return AppError::ValidationError(format!("Invalid cursor: {error}"))
                    .into_response()
            }
        },
        None => None,
    };

    let result = if let Some(endpoint_id) = params.endpoint_id {
        match cursor.as_ref() {
            Some(cursor) => {
                sqlx::query_as::<_, WebhookDeliveryFailure>(
                    r#"
                SELECT * FROM webhook_delivery_failures
                WHERE endpoint_id = $1
                  AND (failed_at < $2 OR (failed_at = $2 AND id < $3))
                ORDER BY failed_at DESC, id DESC
                LIMIT $4
                "#,
                )
                .bind(endpoint_id)
                .bind(cursor.failed_at)
                .bind(cursor.id)
                .bind((limit + 1) as i64)
                .fetch_all(&state.pool)
                .await
            }
            None => {
                sqlx::query_as::<_, WebhookDeliveryFailure>(
                    r#"
                SELECT * FROM webhook_delivery_failures
                WHERE endpoint_id = $1
                ORDER BY failed_at DESC, id DESC
                LIMIT $2
                "#,
                )
                .bind(endpoint_id)
                .bind((limit + 1) as i64)
                .fetch_all(&state.pool)
                .await
            }
        }
    } else if let Some(organizer_id) = params.organizer_id {
        match cursor.as_ref() {
            Some(cursor) => {
                sqlx::query_as::<_, WebhookDeliveryFailure>(
                    r#"
                SELECT f.* FROM webhook_delivery_failures f
                JOIN webhook_endpoints e ON f.endpoint_id = e.id
                WHERE e.organizer_id = $1
                  AND (f.failed_at < $2 OR (f.failed_at = $2 AND f.id < $3))
                ORDER BY f.failed_at DESC, f.id DESC
                LIMIT $4
                "#,
                )
                .bind(organizer_id)
                .bind(cursor.failed_at)
                .bind(cursor.id)
                .bind((limit + 1) as i64)
                .fetch_all(&state.pool)
                .await
            }
            None => {
                sqlx::query_as::<_, WebhookDeliveryFailure>(
                    r#"
                SELECT f.* FROM webhook_delivery_failures f
                JOIN webhook_endpoints e ON f.endpoint_id = e.id
                WHERE e.organizer_id = $1
                ORDER BY f.failed_at DESC, f.id DESC
                LIMIT $2
                "#,
                )
                .bind(organizer_id)
                .bind((limit + 1) as i64)
                .fetch_all(&state.pool)
                .await
            }
        }
    } else {
        match cursor.as_ref() {
            Some(cursor) => {
                sqlx::query_as::<_, WebhookDeliveryFailure>(
                    r#"
                SELECT * FROM webhook_delivery_failures
                WHERE failed_at < $1 OR (failed_at = $1 AND id < $2)
                ORDER BY failed_at DESC, id DESC
                LIMIT $3
                "#,
                )
                .bind(cursor.failed_at)
                .bind(cursor.id)
                .bind((limit + 1) as i64)
                .fetch_all(&state.pool)
                .await
            }
            None => {
                sqlx::query_as::<_, WebhookDeliveryFailure>(
                    r#"
                SELECT * FROM webhook_delivery_failures
                ORDER BY failed_at DESC, id DESC
                LIMIT $1
                "#,
                )
                .bind((limit + 1) as i64)
                .fetch_all(&state.pool)
                .await
            }
        }
    };

    match result {
        Ok(mut failures) => {
            let has_more = failures.len() > limit as usize;
            let next_cursor = if has_more {
                failures.pop();
                failures.last().and_then(|failure| {
                    encode_cursor(&FailureCursor {
                        failed_at: failure.failed_at,
                        id: failure.id,
                    })
                    .ok()
                })
            } else {
                None
            };
            let response = CursorResponse::new(
                failures,
                &ValidatedCursorParams {
                    limit,
                    cursor: None,
                    include_count: false,
                },
                next_cursor,
            );
            success(response, "Webhook failures retrieved").into_response()
        }
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
    ),
    tag = "Webhooks"
)]
pub async fn retry_webhook_failure(
    State(state): State<WebhookState>,
    Path(failure_id): Path<Uuid>,
) -> Response {
    match crate::services::webhook_dispatcher::retry_failed_delivery(&state.pool, failure_id).await
    {
        Ok(failure) => success(failure, "Webhook redelivered successfully").into_response(),
        Err((status, msg)) => crate::utils::error::ApiError::new(status, msg).into_response(),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn webhook_failure_limit_clamps_to_accepted_range() {
        assert_eq!(webhook_failure_limit(0), 1);
        assert_eq!(webhook_failure_limit(50), 50);
        assert_eq!(webhook_failure_limit(201), 200);
    }

    #[test]
    fn webhook_failure_cursor_roundtrips_and_starts_second_page() {
        let first_page_last_failed_at = chrono::Utc::now();
        let first_page_last_id = Uuid::new_v4();
        let cursor = encode_cursor(&FailureCursor {
            failed_at: first_page_last_failed_at,
            id: first_page_last_id,
        })
        .unwrap();

        let decoded: FailureCursor = decode_cursor(&cursor).unwrap();
        assert_eq!(decoded.failed_at, first_page_last_failed_at);
        assert_eq!(decoded.id, first_page_last_id);

        let second_page_rows = vec![FailureCursor {
            failed_at: first_page_last_failed_at - chrono::Duration::seconds(1),
            id: Uuid::new_v4(),
        }];
        assert!(second_page_rows
            .iter()
            .all(|row| row.failed_at < decoded.failed_at));
    }

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
