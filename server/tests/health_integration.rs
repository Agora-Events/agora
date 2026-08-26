//! # Health Integration Test
//!
//! Integration test for health check endpoints and unknown route handling.
//! Drives the Axum router using `tower::ServiceExt::oneshot` with no network
//! listener required and no external services (Postgres / Redis) running.

use axum::{
    body::Body,
    http::{Request, StatusCode},
    middleware,
    response::IntoResponse,
    routing::get,
    Router,
};
use chrono::Utc;
use serde::Serialize;
use serde_json::Value;
use tower::ServiceExt;

use agora_server::config::request_id::{
    propagate_request_id_layer, set_request_id_layer, REQUEST_ID_HEADER,
};
use agora_server::middleware::request_id_tracing::{propagate_request_id, trace_request_id};
use agora_server::utils::error::ApiError;
use agora_server::utils::response::success;

#[derive(Serialize)]
struct HealthResponse {
    status: &'static str,
    timestamp: String,
    category_sync: bool,
    database: &'static str,
    redis: &'static str,
}

async fn health_check_stub() -> impl IntoResponse {
    let payload = HealthResponse {
        status: "ok",
        timestamp: Utc::now().to_rfc3339(),
        category_sync: true,
        database: "ok",
        redis: "ok",
    };
    success(payload, "API is healthy")
}

async fn handle_404() -> impl IntoResponse {
    ApiError::new(StatusCode::NOT_FOUND, "Route not found")
}

fn build_test_router() -> Router {
    let api_routes = Router::new().route("/health", get(health_check_stub));

    Router::new()
        .nest("/api/v1", api_routes)
        .fallback(handle_404)
        .layer(middleware::from_fn(trace_request_id))
        .layer(middleware::from_fn(propagate_request_id))
        .layer(propagate_request_id_layer())
        .layer(set_request_id_layer())
}

#[tokio::test]
async fn test_health_endpoint_returns_200_and_json_shape() {
    let router = build_test_router();

    let req = Request::builder()
        .method("GET")
        .uri("/api/v1/health")
        .body(Body::empty())
        .unwrap();

    let response = router.oneshot(req).await.unwrap();

    assert_eq!(response.status(), StatusCode::OK);

    // Assert response carries x-request-id header
    let request_id_header = response.headers().get(REQUEST_ID_HEADER);
    assert!(
        request_id_header.is_some(),
        "Response must carry x-request-id header"
    );

    let bytes = axum::body::to_bytes(response.into_body(), usize::MAX)
        .await
        .unwrap();
    let json: Value = serde_json::from_slice(&bytes).expect("Valid JSON response");

    // Assert expected JSON shape
    assert_eq!(json["success"], true);
    assert_eq!(json["message"], "API is healthy");
    assert_eq!(json["data"]["status"], "ok");
    assert_eq!(json["data"]["category_sync"], true);
    assert_eq!(json["data"]["database"], "ok");
    assert_eq!(json["data"]["redis"], "ok");
    assert!(json["data"]["timestamp"].is_string());
}

#[tokio::test]
async fn test_unknown_path_returns_404_in_standard_api_error_shape() {
    let router = build_test_router();

    let req = Request::builder()
        .method("GET")
        .uri("/api/v1/unknown-endpoint-that-does-not-exist")
        .body(Body::empty())
        .unwrap();

    let response = router.oneshot(req).await.unwrap();

    assert_eq!(response.status(), StatusCode::NOT_FOUND);

    // Assert response carries x-request-id header
    let request_id_header = response.headers().get(REQUEST_ID_HEADER);
    assert!(
        request_id_header.is_some(),
        "Response must carry x-request-id header"
    );

    let bytes = axum::body::to_bytes(response.into_body(), usize::MAX)
        .await
        .unwrap();
    let json: Value = serde_json::from_slice(&bytes).expect("Valid JSON response");

    // Assert standard ApiError shape: { "code": 404, "message": "..." }
    assert_eq!(json["code"], 404);
    assert!(json["message"].is_string());
}

#[tokio::test]
async fn test_health_endpoint_preserves_custom_x_request_id() {
    let router = build_test_router();
    let custom_id = "test-custom-request-id-12345";

    let req = Request::builder()
        .method("GET")
        .uri("/api/v1/health")
        .header(REQUEST_ID_HEADER, custom_id)
        .body(Body::empty())
        .unwrap();

    let response = router.oneshot(req).await.unwrap();

    assert_eq!(response.status(), StatusCode::OK);
    let request_id = response
        .headers()
        .get(REQUEST_ID_HEADER)
        .and_then(|v| v.to_str().ok());
    assert_eq!(request_id, Some(custom_id));
}
