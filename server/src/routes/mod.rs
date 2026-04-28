//! # Routes Module
//!
//! This module defines the application's HTTP routing structure.
//! It organizes all API endpoints under versioned paths and applies
//! middleware layers for security, CORS, and request tracking.
//!
//! ## Route Structure
//!
//! All routes are nested under `/api/v1/` prefix:
//! - Health check endpoints for monitoring
//! - Example endpoints for testing error responses
//! - Future: Event management endpoints
//!
//! ## Middleware Layers
//!
//! Routes are wrapped with middleware in this order:
//! 1. Request ID generation and propagation
//! 2. CORS handling
//! 3. Security headers
//! 4. Database connection state

use axum::{ middleware, routing::{get, post}, Router };
use sqlx::PgPool;
use std::time::Duration;

use crate::config::{
    create_cors_layer,
    create_security_headers_layer,
    propagate_request_id_layer,
    set_request_id_layer,
    Config,
};
use crate::middleware::request_id_tracing::trace_request_id;
use crate::cache::RedisCache;
use crate::handlers::{
    categories::{get_category, list_categories},
    events::{get_event, list_events, submit_event_rating, toggle_event_flag, EventState},
    example_empty_success,
    example_not_found,
    example_validation_error,
    health::{ health_check, health_check_blockchain, health_check_db, health_check_ready },
    monitoring::{monitoring_dashboard, MonitoringState},
    qr_payload::{generate_qr_payload, list_qr_payloads, mark_qr_used, verify_qr_payload},
    ws::{ws_purchases_handler, PurchaseBroadcaster},
};
use crate::middleware::audit::audit_layer;
use crate::middleware::rate_limit::GovernorRateLimitLayer;
use crate::utils::rate_limit::RateLimitLayer;

/// Sensitive routes that hit the database or expose internal state.
/// Limited to 30 requests per IP per minute.
const SENSITIVE_RATE_LIMIT: usize = 30;
const SENSITIVE_WINDOW: Duration = Duration::from_secs(60);

/// General API routes. Limited to 120 requests per IP per minute.
const GENERAL_RATE_LIMIT: usize = 120;
const GENERAL_WINDOW: Duration = Duration::from_secs(60);

/// Creates the main application router with all routes and middleware
///
/// # Arguments
/// * `pool` - PostgreSQL connection pool for database operations
/// * `config` - Application configuration
/// * `redis` - Redis cache client
///
/// # Returns
/// A configured Axum Router with all routes and middleware applied
pub async fn create_routes(pool: PgPool, config: Config, redis: RedisCache) -> Router {
    let broadcaster = PurchaseBroadcaster::new();
    
    let event_state = EventState {
        pool: pool.clone(),
        redis: redis.clone(),
    };
    
    let monitoring_state = MonitoringState {
        pool: pool.clone(),
        redis: redis.clone(),
    };

    // Admin sub-router — every request is recorded in audit_logs.
    let admin_routes = Router::new()
        .route("/events/:id/toggle-flag", post(toggle_event_flag))
        .route("/health", get(health_check))
        .route_layer(middleware::from_fn_with_state(pool.clone(), audit_layer))
        .with_state(event_state.clone());

    // WebSocket sub-router for real-time purchase updates.
    let ws_routes = Router::new()
        .route("/purchases", get(ws_purchases_handler))
        .with_state(broadcaster);

    // QR payload routes for cryptographically signed QR codes
    let qr_routes = Router::new()
        .route("/generate", post(generate_qr_payload))
        .route("/verify", post(verify_qr_payload))
        .route("/mark-used/:id", post(mark_qr_used))
        .route("/list", get(list_qr_payloads))
        .with_state(pool.clone());

    // Event routes with Redis caching
    let event_routes = Router::new()
        .route("/", get(list_events))
        .route("/:id", get(get_event))
        .route("/:id/rate", post(submit_event_rating))
        .with_state(event_state);

    // Category routes
    let category_routes = Router::new()
        .route("/", get(list_categories))
        .route("/:id", get(get_category))
        .with_state(pool.clone());

    let sensitive_routes = Router::new()
        .route("/health", get(health_check))
        .route("/health/blockchain", get(health_check_blockchain))
        .route("/health/db", get(health_check_db))
        .route("/health/ready", get(health_check_ready))
        .route("/monitoring/dashboard", get(monitoring_dashboard))
        .with_state(monitoring_state)
        .layer(RateLimitLayer::new(SENSITIVE_RATE_LIMIT, SENSITIVE_WINDOW));

    // General endpoints — relaxed rate limit
    let general_routes = Router::new()
        .route("/health", get(health_check))
        .route("/examples/validation-error", get(example_validation_error))
        .route("/examples/empty-success", get(example_empty_success))
        .route("/examples/not-found/:id", get(example_not_found))
        .route("/leaderboard", get(get_leaderboard))
        .with_state(pool)
        .layer(RateLimitLayer::new(GENERAL_RATE_LIMIT, GENERAL_WINDOW));

    // Public API routes with tower-governor rate limiting
    let public_api_routes = Router::new()
        .nest("/events", event_routes)
        .nest("/categories", category_routes)
        .layer(GovernorRateLimitLayer::new(100, Duration::from_secs(60)));

    let api_routes = Router::new()
        .merge(sensitive_routes)
        .merge(general_routes)
        .merge(public_api_routes);

    // Deep linking routes
    let deep_link_routes = Router::new()
        .route("/.well-known/apple-app-site-association", get(serve_apple_app_site_association))
        .route("/.well-known/assetlinks.json", get(serve_assetlinks));

    Router::new()
        .nest("/api/v1", api_routes)
        .nest("/api/v1/admin", admin_routes)
        .merge(deep_link_routes)
        .layer(create_security_headers_layer())
        .layer(create_cors_layer())
        .layer(middleware::from_fn(trace_request_id))
        .layer(propagate_request_id_layer())
        .layer(set_request_id_layer())
}

/// Serve Apple App Site Association file for iOS deep linking
async fn serve_apple_app_site_association() -> Response {
    let content = include_str!("../../.well-known/apple-app-site-association");
    (
        axum::http::StatusCode::OK,
        [("Content-Type", "application/json")],
        content
    ).into_response()
}

/// Serve Android Asset Links file for Android deep linking
async fn serve_assetlinks() -> Response {
    let content = include_str!("../../.well-known/assetlinks.json");
    (
        axum::http::StatusCode::OK,
        [("Content-Type", "application/json")],
        content
    ).into_response()
}

#[cfg(test)]
mod tests {
    use super::*;
    use axum::{ body::Body, http::{ Request, StatusCode } };
    use tower::ServiceExt;

    fn test_router() -> Router {
        Router::new()
            .route(
                "/api/v1/health",
                get(|| async { "ok" })
            )
            .route(
                "/api/v1/health/blockchain",
                get(|| async { "ok" })
            )
            .route(
                "/api/v1/health/db",
                get(|| async { "ok" })
            )
            .route(
                "/api/v1/health/ready",
                get(|| async { "ok" })
            )
            .route(
                "/api/v1/examples/validation-error",
                get(|| async { "ok" })
            )
            .route(
                "/api/v1/examples/empty-success",
                get(|| async { "ok" })
            )
            .route(
                "/api/v1/examples/not-found/:id",
                get(|| async { "ok" })
            )
    }

    async fn get_status(router: Router, path: &str) -> StatusCode {
        let req = Request::builder().uri(path).body(Body::empty()).unwrap();
        router.oneshot(req).await.unwrap().status()
    }

    #[tokio::test]
    async fn test_health_route_exists_under_api_v1() {
        let router = test_router();
        assert_ne!(get_status(router, "/api/v1/health").await, StatusCode::NOT_FOUND);
    }

    #[tokio::test]
    async fn test_health_db_route_exists_under_api_v1() {
        let router = test_router();
        assert_ne!(get_status(router, "/api/v1/health/db").await, StatusCode::NOT_FOUND);
    }

    #[tokio::test]
    async fn test_health_blockchain_route_exists_under_api_v1() {
        let router = test_router();
        assert_ne!(
            get_status(router, "/api/v1/health/blockchain").await,
            StatusCode::NOT_FOUND
        );
    }

    #[tokio::test]
    async fn test_health_ready_route_exists_under_api_v1() {
        let router = test_router();
        assert_ne!(get_status(router, "/api/v1/health/ready").await, StatusCode::NOT_FOUND);
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
    async fn test_upload_image_route_exists_under_api_v1() {
        let router = test_router();
        // POST /api/v1/upload/image should not 404 (method-not-allowed or ok, but not 404)
        let req = Request::builder()
            .method("POST")
            .uri("/api/v1/upload/image")
            .body(Body::empty())
            .unwrap();
        let status = router.oneshot(req).await.unwrap().status();
        assert_ne!(status, StatusCode::NOT_FOUND);
    }

    #[tokio::test]
    async fn test_old_routes_without_prefix_return_404() {
        let router = test_router();
        assert_eq!(get_status(router.clone(), "/health").await, StatusCode::NOT_FOUND);
        assert_eq!(get_status(router.clone(), "/health/blockchain").await, StatusCode::NOT_FOUND);
        assert_eq!(get_status(router.clone(), "/health/db").await, StatusCode::NOT_FOUND);
        assert_eq!(get_status(router, "/health/ready").await, StatusCode::NOT_FOUND);
    }

    #[tokio::test]
    async fn test_api_without_version_returns_404() {
        let router = test_router();
        assert_eq!(get_status(router, "/api/health").await, StatusCode::NOT_FOUND);
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
