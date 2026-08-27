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
    http::{header::HeaderValue, HeaderMap, Request, Response, StatusCode},
    response::IntoResponse,
};
use dashmap::DashMap;
use tower::{Layer, Service};

use crate::utils::error::ApiError;

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
    fn try_acquire(&mut self, capacity: f64, refill_per_sec: f64) -> AcquireOutcome {
        let now = Instant::now();
        let elapsed = now.duration_since(self.last_refill).as_secs_f64();
        self.tokens = (self.tokens + elapsed * refill_per_sec).min(capacity);
        self.last_refill = now;

        if self.tokens >= 1.0 {
            self.tokens -= 1.0;
            let secs_until_full = if refill_per_sec > 0.0 {
                ((capacity - self.tokens) / refill_per_sec).ceil() as u64
            } else {
                1
            };
            AcquireOutcome {
                allowed: true,
                remaining: self.tokens.floor() as u64,
                retry_after: secs_until_full.max(1),
            }
        } else {
            let retry_after = if refill_per_sec > 0.0 {
                ((1.0 - self.tokens) / refill_per_sec).ceil() as u64
            } else {
                1
            };
            AcquireOutcome {
                allowed: false,
                remaining: 0,
                retry_after: retry_after.max(1),
            }
        }
    }
}

struct AcquireOutcome {
    allowed: bool,
    remaining: u64,
    retry_after: u64,
}

/// Apply standard rate-limit headers. `Retry-After` is set only on 429s.
pub fn apply_rate_limit_headers(
    headers: &mut HeaderMap,
    limit: usize,
    remaining: u64,
    reset_epoch: u64,
    retry_after: Option<u64>,
) {
    if let Ok(v) = HeaderValue::from_str(&limit.to_string()) {
        headers.insert("x-ratelimit-limit", v);
    }
    if let Ok(v) = HeaderValue::from_str(&remaining.to_string()) {
        headers.insert("x-ratelimit-remaining", v);
    }
    if let Ok(v) = HeaderValue::from_str(&reset_epoch.to_string()) {
        headers.insert("x-ratelimit-reset", v);
    }
    if let Some(secs) = retry_after {
        let value = secs.max(1);
        if let Ok(v) = HeaderValue::from_str(&value.to_string()) {
            headers.insert("retry-after", v);
        }
    }
}

fn unix_now() -> u64 {
    std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .map(|d| d.as_secs())
        .unwrap_or(0)
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
        let window_secs = self.window.as_secs().max(1);
        let capacity = self.max_requests as f64;
        let refill_per_sec = if self.max_requests == 0 {
            0.0
        } else {
            capacity / self.window.as_secs_f64().max(0.001)
        };

        let outcome = if self.max_requests == 0 {
            AcquireOutcome {
                allowed: false,
                remaining: 0,
                retry_after: window_secs,
            }
        } else {
            let mut entry = self
                .store
                .entry(ip)
                .or_insert_with(|| TokenBucket::new(capacity));
            entry.try_acquire(capacity, refill_per_sec)
        };

        let limit = self.max_requests;
        let remaining = outcome.remaining;
        let retry_after = outcome.retry_after.max(1);
        let reset_epoch = unix_now().saturating_add(retry_after);

        if !outcome.allowed {
            let mut response = ApiError::with_code(
                crate::utils::error::ErrorCode::RateLimited,
                StatusCode::TOO_MANY_REQUESTS,
                "Too many requests. Please try again later.",
            )
            .into_response();
            apply_rate_limit_headers(
                response.headers_mut(),
                limit,
                remaining,
                reset_epoch,
                Some(retry_after),
            );
            return Box::pin(async move { Ok(response) });
        }

        let future = self.inner.call(req);
        Box::pin(async move {
            let mut response = future.await?;
            apply_rate_limit_headers(response.headers_mut(), limit, remaining, reset_epoch, None);
            Ok(response)
        })
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
        send_full(router, ip).await.status()
    }

    async fn send_full(router: &Router, ip: &str) -> axum::http::Response<Body> {
        let req = Request::builder()
            .uri("/test")
            .header("x-forwarded-for", ip)
            .body(Body::empty())
            .unwrap();
        router.clone().oneshot(req).await.unwrap()
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
        assert_eq!(body["code"], "RATE_LIMITED");
        assert!(body["message"].is_string());
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
        let secs: u64 = resp
            .headers()
            .get("retry-after")
            .and_then(|v| v.to_str().ok())
            .and_then(|s| s.parse().ok())
            .unwrap_or(0);
        assert!(secs >= 1, "Retry-After must be at least 1, got {secs}");
        assert!(resp.headers().contains_key("x-ratelimit-limit"));
        assert!(resp.headers().contains_key("x-ratelimit-remaining"));
        assert!(resp.headers().contains_key("x-ratelimit-reset"));
    }

    #[tokio::test]
    async fn test_exhausted_limit_retry_after_is_present_and_positive() {
        let router = rate_limited_router(1, Duration::from_secs(60));
        let allowed = send_full(&router, "8.8.8.8").await;
        assert_eq!(allowed.status(), StatusCode::OK);
        assert_eq!(
            allowed.headers().get("x-ratelimit-limit").unwrap(),
            "1"
        );
        assert!(allowed.headers().contains_key("x-ratelimit-remaining"));
        assert!(allowed.headers().contains_key("x-ratelimit-reset"));
        assert!(allowed.headers().get("retry-after").is_none());

        let rejected = send_full(&router, "8.8.8.8").await;
        assert_eq!(rejected.status(), StatusCode::TOO_MANY_REQUESTS);
        let secs: u64 = rejected
            .headers()
            .get("retry-after")
            .and_then(|v| v.to_str().ok())
            .and_then(|s| s.parse().ok())
            .expect("Retry-After header");
        assert!(secs >= 1);
        assert_eq!(
            rejected.headers().get("x-ratelimit-remaining").unwrap(),
            "0"
        );
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
