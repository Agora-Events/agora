//! # Rate Limiting Middleware Stub
//!
//! `tower_governor` 0.8 requires axum 0.8, which is incompatible with the
//! current axum 0.7 dependency. Rate limiting for public API routes is handled
//! by the `RateLimitLayer` in `utils::rate_limit` instead.
//!
//! This module provides a no-op `GovernorRateLimitLayer` that passes requests
//! through unchanged, preserving the existing call sites in `routes/mod.rs`.

use std::time::Duration;
use tower::Layer;

/// No-op rate limit layer (placeholder until axum is upgraded to 0.8).
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
