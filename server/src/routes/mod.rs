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

use crate::handlers::{delta_sync, sync_status, SyncState};
use crate::handlers::indexer::{replay_indexer, IndexerAdminState};
use axum::{
    error_handling::HandleErrorLayer,
    middleware,
    response::IntoResponse,
    response::Response,
    routing::{delete, get, patch, post},
    Router,
};
use sqlx::PgPool;
use std::time::Duration;
use tower::ServiceBuilder;
use tower_http::compression::{
    predicate::{DefaultPredicate, Predicate, SizeAbove},
    CompressionLayer,
};
use tower_http::limit::RequestBodyLimitLayer;
use tower_http::timeout::TimeoutLayer;

use crate::cache::RedisCache;
use crate::config::{
    create_cors_layer, create_security_headers_layer, propagate_request_id_layer,
    set_request_id_layer, Config,
};
use crate::middleware::catch_panic::catch_panic_layer;
use crate::middleware::content_type::require_json_content_type;
use crate::middleware::request_id_tracing::{propagate_request_id, trace_request_id};
use crate::utils::rate_limit::RateLimitLayer;

use crate::handlers::{
    auth::{logout, request_nonce, verify_signature},
    categories::{get_category, list_categories, CategoryState},
    events::{
        export_attendees_csv, flag_event, get_attendee_count, get_checkin_stats, get_event,
        get_event_counts, get_event_organizer, get_event_share_link, get_event_social_proof,
        get_events_map, get_ratings_summary, list_event_attendees, list_event_ratings,
        list_event_tickets, list_events, list_events_by_category, list_past_events,
        list_similar_events, list_ticket_tiers, list_upcoming_events, search_events,
        set_event_featured, submit_event_rating, toggle_event_flag, EventState,
    },
    governance::{
        cast_vote, get_dispute, get_dispute_votes, list_disputes, open_dispute,
        resolve_dispute, GovernanceState,
    },
    example_empty_success, example_not_found, example_validation_error,
    health::{
        health_check, health_check_blockchain, health_check_db, health_check_ready,
        health_check_redis, version,
    },
    marketplace::{
        cancel_listing, create_key_envelope, create_listing, create_offer, get_key_envelope,
        get_listing, list_listings, list_offers, register_push_token, MarketplaceState,
    },
    pricing::{
        get_bonding_curve_price, get_bonding_curve_series, get_dutch_auction_price, PricingState,
    },
    profile::{
        delete_profile, get_my_profile, get_organizer_stats, get_profile_by_address,
        get_wallet_tickets, list_events_by_organizer, list_my_transactions, patch_profile,
        upsert_profile, ProfileState,
    },
    qr_payload::{
        delete_qr_payload, generate_attendee_qr, generate_qr_payload, list_event_qr_codes,
        list_qr_payloads, mark_qr_used, scan_ticket, verify_qr_payload,
    },
    rates::{get_rates, RatesState},
    waiting_room::{
        join_queue, queue_status, queue_stream, request_challenge, WaitingRoomConfig,
        WaitingRoomState,
    },
    ws::{ws_purchases_handler, PurchaseBroadcaster},
    zk_checkin::{get_ring, register_commitment, seal_bucket, zk_checkin, ZkCheckinState},
};
use crate::metrics::{metrics_handler, track_metrics};
use crate::middleware::admin_auth::{require_admin_token, AdminAuthState};
use crate::middleware::audit::audit_layer;
use crate::services::indexer::spawn_indexer;
use crate::services::indexer::IndexerConfig;
use crate::services::queue::QueueEngine;

/// Sensitive routes that hit the database or expose internal state.
/// Limited to 30 requests per IP per minute.
const SENSITIVE_RATE_LIMIT: usize = 30;
const SENSITIVE_WINDOW: Duration = Duration::from_secs(60);

/// General API routes. Limited to 120 requests per IP per minute.
const GENERAL_RATE_LIMIT: usize = 120;
const GENERAL_WINDOW: Duration = Duration::from_secs(60);

use utoipa::OpenApi;

#[derive(OpenApi)]
#[openapi(
    paths(
        crate::handlers::health::health_check
    ),
    components(
        schemas(
            crate::handlers::health::HealthResponse,
            crate::utils::error::ApiError
        )
    ),
    tags(
        (name = "Agora API", description = "Agora Events Platform API")
    )
)]
pub struct ApiDoc;

pub async fn create_routes(pool: PgPool, config: Config, redis: RedisCache) -> Router {
    let broadcaster = PurchaseBroadcaster::new();

    // Virtual waiting room (Issue #1187): queue engine + background admission
    // worker. Admission rate / PoW difficulty / grant TTL are env-tunable.
    let waiting_room_config = WaitingRoomConfig::from_env();
    let waiting_room_engine = std::sync::Arc::new(QueueEngine::new(redis.clone()));
    QueueEngine::spawn_admission_worker(
        waiting_room_engine.clone(),
        Duration::from_millis(waiting_room_config.tick_interval_ms),
        waiting_room_config.grant_ttl_minutes,
    );

    let event_state = EventState {
        pool: pool.clone(),
        redis: redis.clone(),
        base_url: config.base_url.clone(),
    };

    let rates_state = RatesState::new(redis.clone(), reqwest::Client::new());

    // Sync state for CRDT delta synchronization
    let sync_state = SyncState::new(pool.clone());
    // Dynamic pricing state for Dutch auction & bonding curve projections (Issue #1175)
    let pricing_state = PricingState::new(redis.clone());

    // Spawn the high-throughput, re-org resilient Soroban event indexer (Issue #1174)
    let indexer_config = IndexerConfig::from_env();
    spawn_indexer(
        pool.clone(),
        Some(redis.clone()),
        Some(broadcaster.clone()),
        indexer_config,
    );

    // Auth routes — challenge-response JWT flow (Issue #484, #875)
    let auth_routes = Router::new()
        .route("/nonce", post(request_nonce))
        .route("/verify", post(verify_signature))
        .route("/logout", post(logout))
        .with_state(pool.clone())
        .layer(RateLimitLayer::new(
            config.auth_rate_limit_per_minute,
            Duration::from_secs(60),
        ));

    let profile_state = ProfileState {
        pool: pool.clone(),
        redis: redis.clone(),
    };

    // Organizer profile routes (Issue #486)
    let profile_routes = Router::new()
        .route(
            "/",
            get(get_my_profile)
                .put(upsert_profile)
                .patch(patch_profile)
                .delete(delete_profile),
        )
        .route("/transactions", get(list_my_transactions))
        .route("/tickets", get(get_wallet_tickets))
        .route("/:address", get(get_profile_by_address))
        .route("/:address/events", get(list_events_by_organizer))
        .route("/:address/stats", get(get_organizer_stats))
        .with_state(profile_state);

    // Admin sub-router — every request is recorded in audit_logs and requires admin auth.
    let admin_auth_state = AdminAuthState {
        token: config.admin_token.clone(),
    };

    let governance_state = GovernanceState {
        pool: pool.clone(),
    };

    let governance_routes = Router::new()
        .route("/disputes", get(list_disputes).post(open_dispute))
        .route("/disputes/:id", get(get_dispute))
        .route(
            "/disputes/:id/votes",
            get(get_dispute_votes).post(cast_vote),
        )
        .route("/disputes/:id/resolve", post(resolve_dispute))
        .layer(middleware::from_fn_with_state(
            admin_auth_state.clone(),
            require_admin_token,
        ))
        .layer(middleware::from_fn_with_state(pool.clone(), audit_layer))
        .with_state(governance_state);

    let admin_routes = Router::new()
        .route("/events/:id/toggle-flag", post(toggle_event_flag))
        .route("/events/:id/feature", patch(set_event_featured))
        .route("/events/:id/flag", patch(flag_event))
        .merge(governance_routes)
        .route_layer(middleware::from_fn_with_state(
            admin_auth_state,
            require_admin_token,
        ))
        .route_layer(middleware::from_fn_with_state(pool.clone(), audit_layer))
        .with_state(event_state.clone())
        .merge(
            Router::new()
                .route("/indexer/replay", get(replay_indexer))
                .with_state(IndexerAdminState {
                    pool: pool.clone(),
                    broker: Some(broadcaster.clone()),
                }),
        );

    // WebSocket sub-router for real-time purchase updates.
    let ws_routes = Router::new()
        .route("/purchases", get(ws_purchases_handler))
        .with_state(broadcaster);

    // Virtual waiting room routes (Issue #1187)
    let waiting_room_state = WaitingRoomState {
        engine: waiting_room_engine,
        pool: pool.clone(),
        config: waiting_room_config,
    };
    let waiting_room_routes = Router::new()
        .route("/challenge", post(request_challenge))
        .route("/join", post(join_queue))
        .route("/status", get(queue_status))
        .with_state(waiting_room_state.clone());

    // Long-lived SSE stream — kept out of the request timeout layer below so
    // it isn't cut off while a client is still connected (Issue #1252).
    let waiting_room_stream_routes = Router::new()
        .route("/stream", get(queue_stream))
        .with_state(waiting_room_state);

    // QR payload routes for cryptographically signed QR codes
    let qr_routes = Router::new()
        .route("/generate", post(generate_qr_payload))
        .route("/attendee", post(generate_attendee_qr))
        .route("/verify", post(verify_qr_payload))
        .route("/mark-used/:id", post(mark_qr_used))
        .route("/list", get(list_qr_payloads))
        .route("/:id", delete(delete_qr_payload))
        .with_state(pool.clone());

    // Zero-knowledge ticket attestation (Issue #1186). The public half is the
    // gate: fetch an anonymity set, then present a proof. Registering
    // commitments and sealing a set are issuer actions and live under /admin.
    let zk_state = ZkCheckinState::new(pool.clone());
    let zk_routes = Router::new()
        .route("/ring", get(get_ring))
        .route("/checkin", post(zk_checkin))
        .with_state(zk_state.clone());

    let admin_zk_routes = Router::new()
        .route("/zk/commitments", post(register_commitment))
        .route("/zk/buckets/seal", post(seal_bucket))
        .route_layer(middleware::from_fn_with_state(
            AdminAuthState {
                token: config.admin_token.clone(),
            },
            require_admin_token,
        ))
        .route_layer(middleware::from_fn_with_state(pool.clone(), audit_layer))
        .with_state(zk_state);

    // Event routes with Redis caching
    let event_routes = Router::new()
        .route("/", get(list_events))
        .route("/map", get(get_events_map))
        .route("/count", get(get_event_counts))
        .route("/past", get(list_past_events))
        .route("/upcoming", get(list_upcoming_events))
        .route("/search", get(search_events))
        .route("/:id", get(get_event))
        .route("/:id/attendees", get(list_event_attendees))
        .route("/:id/attendees/count", get(get_attendee_count))
        .route("/:id/rate", post(submit_event_rating))
        .route("/:id/check-in-stats", get(get_checkin_stats))
        .route("/:id/ratings", get(list_event_ratings))
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

    // Ticket scan routes for organiser verification
    let ticket_routes = Router::new()
        .route("/:id/scan", post(scan_ticket))
        .with_state(pool.clone());

    // Secondary ticket market (Issue #1184). The key-envelope endpoints relay
    // seller-sealed ticket secrets; see `handlers::marketplace` for the trust
    // model. Push delivery is the first consumer of `NotificationService`.
    let mut notification_service = crate::notifications::NotificationService::new();
    notification_service.register(crate::notifications::push::ExpoPushProvider::new(
        reqwest::Client::new(),
    ));
    let marketplace_state = MarketplaceState {
        pool: pool.clone(),
        notifications: std::sync::Arc::new(notification_service),
    };
    let marketplace_routes = Router::new()
        .route("/listings", get(list_listings).post(create_listing))
        .route(
            "/listings/:payment_id",
            get(get_listing).delete(cancel_listing),
        )
        .route(
            "/listings/:payment_id/offers",
            get(list_offers).post(create_offer),
        )
        .route(
            "/listings/:payment_id/key-envelope",
            get(get_key_envelope).post(create_key_envelope),
        )
        .route("/push-token", post(register_push_token))
        .with_state(marketplace_state);

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

    let sensitive_routes = Router::new()
        .route("/health/blockchain", get(health_check_blockchain))
        .route("/health/db", get(health_check_db))
        .route("/health/ready", get(health_check_ready))
        .with_state(pool.clone())
        .merge(
            Router::new()
                .route("/health/redis", get(health_check_redis))
                .with_state(redis.clone()),
        )
        .merge(
            Router::new()
                .route("/health", get(health_check))
                .with_state(crate::handlers::health::HealthState {
                    pool: pool.clone(),
                    redis: redis.clone(),
                }),
        )
        .layer(RateLimitLayer::new(SENSITIVE_RATE_LIMIT, SENSITIVE_WINDOW));

    // General endpoints — relaxed rate limit
    let general_routes = Router::new()
        .route("/examples/validation-error", get(example_validation_error))
        .route("/examples/empty-success", get(example_empty_success))
        .route("/examples/not-found/:id", get(example_not_found))
        .route("/version", get(version))
        .with_state(pool)
        .layer(RateLimitLayer::new(GENERAL_RATE_LIMIT, GENERAL_WINDOW));

    // Public API routes with tower-governor rate limiting
    let rates_route = Router::new()
        .route("/rates", get(get_rates))
        .with_state(rates_state);

    // Sync routes for CRDT delta synchronization
    let sync_routes = Router::new()
        .route("/delta", post(delta_sync))
        .route("/status/:node_id", get(sync_status))
        .with_state(sync_state);
    // Dynamic pricing routes: Dutch auction projections and bonding curve
    // visualisation series (Issue #1175).
    let pricing_routes = Router::new()
        .route("/dutch-auction", get(get_dutch_auction_price))
        .route("/bonding-curve", get(get_bonding_curve_price))
        .route("/bonding-curve/series", get(get_bonding_curve_series))
        .with_state(pricing_state);

    let public_api_routes = Router::new()
        .nest("/events", event_routes)
        .nest("/events", event_qr_routes)
        .nest("/tickets", ticket_routes)
        .nest("/marketplace", marketplace_routes)
        .nest("/categories", category_routes)
        .nest("/auth", auth_routes)
        .nest("/profile", profile_routes)
        .nest("/waiting-room", waiting_room_routes)
        .nest("/qr", qr_routes)
        .nest("/zk", zk_routes)
        .nest("/pricing", pricing_routes)
        .nest("/sync", sync_routes)
        .merge(rates_route)
        .layer(middleware::from_fn(require_json_content_type))
        .layer(RequestBodyLimitLayer::new(1024 * 1024))
        // Per-IP token-bucket rate limit (RATE_LIMIT_MAX / RATE_LIMIT_WINDOW).
        .layer(RateLimitLayer::from_env());

    // WebSocket and SSE routes get the same content-type/body-limit/rate-limit
    // protections as the rest of the public API, but are exempt from the
    // request timeout below since they are meant to stay connected
    // indefinitely (Issue #1252).
    let streaming_routes = Router::new()
        .nest("/ws", ws_routes)
        .nest("/waiting-room", waiting_room_stream_routes)
        .layer(middleware::from_fn(require_json_content_type))
        .layer(RequestBodyLimitLayer::new(1024 * 1024))
        .layer(RateLimitLayer::from_env());

    let api_routes = Router::new()
        .merge(sensitive_routes)
        .merge(general_routes)
        .merge(public_api_routes)
        .merge(
            utoipa_swagger_ui::SwaggerUi::new("/swagger-ui")
                .url("/openapi.json", ApiDoc::openapi()),
        )
        .layer(
            ServiceBuilder::new()
                .layer(HandleErrorLayer::new(crate::utils::error::handle_timeout_error))
                .layer(TimeoutLayer::new(Duration::from_secs(
                    config.request_timeout_secs,
                ))),
        )
        // gzip/br response compression — excludes /metrics (mounted outside
        // `api_routes`) and the streaming routes merged in below, and skips
        // small bodies that wouldn't benefit (Issue #1253).
        .layer(CompressionLayer::new().compress_when(DefaultPredicate::new().and(SizeAbove::new(1024))))
        .merge(streaming_routes)
        .layer(middleware::from_fn(crate::middleware::csrf::check_csrf));

    // Deep linking routes
    let deep_link_routes = Router::new()
        .route(
            "/.well-known/apple-app-site-association",
            get(serve_apple_app_site_association),
        )
        .route("/.well-known/assetlinks.json", get(serve_assetlinks));

    Router::new()
        .route("/metrics", get(metrics_handler))
        .nest("/api/v1", api_routes)
        .nest("/api/v1/admin", admin_routes.merge(admin_zk_routes))
        .merge(deep_link_routes)
        .fallback(handle_404)
        .layer(middleware::from_fn(track_metrics))
        .layer(create_security_headers_layer())
        .layer(create_cors_layer())
        .layer(middleware::from_fn(trace_request_id))
        .layer(middleware::from_fn(propagate_request_id))
        .layer(propagate_request_id_layer())
        .layer(set_request_id_layer())
        .layer(catch_panic_layer())
}

async fn handle_404() -> Response {
    crate::utils::error::ApiError::new(axum::http::StatusCode::NOT_FOUND, "Route not found").into_response()
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
            .route("/api/v1/health/redis", get(|| async { "ok" }))
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
    async fn test_health_redis_route_exists_under_api_v1() {
        let router = test_router();
        assert_ne!(
            get_status(router, "/api/v1/health/redis").await,
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

    fn rate_limited_test_router(sensitive_max: usize, general_max: usize) -> Router {
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

    #[tokio::test]
    async fn test_slow_handler_returns_504_gateway_timeout() {
        use axum::body::Body;
        use tower::ServiceExt;

        let router = Router::new()
            .route(
                "/slow",
                get(|| async {
                    tokio::time::sleep(Duration::from_millis(50)).await;
                    "too slow"
                }),
            )
            .layer(
                ServiceBuilder::new()
                    .layer(HandleErrorLayer::new(
                        crate::utils::error::handle_timeout_error,
                    ))
                    .layer(TimeoutLayer::new(Duration::from_millis(10))),
            );

        let req = Request::builder()
            .uri("/slow")
            .body(Body::empty())
            .unwrap();

        let resp = router.oneshot(req).await.unwrap();

        assert_eq!(resp.status(), StatusCode::GATEWAY_TIMEOUT);

        let bytes = axum::body::to_bytes(resp.into_body(), usize::MAX)
            .await
            .unwrap();
        let json: serde_json::Value = serde_json::from_slice(&bytes).unwrap();
        assert_eq!(json["code"], "SERVICE_UNAVAILABLE");
    }
}
