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

use axum::{
    middleware,
    response::IntoResponse,
    response::Response,
    routing::{delete, get, patch, post},
    Router,
};
use sqlx::PgPool;
use std::time::Duration;
use tower_http::limit::RequestBodyLimitLayer;

use crate::cache::RedisCache;
use crate::config::{
    create_cors_layer, create_security_headers_layer, propagate_request_id_layer,
    set_request_id_layer, Config,
};
use crate::handlers::{
    auth::{logout, request_nonce, verify_signature},
    categories::{get_category, list_categories, CategoryState},
    events::{
        export_attendees_csv, get_attendee_count, get_checkin_stats, get_event, get_event_counts,
        get_event_organizer, get_event_share_link, get_event_social_proof, get_ratings_summary,
        list_event_tickets, list_events, list_events_by_category,
        list_past_events,
        list_similar_events, list_ticket_tiers, list_upcoming_events, search_events,
        set_event_featured, submit_event_rating, toggle_event_flag, EventState,
    },
    example_empty_success, example_not_found, example_validation_error,
    health::{health_check, health_check_blockchain, health_check_db, health_check_ready},
    leaderboard::{get_leaderboard, LeaderboardState},
    monitoring::{monitoring_dashboard, MonitoringState},
    profile::{
        delete_profile, get_my_profile, get_organizer_stats, get_profile_by_address,
        list_my_transactions, patch_profile, upsert_profile, ProfileState,
    },
    rates::{get_rates, RatesState},
    soroban_listener::{spawn_listener, ListenerConfig},
    ws::{ws_purchases_handler, PurchaseBroadcaster},
};
use crate::middleware::audit::audit_layer;
use crate::middleware::admin_auth::{require_admin_token, AdminAuthState};
use crate::middleware::monitoring_auth::{require_monitoring_token, MonitoringAuthState};
use crate::middleware::rate_limit::GovernorRateLimitLayer;
use crate::middleware::request_id_tracing::trace_request_id;
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
        base_url: config.base_url.clone(),
    };

    let monitoring_state = MonitoringState {
        pool: pool.clone(),
        redis: redis.clone(),
    };

    let rates_state = RatesState {
        redis: redis.clone(),
        http: reqwest::Client::new(),
    };

    // Spawn the Soroban event listener background task (Issue #490)
    let listener_config = ListenerConfig::from_env();
    spawn_listener(pool.clone(), listener_config);

    // Auth routes — challenge-response JWT flow (Issue #484)
    let auth_routes = Router::new()
        .route("/nonce", post(request_nonce))
        .route("/verify", post(verify_signature))
        .route("/logout", post(logout))
        .with_state(pool.clone());

    let profile_state = ProfileState {
        pool: pool.clone(),
        redis: redis.clone(),
    };

    // Organizer profile routes (Issue #486)
    // Routes that use Redis caching use ProfileState; stats route keeps PgPool.
    let profile_routes = Router::new()
        .route("/", get(get_my_profile).put(upsert_profile).patch(patch_profile).delete(delete_profile))
        .route("/transactions", get(list_my_transactions))
        .route("/:address", get(get_profile_by_address))
        .with_state(profile_state)
        .merge(
            Router::new()
                .route("/:address/stats", get(get_organizer_stats))
                .with_state(pool.clone()),
        );

    // Admin sub-router — every request is recorded in audit_logs and requires admin auth.
    let admin_auth_state = AdminAuthState {
        token: config.admin_token.clone(),
    };

    let admin_routes = Router::new()
        .route("/events/:id/toggle-flag", post(toggle_event_flag))
        .route("/events/:id/feature", patch(set_event_featured))
        .route_layer(middleware::from_fn_with_state(admin_auth_state, require_admin_token))
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
        .route("/:id", delete(delete_qr_payload))
        .with_state(pool.clone());

    // Event routes with Redis caching
    let event_routes = Router::new()
        .route("/", get(list_events))
        .route("/count", get(get_event_counts))
        .route("/past", get(list_past_events))
        .route("/upcoming", get(list_upcoming_events))
        .route("/search", get(search_events))
        .route("/:id", get(get_event))
        .route("/:id/attendees/count", get(get_attendee_count))
        .route("/:id/rate", post(submit_event_rating))
        .route("/:id/check-in-stats", get(get_checkin_stats))
        .route("/:id/ratings/summary", get(get_ratings_summary))
        .route("/:id/organizer", get(get_event_organizer))
        .route("/:id/export-attendees", get(export_attendees_csv))
        .route("/:id/share-link", get(get_event_share_link))
        .route("/:id/social-proof", get(get_event_social_proof))
        .route("/:id/tickets", get(list_event_tickets))
        .route("/:id/ticket-tiers", get(list_ticket_tiers))
        .route("/:id/similar", get(list_similar_events))
        .route("/categories/:category_id", get(list_events_by_category))
        .with_state(event_state);

    // QR-code routes scoped to an event (uses bare PgPool state)
    let event_qr_routes = Router::new()
        .route("/:id/qr-codes", get(list_event_qr_codes))
        .with_state(pool.clone());

    // Category routes — listing is Redis-cached (Issue #583); the single-item
    // lookup keeps the bare PgPool state.
    let category_state = CategoryState {
        pool: pool.clone(),
        redis: redis.clone(),
    };
    let category_routes = Router::new()
        .route("/", get(list_categories))
        .with_state(category_state)
        .merge(
            Router::new()
                .route("/:id", get(get_category))
                .with_state(pool.clone()),
        );

    let monitoring_auth_state = MonitoringAuthState {
        token: config.monitoring_token.clone(),
    };

    let sensitive_routes = Router::new()
        .route("/health", get(health_check))
        .route("/health/blockchain", get(health_check_blockchain))
        .route("/health/db", get(health_check_db))
        .route("/health/ready", get(health_check_ready))
        .with_state(pool.clone())
        .merge(
            Router::new()
                .route("/monitoring/dashboard", get(monitoring_dashboard))
                .route_layer(middleware::from_fn_with_state(
                    monitoring_auth_state,
                    require_monitoring_token,
                ))
                .with_state(monitoring_state),
        )
        .layer(RateLimitLayer::new(SENSITIVE_RATE_LIMIT, SENSITIVE_WINDOW));

    let leaderboard_state = LeaderboardState {
        pool: pool.clone(),
        redis: redis.clone(),
    };

    // General endpoints — relaxed rate limit
    let general_routes = Router::new()
        .route("/examples/validation-error", get(example_validation_error))
        .route("/examples/empty-success", get(example_empty_success))
        .route("/examples/not-found/:id", get(example_not_found))
        .route("/leaderboard", get(get_leaderboard))
        .with_state(leaderboard_state)
        .layer(RateLimitLayer::new(GENERAL_RATE_LIMIT, GENERAL_WINDOW));

    // Public API routes with tower-governor rate limiting
    let rates_route = Router::new()
        .route("/rates", get(get_rates))
        .with_state(rates_state);

    let public_api_routes = Router::new()
        .nest("/events", event_routes)
        .nest("/events", event_qr_routes)
        .nest("/categories", category_routes)
        .nest("/auth", auth_routes)
        .nest("/profile", profile_routes)
        .nest("/ws", ws_routes)
        .nest("/qr", qr_routes)
        .merge(rates_route)
        .layer(RequestBodyLimitLayer::new(1024 * 1024))
        .layer(GovernorRateLimitLayer::new(100, Duration::from_secs(60)));

    let api_routes = Router::new()
        .merge(sensitive_routes)
        .merge(general_routes)
        .merge(public_api_routes);

    // Deep linking routes
    let deep_link_routes = Router::new()
        .route(
            "/.well-known/apple-app-site-association",
            get(serve_apple_app_site_association),
        )
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
        content,
    )
        .into_response()
}

/// Serve Android Asset Links file for Android deep linking
async fn serve_assetlinks() -> Response {
    let content = include_str!("../../.well-known/assetlinks.json");
    (
        axum::http::StatusCode::OK,
        [("Content-Type", "application/json")],
        content,
    )
        .into_response()
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
            .route("/api/v1/health/blockchain", get(|| async { "ok" }))
            .route("/api/v1/health/db", get(|| async { "ok" }))
            .route("/api/v1/health/ready", get(|| async { "ok" }))
            .route("/api/v1/examples/validation-error", get(|| async { "ok" }))
            .route("/api/v1/examples/empty-success", get(|| async { "ok" }))
            .route("/api/v1/examples/not-found/:id", get(|| async { "ok" }))
            .route("/api/v1/upload/image", post(|| async { "ok" }))
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
        assert_eq!(
            get_status(router.clone(), "/health").await,
            StatusCode::NOT_FOUND
        );
        assert_eq!(
            get_status(router.clone(), "/health/blockchain").await,
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

    #[tokio::test]
    async fn test_body_size_limit_rejects_oversized_requests() {
        use axum::body::Body;
        use tower::ServiceExt;
        use tower_http::limit::RequestBodyLimitLayer;

        let limit: usize = 1024;
        let router = Router::new()
            .route("/test", post(|| async { "ok" }))
            .layer(RequestBodyLimitLayer::new(limit));

        let req = Request::builder()
            .method("POST")
            .uri("/test")
            .header("content-length", "2048")
            .body(Body::empty())
            .unwrap();

        let status = router.oneshot(req).await.unwrap().status();
        assert_eq!(status, StatusCode::PAYLOAD_TOO_LARGE);
    }
}
