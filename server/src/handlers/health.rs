use axum::{extract::State, response::IntoResponse, response::Response};
use chrono::Utc;
use serde::Serialize;
use sqlx::PgPool;

use crate::utils::error::AppError;
use crate::utils::response::success;

#[derive(Serialize)]
struct HealthResponse {
    status: &'static str,
    timestamp: String,
}

#[derive(Serialize)]
struct HealthDbResponse {
    status: &'static str,
    database: &'static str,
    timestamp: String,
}

#[derive(Serialize)]
struct HealthReadyResponse {
    status: &'static str,
    api: &'static str,
    database: &'static str,
}

/// GET /health – Combined check for API and Database.
///
/// Returns 200 when both the API process and the database are healthy.
/// On failure it returns a structured JSON 503 error (via [`AppError`]).
pub async fn health_check(State(pool): State<PgPool>) -> Response {
    match sqlx::query("SELECT 1").fetch_one(&pool).await {
        Ok(_) => {
            let payload = HealthResponse {
                status: "ok",
                timestamp: Utc::now().to_rfc3339(),
            };
            success(payload, "API is healthy").into_response()
        }
        Err(e) => {
            tracing::error!("Health check failed: {:?}", e);
            AppError::ExternalServiceError(format!(
                "API is not ready: database is unreachable ({e})"
            ))
            .into_response()
        }
    }
}

/// GET /health/db – Database connectivity check.
///
/// Returns 200 when the database is reachable.
/// Returns a structured JSON error (via [`AppError`]) when it is not,
/// ensuring the error payload matches the API-wide error schema.
pub async fn health_check_db(State(pool): State<PgPool>) -> Response {
    match sqlx::query("SELECT 1").fetch_one(&pool).await {
        Ok(_) => {
            let payload = HealthDbResponse {
                status: "ok",
                database: "connected",
                timestamp: Utc::now().to_rfc3339(),
            };
            success(payload, "Database is healthy").into_response()
        }
        Err(e) => {
            // Delegate to AppError so the error body is identical to every
            // other error response in the API.
            AppError::ExternalServiceError(format!("Database health check failed: {e}"))
                .into_response()
        }
    }
}

/// GET /health/ready – Readiness check.
///
/// Returns 200 only when both the API process and the database are healthy.
/// On failure the response uses [`AppError`] for a consistent error schema.
pub async fn health_check_ready(State(pool): State<PgPool>) -> Response {
    let db_ok = sqlx::query("SELECT 1").fetch_one(&pool).await.is_ok();

    if db_ok {
        let payload = HealthReadyResponse {
            status: "ready",
            api: "ok",
            database: "ok",
        };
        success(payload, "Service is ready").into_response()
    } else {
        AppError::ExternalServiceError("Service is not ready: database is unreachable".to_string())
            .into_response()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::utils::error::AppError;
    use axum::{
        body::Body,
        http::{Request, StatusCode},
        routing::get,
        Router,
    };
    use tower::ServiceExt;

    #[tokio::test]
    async fn test_health_response_ok_status() {
        // Success case for health check response.
        let payload = HealthResponse {
            status: "ok",
            timestamp: Utc::now().to_rfc3339(),
        };
        let resp = success(payload, "API is healthy").into_response();
        assert_eq!(resp.status(), StatusCode::OK);
    }

    #[tokio::test]
    async fn test_health_db_error_status() {
        // DB Failure case for health check response (via AppError).
        let err = AppError::ExternalServiceError("database is unreachable".to_string());
        let resp = err.into_response();
        assert_eq!(resp.status(), StatusCode::SERVICE_UNAVAILABLE);
    }

    #[tokio::test]
    async fn test_health_endpoint_returns_200_with_expected_json() {
        let router = Router::new().route(
            "/health",
            get(|| async {
                let payload = HealthResponse {
                    status: "ok",
                    timestamp: Utc::now().to_rfc3339(),
                };
                success(payload, "API is healthy").into_response()
            }),
        );

        let req = Request::builder()
            .uri("/health")
            .body(Body::empty())
            .unwrap();

        let resp = router.oneshot(req).await.unwrap();

        assert_eq!(resp.status(), StatusCode::OK);

        let bytes = axum::body::to_bytes(resp.into_body(), usize::MAX)
            .await
            .unwrap();
        let json: serde_json::Value = serde_json::from_slice(&bytes).unwrap();

        assert_eq!(json["success"], true);
        assert_eq!(json["message"], "API is healthy");
        assert_eq!(json["data"]["status"], "ok");
        assert!(json["data"]["timestamp"].is_string());
    }
}
