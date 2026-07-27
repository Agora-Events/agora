//! # Monitoring Handlers
//!
//! Exposes internal operational metrics (DB pool utilization, etc.) for
//! observability tooling. Protected by a static API key so that unauthenticated
//! callers can't use it for infrastructure reconnaissance.

use axum::{
    extract::State,
    http::{header::AUTHORIZATION, HeaderMap},
    response::{IntoResponse, Response},
};
use serde::Serialize;
use sqlx::PgPool;

use crate::utils::error::AppError;
use crate::utils::response::success;

#[derive(Debug, Serialize)]
pub struct MonitoringMetrics {
    pub db_pool_size: u32,
    pub db_pool_idle: usize,
    pub db_pool_in_use: usize,
}

/// Validates the `Authorization: Bearer <key>` header against the
/// `MONITORING_API_KEY` environment variable.
fn is_authorized(headers: &HeaderMap) -> bool {
    let expected = match std::env::var("MONITORING_API_KEY") {
        Ok(key) if !key.is_empty() => key,
        _ => return false,
    };

    let provided = headers
        .get(AUTHORIZATION)
        .and_then(|v| v.to_str().ok())
        .and_then(|v| v.strip_prefix("Bearer "));

    matches!(provided, Some(token) if token == expected)
}

/// Get internal monitoring metrics.
///
/// # Endpoint
/// GET `/api/v1/monitoring`
///
/// Requires an `Authorization: Bearer <MONITORING_API_KEY>` header. Requests
/// without a valid credential are rejected with 401.
pub async fn get_monitoring(State(pool): State<PgPool>, headers: HeaderMap) -> Response {
    if !is_authorized(&headers) {
        return AppError::AuthError(
            "A valid monitoring credential is required".to_string(),
        )
        .into_response();
    }

    let db_pool_size = pool.size();
    let db_pool_idle = pool.num_idle();
    let metrics = MonitoringMetrics {
        db_pool_size,
        db_pool_idle,
        db_pool_in_use: (db_pool_size as usize).saturating_sub(db_pool_idle),
    };

    success(metrics, "Monitoring metrics retrieved successfully").into_response()
}

#[cfg(test)]
mod tests {
    use super::*;
    use axum::http::HeaderValue;
    use std::sync::Mutex;

    // MONITORING_API_KEY is process-global env state; serialize these tests.
    static ENV_MUTEX: Mutex<()> = Mutex::new(());

    #[test]
    fn test_rejects_missing_credential() {
        let _guard = ENV_MUTEX.lock().unwrap();
        std::env::set_var("MONITORING_API_KEY", "secret-key");
        let headers = HeaderMap::new();
        assert!(!is_authorized(&headers));
        std::env::remove_var("MONITORING_API_KEY");
    }

    #[test]
    fn test_rejects_wrong_credential() {
        let _guard = ENV_MUTEX.lock().unwrap();
        std::env::set_var("MONITORING_API_KEY", "secret-key");
        let mut headers = HeaderMap::new();
        headers.insert(AUTHORIZATION, HeaderValue::from_static("Bearer wrong-key"));
        assert!(!is_authorized(&headers));
        std::env::remove_var("MONITORING_API_KEY");
    }

    #[test]
    fn test_accepts_valid_credential() {
        let _guard = ENV_MUTEX.lock().unwrap();
        std::env::set_var("MONITORING_API_KEY", "secret-key");
        let mut headers = HeaderMap::new();
        headers.insert(AUTHORIZATION, HeaderValue::from_static("Bearer secret-key"));
        assert!(is_authorized(&headers));
        std::env::remove_var("MONITORING_API_KEY");
    }

    #[test]
    fn test_rejects_when_key_not_configured() {
        let _guard = ENV_MUTEX.lock().unwrap();
        std::env::remove_var("MONITORING_API_KEY");
        let mut headers = HeaderMap::new();
        headers.insert(AUTHORIZATION, HeaderValue::from_static("Bearer anything"));
        assert!(!is_authorized(&headers));
    }
}
