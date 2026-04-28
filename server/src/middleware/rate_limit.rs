//! # Tower-Governor Rate Limiting Middleware
//!
//! Provides IP-based rate limiting using tower-governor for public API routes.

use axum::{
    body::Body,
    extract::ConnectInfo,
    http::{Request, StatusCode},
    response::{IntoResponse, Response},
};
use std::{net::SocketAddr, sync::Arc, time::Duration};
use tower::{Layer, Service};
use tower_governor::{
    governor::GovernorConfigBuilder, key_extractor::SmartIpKeyExtractor, GovernorLayer,
};

/// Rate limit layer using tower-governor
#[derive(Clone)]
pub struct GovernorRateLimitLayer {
    inner: GovernorLayer<SmartIpKeyExtractor>,
}

impl GovernorRateLimitLayer {
    /// Create a new rate limit layer
    ///
    /// # Arguments
    /// * `requests_per_minute` - Maximum number of requests allowed per minute
    /// * `window` - Time window for rate limiting
    pub fn new(requests_per_minute: u64, window: Duration) -> Self {
        let config = Box::new(
            GovernorConfigBuilder::default()
                .per_second(requests_per_minute / 60)
                .burst_size(requests_per_minute as u32)
                .finish()
                .unwrap(),
        );

        let governor_limiter = Arc::new(config);
        let layer = GovernorLayer {
            config: governor_limiter,
        };

        Self { inner: layer }
    }
}

impl<S> Layer<S> for GovernorRateLimitLayer {
    type Service = <GovernorLayer<SmartIpKeyExtractor> as Layer<S>>::Service;

    fn layer(&self, inner: S) -> Self::Service {
        self.inner.layer(inner)
    }
}
