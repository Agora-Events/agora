//! Per-IP token-bucket rate limiter implemented as a Tower [`Layer`].
//!
//! Each unique client IP is allocated a token bucket with capacity
//! `RATE_LIMIT_MAX` that refills over `RATE_LIMIT_WINDOW` seconds.
//! Requests that exceed the limit receive a `429 Too Many Requests`
//! response immediately, without forwarding to the inner service.
//!
//! # Environment
//! * `RATE_LIMIT_MAX` — max requests per window (default: 100)
//! * `RATE_LIMIT_WINDOW` — window length in seconds (default: 60)

use std::{
    net::IpAddr,
    sync::Arc,
    task::{Context, Poll},
    time::{Duration, Instant},
};

use axum::{
    body::Body,
    http::{Request, Response, StatusCode},
};
use dashmap::DashMap;
use serde_json::json;
use tower::{Layer, Service};

// ---------------------------------------------------------------------------
// Token bucket
// ---------------------------------------------------------------------------

#[derive(Clone)]
struct TokenBucket {
    tokens: f64,
    last_refill: Instant,
}

impl TokenBucket {
    fn new(capacity: f64) -> Self {
        Self {
            tokens: capacity,
            last_refill: Instant::now(),
        }
    }

    /// Refill tokens based on elapsed time, then try to consume one.
    /// Returns `true` if the request is allowed.
    fn try_acquire(&mut self, capacity: f64, refill_per_sec: f64) -> bool {
        let now = Instant::now();
        let elapsed = now.duration_since(self.last_refill).as_secs_f64();
        self.tokens = (self.tokens + elapsed * refill_per_sec).min(capacity);
        self.last_refill = now;

        if self.tokens >= 1.0 {
            self.tokens -= 1.0;
            true
        } else {
            false
        }
    }
}

type Store = Arc<DashMap<IpAddr, TokenBucket>>;

// ---------------------------------------------------------------------------
// Config from environment
// ---------------------------------------------------------------------------

/// Read rate-limit settings from `RATE_LIMIT_MAX` / `RATE_LIMIT_WINDOW`.
pub fn rate_limit_from_env() -> (usize, Duration) {
    let max = std::env::var("RATE_LIMIT_MAX")
        .ok()
        .and_then(|v| v.parse().ok())
        .unwrap_or(100);
    let window_secs = std::env::var("RATE_LIMIT_WINDOW")
        .ok()
        .and_then(|v| v.parse().ok())
        .unwrap_or(60);
    (max, Duration::from_secs(window_secs))
}

// ---------------------------------------------------------------------------
// Layer
// ---------------------------------------------------------------------------

/// Tower [`Layer`] that wraps a service with per-IP rate limiting.
#[derive(Clone)]
pub struct RateLimitLayer {
    max_requests: usize,
    window: Duration,
    store: Store,
}

impl RateLimitLayer {
    /// Create a new layer.
    ///
    /// * `max_requests` – bucket capacity (max burst / steady-state rate).
    /// * `window`       – duration over which `max_requests` tokens refill.
    pub fn new(max_requests: usize, window: Duration) -> Self {
        Self {
            max_requests,
            window,
            store: Arc::new(DashMap::new()),
        }
    }

    /// Build a layer from `RATE_LIMIT_MAX` / `RATE_LIMIT_WINDOW` env vars.
    pub fn from_env() -> Self {
        let (max, window) = rate_limit_from_env();
        Self::new(max, window)
    }
}

impl<S> Layer<S> for RateLimitLayer {
    type Service = RateLimitMiddleware<S>;

    fn layer(&self, inner: S) -> Self::Service {
        RateLimitMiddleware {
            inner,
            max_requests: self.max_requests,
            window: self.window,
            store: self.store.clone(),
        }
    }
}

// ---------------------------------------------------------------------------
// Middleware service
// ---------------------------------------------------------------------------

#[derive(Clone)]
pub struct RateLimitMiddleware<S> {
    inner: S,
    max_requests: usize,
    window: Duration,
    store: Store,
}

impl<S> Service<Request<Body>> for RateLimitMiddleware<S>
where
    S: Service<Request<Body>, Response = Response<Body>> + Clone + Send + 'static,
    S::Future: Send + 'static,
{
    type Response = Response<Body>;
    type Error = S::Error;
    type Future = std::pin::Pin<
        Box<dyn std::future::Future<Output = Result<Self::Response, Self::Error>> + Send>,
    >;

    fn poll_ready(&mut self, cx: &mut Context<'_>) -> Poll<Result<(), Self::Error>> {
        self.inner.poll_ready(cx)
    }

    fn call(&mut self, req: Request<Body>) -> Self::Future {
        let ip = extract_ip(&req);

        let allowed = if self.max_requests == 0 {
            false
        } else {
            let capacity = self.max_requests as f64;
            let refill_per_sec = capacity / self.window.as_secs_f64().max(0.001);
            let mut entry = self
                .store
                .entry(ip)
                .or_insert_with(|| TokenBucket::new(capacity));
            entry.try_acquire(capacity, refill_per_sec)
        };

        if !allowed {
            let retry_after = self.window.as_secs().max(1).to_string();
            let body = json!({
                "success": false,
                "error": {
                    "code": 429,
                    "message": "Too many requests. Please try again later."
                }
            })
            .to_string();

            let response = Response::builder()
                .status(StatusCode::TOO_MANY_REQUESTS)
                .header("Content-Type", "application/json")
                .header("Retry-After", retry_after)
                .body(Body::from(body))
                .unwrap();

            return Box::pin(async move { Ok(response) });
        }

        let future = self.inner.call(req);
        Box::pin(future)
    }
}

// ---------------------------------------------------------------------------
// IP extraction
// ---------------------------------------------------------------------------

/// Extract the client IP from `X-Forwarded-For`, `X-Real-IP`, or the peer
/// address stored in request extensions.  Falls back to `127.0.0.1`.
fn extract_ip(req: &Request<Body>) -> IpAddr {
    if let Some(forwarded) = req
        .headers()
        .get("x-forwarded-for")
        .and_then(|v| v.to_str().ok())
    {
        if let Some(ip) = forwarded
            .split(',')
            .next()
            .and_then(|s| s.trim().parse().ok())
        {
            return ip;
        }
    }

    if let Some(real_ip) = req
        .headers()
        .get("x-real-ip")
        .and_then(|v| v.to_str().ok())
        .and_then(|s| s.trim().parse().ok())
    {
        return real_ip;
    }

    if let Some(addr) = req
        .extensions()
        .get::<axum::extract::ConnectInfo<std::net::SocketAddr>>()
    {
        return addr.0.ip();
    }

    IpAddr::from([127, 0, 0, 1])
}

// ---------------------------------------------------------------------------
// Tests
// ---------------------------------------------------------------------------

#[cfg(test)]
mod tests {
    use super::*;
    use axum::{body::Body, http::Request, routing::get, Router};
    use http_body_util::BodyExt;
    use tower::ServiceExt;

    fn rate_limited_router(max: usize, window: Duration) -> Router {
        Router::new()
            .route("/test", get(|| async { "ok" }))
            .layer(RateLimitLayer::new(max, window))
    }

    async fn send(router: &Router, ip: &str) -> StatusCode {
        let req = Request::builder()
            .uri("/test")
            .header("x-forwarded-for", ip)
            .body(Body::empty())
            .unwrap();
        router.clone().oneshot(req).await.unwrap().status()
    }

    #[tokio::test]
    async fn test_requests_within_limit_are_allowed() {
        let router = rate_limited_router(3, Duration::from_secs(60));
        assert_eq!(send(&router, "1.2.3.4").await, StatusCode::OK);
        assert_eq!(send(&router, "1.2.3.4").await, StatusCode::OK);
        assert_eq!(send(&router, "1.2.3.4").await, StatusCode::OK);
    }

    #[tokio::test]
    async fn test_request_exceeding_limit_is_rejected() {
        let router = rate_limited_router(2, Duration::from_secs(60));
        assert_eq!(send(&router, "1.2.3.4").await, StatusCode::OK);
        assert_eq!(send(&router, "1.2.3.4").await, StatusCode::OK);
        assert_eq!(
            send(&router, "1.2.3.4").await,
            StatusCode::TOO_MANY_REQUESTS
        );
    }

    #[tokio::test]
    async fn test_different_ips_have_independent_limits() {
        let router = rate_limited_router(1, Duration::from_secs(60));
        assert_eq!(send(&router, "1.1.1.1").await, StatusCode::OK);
        assert_eq!(send(&router, "2.2.2.2").await, StatusCode::OK);
        assert_eq!(
            send(&router, "1.1.1.1").await,
            StatusCode::TOO_MANY_REQUESTS
        );
        assert_eq!(
            send(&router, "2.2.2.2").await,
            StatusCode::TOO_MANY_REQUESTS
        );
    }

    #[tokio::test]
    async fn test_window_expiry_allows_new_requests() {
        let router = rate_limited_router(1, Duration::from_millis(50));
        assert_eq!(send(&router, "1.2.3.4").await, StatusCode::OK);
        assert_eq!(
            send(&router, "1.2.3.4").await,
            StatusCode::TOO_MANY_REQUESTS
        );
        tokio::time::sleep(Duration::from_millis(60)).await;
        assert_eq!(send(&router, "1.2.3.4").await, StatusCode::OK);
    }

    #[tokio::test]
    async fn test_rate_limit_response_body_is_json_with_code_429() {
        let router = rate_limited_router(0, Duration::from_secs(60));
        let req = Request::builder()
            .uri("/test")
            .header("x-forwarded-for", "1.2.3.4")
            .body(Body::empty())
            .unwrap();
        let resp = router.oneshot(req).await.unwrap();
        assert_eq!(resp.status(), StatusCode::TOO_MANY_REQUESTS);
        let ct = resp
            .headers()
            .get("content-type")
            .and_then(|v| v.to_str().ok())
            .unwrap_or("");
        assert!(ct.contains("application/json"));

        let bytes = resp.into_body().collect().await.unwrap().to_bytes();
        let body: serde_json::Value = serde_json::from_slice(&bytes).unwrap();
        assert_eq!(body["success"], false);
        assert_eq!(body["error"]["code"], 429);
    }

    #[tokio::test]
    async fn test_rate_limit_response_has_retry_after_header() {
        let router = rate_limited_router(0, Duration::from_secs(60));
        let req = Request::builder()
            .uri("/test")
            .header("x-forwarded-for", "1.2.3.4")
            .body(Body::empty())
            .unwrap();
        let resp = router.oneshot(req).await.unwrap();
        assert!(resp.headers().contains_key("retry-after"));
    }

    #[tokio::test]
    async fn test_rapid_requests_trigger_429_integration() {
        // Simulates a burst: max=3 within a long window → 4th request is 429.
        let router = rate_limited_router(3, Duration::from_secs(60));
        for _ in 0..3 {
            assert_eq!(send(&router, "9.9.9.9").await, StatusCode::OK);
        }
        assert_eq!(
            send(&router, "9.9.9.9").await,
            StatusCode::TOO_MANY_REQUESTS
        );
    }

    #[test]
    fn test_rate_limit_from_env_defaults() {
        temp_env::with_vars(
            [
                ("RATE_LIMIT_MAX", None::<&str>),
                ("RATE_LIMIT_WINDOW", None::<&str>),
            ],
            || {
                let (max, window) = rate_limit_from_env();
                assert_eq!(max, 100);
                assert_eq!(window, Duration::from_secs(60));
            },
        );
    }

    #[test]
    fn test_rate_limit_from_env_custom() {
        temp_env::with_vars(
            [
                ("RATE_LIMIT_MAX", Some("25")),
                ("RATE_LIMIT_WINDOW", Some("30")),
            ],
            || {
                let (max, window) = rate_limit_from_env();
                assert_eq!(max, 25);
                assert_eq!(window, Duration::from_secs(30));
            },
        );
    }
}
