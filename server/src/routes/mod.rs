use axum::{routing::get, Router};
use sqlx::PgPool;
use std::time::Duration;

use crate::config::{
    create_cors_layer, create_security_headers_layer, propagate_request_id_layer,
    set_request_id_layer,
};
use crate::handlers::{
    example_empty_success, example_not_found, example_validation_error,
    health::{health_check, health_check_db, health_check_ready},
};
use crate::utils::rate_limit::RateLimitLayer;

/// Sensitive routes that hit the database or expose internal state.
/// Limited to 30 requests per IP per minute.
const SENSITIVE_RATE_LIMIT: usize = 30;
const SENSITIVE_WINDOW: Duration = Duration::from_secs(60);

/// General API routes. Limited to 120 requests per IP per minute.
const GENERAL_RATE_LIMIT: usize = 120;
const GENERAL_WINDOW: Duration = Duration::from_secs(60);

pub fn create_routes(pool: PgPool) -> Router {
    // Sensitive endpoints — stricter rate limit
    let sensitive_routes = Router::new()
        .route("/health/db", get(health_check_db))
        .route("/health/ready", get(health_check_ready))
        .with_state(pool.clone())
        .layer(RateLimitLayer::new(SENSITIVE_RATE_LIMIT, SENSITIVE_WINDOW));

    // General endpoints — relaxed rate limit
    let general_routes = Router::new()
        .route("/health", get(health_check))
        .route("/examples/validation-error", get(example_validation_error))
        .route("/examples/empty-success", get(example_empty_success))
        .route("/examples/not-found/:id", get(example_not_found))
        .with_state(pool)
        .layer(RateLimitLayer::new(GENERAL_RATE_LIMIT, GENERAL_WINDOW));

    let api_routes = Router::new()
        .merge(sensitive_routes)
        .merge(general_routes);

    Router::new()
        .nest("/api/v1", api_routes)
        .layer(create_security_headers_layer())
        .layer(create_cors_layer())
        .layer(propagate_request_id_layer())
        .layer(set_request_id_layer())
}

#[cfg(test)]
mod tests {
    use super::*;
    use axum::{
        body::Body,
        http::{Request, StatusCode},
    };
    use tower::ServiceExt;

    fn test_router() -> Router {
        Router::new()
            .route("/api/v1/health", get(|| async { "ok" }))
            .route("/api/v1/health/db", get(|| async { "ok" }))
            .route("/api/v1/health/ready", get(|| async { "ok" }))
            .route("/api/v1/examples/validation-error", get(|| async { "ok" }))
            .route("/api/v1/examples/empty-success", get(|| async { "ok" }))
            .route("/api/v1/examples/not-found/:id", get(|| async { "ok" }))
    }

    async fn get_status(router: Router, path: &str) -> StatusCode {
        let req = Request::builder().uri(path).body(Body::empty()).unwrap();
        router.oneshot(req).await.unwrap().status()
    }

    #[tokio::test]
    async fn test_health_route_exists_under_api_v1() {
        let router = test_router();
        assert_ne!(
            get_status(router, "/api/v1/health").await,
            StatusCode::NOT_FOUND
        );
    }

    #[tokio::test]
    async fn test_health_db_route_exists_under_api_v1() {
        let router = test_router();
        assert_ne!(
            get_status(router, "/api/v1/health/db").await,
            StatusCode::NOT_FOUND
        );
    }

    #[tokio::test]
    async fn test_health_ready_route_exists_under_api_v1() {
        let router = test_router();
        assert_ne!(
            get_status(router, "/api/v1/health/ready").await,
            StatusCode::NOT_FOUND
        );
    }

    #[tokio::test]
    async fn test_examples_validation_error_route_exists_under_api_v1() {
        let router = test_router();
        assert_ne!(
            get_status(router, "/api/v1/examples/validation-error").await,
            StatusCode::NOT_FOUND
        );
    }

    #[tokio::test]
    async fn test_examples_empty_success_route_exists_under_api_v1() {
        let router = test_router();
        assert_ne!(
            get_status(router, "/api/v1/examples/empty-success").await,
            StatusCode::NOT_FOUND
        );
    }

    #[tokio::test]
    async fn test_examples_not_found_route_exists_under_api_v1() {
        let router = test_router();
        assert_ne!(
            get_status(router, "/api/v1/examples/not-found/123").await,
            StatusCode::NOT_FOUND
        );
    }

    #[tokio::test]
    async fn test_old_routes_without_prefix_return_404() {
        let router = test_router();
        assert_eq!(
            get_status(router.clone(), "/health").await,
            StatusCode::NOT_FOUND
        );
        assert_eq!(
            get_status(router.clone(), "/health/db").await,
            StatusCode::NOT_FOUND
        );
        assert_eq!(
            get_status(router, "/health/ready").await,
            StatusCode::NOT_FOUND
        );
    }

    #[tokio::test]
    async fn test_api_without_version_returns_404() {
        let router = test_router();
        assert_eq!(
            get_status(router, "/api/health").await,
            StatusCode::NOT_FOUND
        );
    }

    // -----------------------------------------------------------------------
    // Rate-limit integration tests
    // -----------------------------------------------------------------------

    fn rate_limited_test_router(sensitive_max: usize, general_max: usize) -> Router {
        use crate::utils::rate_limit::RateLimitLayer;

        let sensitive = Router::new()
            .route("/api/v1/health/db", get(|| async { "ok" }))
            .route("/api/v1/health/ready", get(|| async { "ok" }))
            .layer(RateLimitLayer::new(sensitive_max, Duration::from_secs(60)));

        let general = Router::new()
            .route("/api/v1/health", get(|| async { "ok" }))
            .layer(RateLimitLayer::new(general_max, Duration::from_secs(60)));

        Router::new().merge(sensitive).merge(general)
    }

    async fn get_status_with_ip(router: Router, path: &str, ip: &str) -> StatusCode {
        let req = Request::builder()
            .uri(path)
            .header("x-forwarded-for", ip)
            .body(Body::empty())
            .unwrap();
        router.oneshot(req).await.unwrap().status()
    }

    #[tokio::test]
    async fn test_sensitive_route_rate_limited() {
        let router = rate_limited_test_router(2, 120);
        assert_ne!(
            get_status_with_ip(router.clone(), "/api/v1/health/db", "5.5.5.5").await,
            StatusCode::TOO_MANY_REQUESTS
        );
        assert_ne!(
            get_status_with_ip(router.clone(), "/api/v1/health/db", "5.5.5.5").await,
            StatusCode::TOO_MANY_REQUESTS
        );
        assert_eq!(
            get_status_with_ip(router, "/api/v1/health/db", "5.5.5.5").await,
            StatusCode::TOO_MANY_REQUESTS
        );
    }

    #[tokio::test]
    async fn test_general_route_not_rate_limited_within_limit() {
        let router = rate_limited_test_router(30, 120);
        for _ in 0..5 {
            assert_ne!(
                get_status_with_ip(router.clone(), "/api/v1/health", "6.6.6.6").await,
                StatusCode::TOO_MANY_REQUESTS
            );
        }
    }
}
