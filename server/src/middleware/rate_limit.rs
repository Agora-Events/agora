//! # Rate Limiting Middleware
//!
//! Per-IP limits are enforced by [`crate::utils::rate_limit::RateLimitLayer`]:
//! 30 req/min on sensitive routes and 120 req/min on general routes.
//!
//! Every response includes:
//! * `X-RateLimit-Limit` — bucket capacity for this route class
//! * `X-RateLimit-Remaining` — tokens left in the current window
//! * `X-RateLimit-Reset` — Unix timestamp (seconds) when the window refreshes
//!
//! Rejected requests additionally receive `429 Too Many Requests` with a
//! `Retry-After` header set to the seconds remaining in the current window
//! (always ≥ 1).

use std::time::Duration;
use tower::Layer;

pub use crate::utils::rate_limit::{apply_rate_limit_headers, RateLimitLayer};

/// No-op rate limit layer kept for call-site compatibility.
/// Real limiting is applied via [`RateLimitLayer`].
#[derive(Clone)]
pub struct GovernorRateLimitLayer;

impl GovernorRateLimitLayer {
    /// Create a new (no-op) rate limit layer.
    pub fn new(_requests_per_minute: u64, _window: Duration) -> Self {
        Self
    }
}

impl<S: Clone> Layer<S> for GovernorRateLimitLayer {
    type Service = S;

    fn layer(&self, inner: S) -> Self::Service {
        inner
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use axum::{
        body::Body,
        http::{Request, StatusCode},
        routing::get,
        Router,
    };
    use tower::{ServiceBuilder, ServiceExt};

    fn test_router(max: usize, window: Duration) -> Router {
        Router::new()
            .route("/test", get(|| async { "ok" }))
            .layer(RateLimitLayer::new(max, window))
    }

    async fn send_request(router: &Router, ip: &str) -> axum::response::Response {
        let req = Request::builder()
            .uri("/test")
            .header("x-forwarded-for", ip)
            .body(Body::empty())
            .unwrap();
        router.clone().oneshot(req).await.unwrap()
    }

    #[tokio::test]
    async fn test_under_limit_requests_pass() {
        let router = test_router(2, Duration::from_secs(60));
        let ip = "192.168.1.10";

        let resp1 = send_request(&router, ip).await;
        assert_eq!(resp1.status(), StatusCode::OK);

        let resp2 = send_request(&router, ip).await;
        assert_eq!(resp2.status(), StatusCode::OK);
    }

    #[tokio::test]
    async fn test_over_limit_request_returns_429_with_positive_integer_retry_after() {
        let router = test_router(2, Duration::from_secs(60));
        let ip = "192.168.1.20";

        // First 2 requests should pass
        let resp1 = send_request(&router, ip).await;
        assert_eq!(resp1.status(), StatusCode::OK);

        let resp2 = send_request(&router, ip).await;
        assert_eq!(resp2.status(), StatusCode::OK);

        // 3rd request exceeds limit -> 429
        let resp3 = send_request(&router, ip).await;
        assert_eq!(resp3.status(), StatusCode::TOO_MANY_REQUESTS);

        // Header Retry-After must be present and parse as a positive integer >= 1
        let retry_after = resp3
            .headers()
            .get("retry-after")
            .expect("Retry-After header must be present on 429 response")
            .to_str()
            .expect("Retry-After header must be a valid string");

        let retry_after_secs: u64 = retry_after
            .parse()
            .expect("Retry-After must parse as an integer");

        assert!(
            retry_after_secs >= 1,
            "Retry-After must be a positive integer, got {}",
            retry_after_secs
        );
    }

    #[tokio::test]
    async fn test_different_ips_are_not_affected() {
        let router = test_router(2, Duration::from_secs(60));
        let ip1 = "10.0.0.1";
        let ip2 = "10.0.0.2";

        // Exhaust IP1's quota
        let _ = send_request(&router, ip1).await;
        let _ = send_request(&router, ip1).await;
        let resp1_blocked = send_request(&router, ip1).await;
        assert_eq!(resp1_blocked.status(), StatusCode::TOO_MANY_REQUESTS);

        // IP2 should still be able to make requests
        let resp2_allowed = send_request(&router, ip2).await;
        assert_eq!(resp2_allowed.status(), StatusCode::OK);
    }

    #[tokio::test]
    async fn test_governor_rate_limit_layer_passthrough() {
        let layer = GovernorRateLimitLayer::new(100, Duration::from_secs(60));
        let router = Router::new()
            .route("/test", get(|| async { "passthrough" }))
            .layer(ServiceBuilder::new().layer(layer));

        let req = Request::builder()
            .uri("/test")
            .body(Body::empty())
            .unwrap();

        let resp = router.oneshot(req).await.unwrap();
        assert_eq!(resp.status(), StatusCode::OK);
    }
}
