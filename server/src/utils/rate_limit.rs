//! Per-IP sliding-window rate limiter implemented as a Tower [`Layer`].
//!
//! Each unique client IP is allowed at most `max_requests` within a rolling
//! `window` duration.  Requests that exceed the limit receive a
//! `429 Too Many Requests` response immediately, without forwarding to the
//! inner service.
//!
//! # Usage
//! ```rust,ignore
//! use std::time::Duration;
//! use crate::utils::rate_limit::RateLimitLayer;
//!
//! let layer = RateLimitLayer::new(60, Duration::from_secs(60));
//! ```

use std::{
    collections::VecDeque,
    net::IpAddr,
    sync::{Arc, Mutex},
    task::{Context, Poll},
    time::{Duration, Instant},
};

use axum::{
    body::Body,
    http::{Request, Response, StatusCode},
};
use serde_json::json;
use std::collections::HashMap;
use tower::{Layer, Service};

// ---------------------------------------------------------------------------
// State shared across all requests
// ---------------------------------------------------------------------------

#[derive(Clone)]
struct IpState {
    /// Timestamps of requests within the current window.
    timestamps: VecDeque<Instant>,
}

impl IpState {
    fn new() -> Self {
        Self {
            timestamps: VecDeque::new(),
        }
    }

    /// Prune entries older than `window`, then check whether a new request is
    /// allowed.  Returns `true` if the request should proceed.
    fn check_and_record(&mut self, max_requests: usize, window: Duration) -> bool {
        let now = Instant::now();
        let cutoff = now - window;

        // Remove timestamps outside the sliding window
        while self.timestamps.front().is_some_and(|t| *t < cutoff) {
            self.timestamps.pop_front();
        }

        if self.timestamps.len() < max_requests {
            self.timestamps.push_back(now);
            true
        } else {
            false
        }
    }
}

type Store = Arc<Mutex<HashMap<IpAddr, IpState>>>;

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
    /// * `max_requests` – maximum number of requests allowed per IP per window.
    /// * `window`       – length of the sliding window.
    pub fn new(max_requests: usize, window: Duration) -> Self {
        Self {
            max_requests,
            window,
            store: Arc::new(Mutex::new(HashMap::new())),
        }
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

        let allowed = {
            let mut store = self.store.lock().unwrap();
            let state = store.entry(ip).or_insert_with(IpState::new);
            state.check_and_record(self.max_requests, self.window)
        };

        if !allowed {
            let body = json!({
                "success": false,
                "error": {
                    "code": "RATE_LIMIT_EXCEEDED",
                    "message": "Too many requests. Please try again later."
                }
            })
            .to_string();

            let response = Response::builder()
                .status(StatusCode::TOO_MANY_REQUESTS)
                .header("Content-Type", "application/json")
                .header("Retry-After", "60")
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
    // X-Forwarded-For: client, proxy1, proxy2
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

    // X-Real-IP
    if let Some(real_ip) = req
        .headers()
        .get("x-real-ip")
        .and_then(|v| v.to_str().ok())
        .and_then(|s| s.trim().parse().ok())
    {
        return real_ip;
    }

    // Axum stores the peer address via `axum::extract::ConnectInfo`
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
        // Both IPs have now used their 1 request; next from each should be blocked
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
        // Wait for the window to expire
        tokio::time::sleep(Duration::from_millis(60)).await;
        assert_eq!(send(&router, "1.2.3.4").await, StatusCode::OK);
    }

    #[tokio::test]
    async fn test_rate_limit_response_body_is_json() {
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
}
