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

/// GET /health – Basic liveness check.
/// Returns 200 if the API process is running.
pub async fn health_check() -> Response {
    let payload = HealthResponse {
        status: "ok",
        timestamp: Utc::now().to_rfc3339(),
    };

    success(payload, "API is healthy").into_response()
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
