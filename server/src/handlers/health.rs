use axum::{extract::State, response::IntoResponse, response::Response, Extension};
use chrono::Utc;
use serde::Serialize;
use serde_json::json;
use sqlx::PgPool;
use std::sync::LazyLock;
use std::time::Duration;

use crate::config::Config;
use crate::utils::error::AppError;
use crate::utils::response::success;

static CATEGORY_SYNC_STATUS: LazyLock<std::sync::Mutex<bool>> =
    LazyLock::new(|| std::sync::Mutex::new(true));

/// Update the category sync status. Called during startup after validation.
pub fn set_category_sync_status(synced: bool) {
    *CATEGORY_SYNC_STATUS.lock().unwrap() = synced;
}

#[derive(Serialize, utoipa::ToSchema)]
pub struct HealthResponse {
    status: &'static str,
    timestamp: String,
    category_sync: bool,
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

#[derive(Serialize)]
struct HealthBlockchainResponse {
    status: &'static str,
    blockchain: &'static str,
    soroban_rpc: String,
    timestamp: String,
}

/// GET /health – Combined check for API and Database.
///
/// Returns 200 when both the API process and the database are healthy.
/// On failure it returns a structured JSON 503 error (via [`AppError`]).
#[utoipa::path(
    get,
    path = "/health",
    responses(
        (status = 200, description = "API is healthy", body = HealthResponse)
    )
)]
pub async fn health_check(State(pool): State<PgPool>) -> Response {
    let category_sync = *CATEGORY_SYNC_STATUS.lock().unwrap();
    
    match sqlx::query("SELECT 1").fetch_one(&pool).await {
        Ok(_) => {
            let payload = HealthResponse {
                status: "ok",
                timestamp: Utc::now().to_rfc3339(),
                category_sync,
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

/// GET /health/blockchain – Soroban RPC connectivity check.
///
/// Returns 200 when the configured Soroban RPC endpoint is reachable.
/// On failure the response uses [`AppError`] for a consistent error schema.
pub async fn health_check_blockchain(
    Extension(config): Extension<Config>,
) -> Response {
    let soroban_rpc_url = config.soroban_rpc_url.clone();

    let client = match reqwest::Client::builder()
        .timeout(Duration::from_secs(5))
        .build()
    {
        Ok(client) => client,
        Err(error) => {
            return AppError::ExternalServiceError(format!(
                "Failed to initialize Soroban RPC probe client: {error}"
            ))
            .into_response();
        }
    };

    let response = client
        .post(&soroban_rpc_url)
        .json(&json!({
            "jsonrpc": "2.0",
            "id": "health-check",
            "method": "getHealth",
        }))
        .send()
        .await;

    match response {
        Ok(resp) if resp.status().is_success() => {
            let payload = HealthBlockchainResponse {
                status: "ok",
                blockchain: "soroban",
                soroban_rpc: soroban_rpc_url,
                timestamp: Utc::now().to_rfc3339(),
            };
            success(payload, "Soroban RPC is reachable").into_response()
        }
        Ok(resp) => AppError::ExternalServiceError(format!(
            "Soroban RPC health check failed with HTTP status {}",
            resp.status()
        ))
        .into_response(),
        Err(error) => {
            AppError::ExternalServiceError(format!("Soroban RPC health check failed: {error}"))
                .into_response()
        }
    }
}

#[derive(Serialize)]
struct HealthRedisResponse {
    status: &'static str,
    timestamp: String,
}

/// GET /health/redis – Redis connectivity check.
///
/// Returns 200 when Redis is reachable.
/// Returns a structured JSON error (via [`AppError`]) when it is not.
pub async fn health_check_redis(State(mut redis): State<crate::cache::RedisCache>) -> Response {
    // Perform a basic Redis command to verify connectivity
    match redis.ping().await {
        Ok(_) => {
            let payload = HealthRedisResponse {
                status: "ok",
                timestamp: Utc::now().to_rfc3339(),
            };
            success(payload, "Redis is healthy").into_response()
        }
        Err(e) => AppError::ExternalServiceError(format!("Redis health check failed: {e}"))
            .into_response(),
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
            category_sync: true,
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
                    category_sync: true,
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
