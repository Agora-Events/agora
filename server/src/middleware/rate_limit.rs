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
