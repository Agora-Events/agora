//! # Exchange Rate Handlers
//!
//! Fetches XLM exchange rates from an external provider and caches the
//! result in Redis so that repeated requests within the TTL window don't
//! hit the provider (and its rate limits) again.
//!
//! ## Circuit Breaker
//!
//! After `CIRCUIT_BREAKER_THRESHOLD` consecutive upstream failures the circuit
//! breaker *opens* and blocks new upstream requests for `CIRCUIT_BREAKER_RESET_SECS`
//! seconds.  During the open state:
//!
//! - Stale cached rates (if any) are served with an `X-Rate-Source: stale-cache` header.
//! - If no stale value exists, a `503` error is returned.
//!
//! Circuit breaker state is visible on `GET /api/v1/health` via
//! [`circuit_breaker_health`].

use axum::{
    extract::{Query, State},
    http::HeaderValue,
    response::{IntoResponse, Response},
};
use serde::{Deserialize, Serialize};
use std::{
    collections::HashMap,
    sync::{
        atomic::{AtomicU64, AtomicUsize, Ordering},
        Arc,
    },
    time::{Duration, SystemTime, UNIX_EPOCH},
};

use crate::cache::RedisCache;
use crate::utils::error::AppError;
use crate::utils::response::success;

// ──────────────────────────────────────────────────────────────────────────────
// Constants
// ──────────────────────────────────────────────────────────────────────────────

/// TTL for *fresh* cached live rates.
const CACHE_TTL: Duration = Duration::from_secs(60);

/// Stale-cache TTL: how long a stale value is kept for fallback.
const STALE_CACHE_TTL: Duration = Duration::from_secs(3600);

const DEFAULT_BASE_CURRENCY: &str = "XLM";
const DEFAULT_QUOTE_CURRENCY: &str = "USD";

/// Number of consecutive upstream failures that open the circuit breaker.
const CIRCUIT_BREAKER_THRESHOLD: usize = 3;

/// How long (seconds) the breaker stays open before the next probe attempt.
const CIRCUIT_BREAKER_RESET_SECS: u64 = 60;

/// Base delay (milliseconds) for the first exponential back-off retry.
const BACKOFF_BASE_MS: u64 = 200;

/// Total upstream attempts (1 initial + 2 retries).
const MAX_ATTEMPTS: u32 = 3;

// ──────────────────────────────────────────────────────────────────────────────
// Circuit Breaker
// ──────────────────────────────────────────────────────────────────────────────

/// Atomic circuit-breaker state shared across all request handlers.
#[derive(Debug, Default)]
pub struct CircuitBreaker {
    /// Number of consecutive upstream failures.
    failures: AtomicUsize,
    /// Unix timestamp (secs) at which the circuit was opened.  Zero means
    /// the breaker is *closed*.
    opened_at: AtomicU64,
}

impl CircuitBreaker {
    pub fn new() -> Self {
        Self::default()
    }

    /// Returns `true` when the circuit breaker is currently open (blocking
    /// upstream calls).
    pub fn is_open(&self) -> bool {
        let opened_at = self.opened_at.load(Ordering::Acquire);
        if opened_at == 0 {
            return false;
        }
        let now = unix_now();
        if now.saturating_sub(opened_at) >= CIRCUIT_BREAKER_RESET_SECS {
            // Reset window has elapsed — allow one probe request through.
            self.opened_at.store(0, Ordering::Release);
            self.failures.store(0, Ordering::Release);
            false
        } else {
            true
        }
    }

    /// Record a successful upstream call: resets the failure counter and
    /// closes the circuit.
    pub fn record_success(&self) {
        self.failures.store(0, Ordering::Release);
        self.opened_at.store(0, Ordering::Release);
    }

    /// Record a failed upstream call.  Opens the circuit breaker once the
    /// failure threshold is reached.
    pub fn record_failure(&self) {
        let prev = self.failures.fetch_add(1, Ordering::AcqRel);
        let new_count = prev + 1;
        if new_count >= CIRCUIT_BREAKER_THRESHOLD {
            let now = unix_now();
            // CAS: only set `opened_at` when it is currently zero (not already
            // open) to avoid resetting the timer mid-open window.
            let _ = self
                .opened_at
                .compare_exchange(0, now, Ordering::AcqRel, Ordering::Relaxed);
            tracing::warn!(
                failures = new_count,
                reset_in_secs = CIRCUIT_BREAKER_RESET_SECS,
                "Exchange rate circuit breaker opened after {} consecutive upstream failures — \
                 upstream paused for {}s",
                new_count,
                CIRCUIT_BREAKER_RESET_SECS,
            );
        }
    }

    /// Human-readable status string for health endpoints.
    pub fn status(&self) -> &'static str {
        if self.is_open() {
            "open"
        } else {
            "closed"
        }
    }

    pub fn failure_count(&self) -> usize {
        self.failures.load(Ordering::Relaxed)
    }
}

// ──────────────────────────────────────────────────────────────────────────────
// Handler State
// ──────────────────────────────────────────────────────────────────────────────

#[derive(Clone)]
pub struct RatesState {
    pub redis: RedisCache,
    pub http: reqwest::Client,
    pub breaker: Arc<CircuitBreaker>,
}

impl RatesState {
    pub fn new(redis: RedisCache, http: reqwest::Client) -> Self {
        Self {
            redis,
            http,
            breaker: Arc::new(CircuitBreaker::new()),
        }
    }
}

// ──────────────────────────────────────────────────────────────────────────────
// Query / Response types
// ──────────────────────────────────────────────────────────────────────────────

fn default_base() -> String {
    DEFAULT_BASE_CURRENCY.to_string()
}

fn default_quote() -> String {
    DEFAULT_QUOTE_CURRENCY.to_string()
}

fn cache_key(base: &str, quote: &str) -> String {
    format!("rates:{}:{}", base, quote)
}

fn stale_cache_key(base: &str, quote: &str) -> String {
    format!("rates:stale:{}:{}", base, quote)
}

#[derive(Debug, Deserialize)]
pub struct RatesQuery {
    #[serde(default = "default_base")]
    pub base: String,
    #[serde(default = "default_quote")]
    pub quote: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ExchangeRate {
    pub base: String,
    pub quote: String,
    pub rate: f64,
}

#[derive(Debug, Deserialize)]
struct ProviderResponse {
    rates: HashMap<String, f64>,
}

// ──────────────────────────────────────────────────────────────────────────────
// Circuit Breaker Health snapshot
// ──────────────────────────────────────────────────────────────────────────────

#[derive(Debug, Serialize)]
pub struct CircuitBreakerHealth {
    pub status: &'static str,
    pub failure_count: usize,
    pub threshold: usize,
    pub reset_secs: u64,
}

/// Returns a health snapshot of the circuit breaker for inclusion in
/// `GET /api/v1/health` responses.
pub fn circuit_breaker_health(breaker: &CircuitBreaker) -> CircuitBreakerHealth {
    CircuitBreakerHealth {
        status: breaker.status(),
        failure_count: breaker.failure_count(),
        threshold: CIRCUIT_BREAKER_THRESHOLD,
        reset_secs: CIRCUIT_BREAKER_RESET_SECS,
    }
}

// ──────────────────────────────────────────────────────────────────────────────
// Handler
// ──────────────────────────────────────────────────────────────────────────────

/// Get the current exchange rate for a currency pair.
///
/// # Endpoint
/// `GET /api/v1/rates?base=XLM&quote=USD`
///
/// ## Caching strategy
/// 1. Fresh Redis cache (60 s TTL) — returned immediately.
/// 2. Circuit breaker open → stale cache (1 h TTL) or `503`.
/// 3. Upstream fetch with exponential back-off (up to 3 attempts).
/// 4. On failure, fall back to stale cache before returning an error.
pub async fn get_rates(
    State(mut state): State<RatesState>,
    Query(params): Query<RatesQuery>,
) -> Response {
    let base = params.base.to_uppercase();
    let quote = params.quote.to_uppercase();
    let key = cache_key(&base, &quote);
    let stale_key = stale_cache_key(&base, &quote);

    // ── 1. Try fresh Redis cache ──────────────────────────────────────────────
    if let Ok(Some(rate)) = state.redis.get::<ExchangeRate>(&key).await {
        return success(rate, "Exchange rate retrieved from cache").into_response();
    }

    // ── 2. Circuit breaker open → serve stale cache ───────────────────────────
    if state.breaker.is_open() {
        tracing::warn!(
            base = %base,
            quote = %quote,
            "Circuit breaker is open — serving stale cache or returning 503"
        );

        if let Ok(Some(stale_rate)) = state.redis.get::<ExchangeRate>(&stale_key).await {
            let mut resp = success(
                stale_rate,
                "Stale exchange rate served (circuit breaker open)",
            )
            .into_response();
            resp.headers_mut()
                .insert("X-Rate-Source", HeaderValue::from_static("stale-cache"));
            return resp;
        }

        return AppError::ExternalServiceError(
            "Exchange rate provider is unavailable and no cached data exists".to_string(),
        )
        .into_response();
    }

    // ── 3. Upstream fetch with exponential back-off ────────────────────────────
    let provider_url = std::env::var("RATES_PROVIDER_URL")
        .unwrap_or_else(|_| "https://api.exchangerate.host/latest".to_string());

    match fetch_with_backoff(&state.http, &provider_url, &base).await {
        Ok(provider_data) => {
            state.breaker.record_success();

            let rate_value = match provider_data.rates.get(&quote) {
                Some(r) => *r,
                None => {
                    return AppError::NotFound(format!("No rate available for {}/{}", base, quote))
                        .into_response();
                }
            };

            let rate = ExchangeRate {
                base: base.clone(),
                quote: quote.clone(),
                rate: rate_value,
            };

            // Persist both fresh and stale caches.
            let _ = state.redis.set(&key, &rate, CACHE_TTL).await;
            let _ = state.redis.set(&stale_key, &rate, STALE_CACHE_TTL).await;

            success(rate, "Exchange rate retrieved successfully").into_response()
        }

        Err(e) => {
            state.breaker.record_failure();
            tracing::error!(
                error = %e,
                failure_count = state.breaker.failure_count(),
                "Exchange rate upstream fetch failed"
            );

            // Fall back to stale cache even before the breaker has opened.
            if let Ok(Some(stale_rate)) = state.redis.get::<ExchangeRate>(&stale_key).await {
                let mut resp = success(stale_rate, "Stale exchange rate served (upstream error)")
                    .into_response();
                resp.headers_mut()
                    .insert("X-Rate-Source", HeaderValue::from_static("stale-cache"));
                return resp;
            }

            AppError::ExternalServiceError("Unable to reach exchange rate provider".to_string())
                .into_response()
        }
    }
}

// ──────────────────────────────────────────────────────────────────────────────
// Helpers
// ──────────────────────────────────────────────────────────────────────────────

/// Attempt to fetch from the upstream provider with exponential back-off.
///
/// Retries up to `MAX_ATTEMPTS` times, doubling the delay after each failure
/// (200 ms → 400 ms → 800 ms …).  HTTP 429 responses are treated as errors
/// and trigger a back-off.
async fn fetch_with_backoff(
    client: &reqwest::Client,
    provider_url: &str,
    base: &str,
) -> Result<ProviderResponse, String> {
    let mut last_err = String::new();

    for attempt in 0..MAX_ATTEMPTS {
        if attempt > 0 {
            let delay_ms = BACKOFF_BASE_MS * (1u64 << (attempt - 1)); // 200, 400, …
            tracing::info!(
                attempt = attempt + 1,
                delay_ms,
                "Retrying exchange rate upstream fetch after backoff"
            );
            tokio::time::sleep(tokio::time::Duration::from_millis(delay_ms)).await;
        }

        let resp = client
            .get(provider_url)
            .query(&[("base", base)])
            .send()
            .await;

        match resp {
            Ok(r) if r.status() == reqwest::StatusCode::TOO_MANY_REQUESTS => {
                last_err = format!("HTTP 429 rate-limited on attempt {}", attempt + 1);
                tracing::warn!(
                    attempt = attempt + 1,
                    "Exchange rate provider returned 429 — backing off"
                );
            }
            Ok(r) if !r.status().is_success() => {
                last_err = format!("HTTP {} on attempt {}", r.status(), attempt + 1);
                tracing::warn!(
                    status = %r.status(),
                    attempt = attempt + 1,
                    "Exchange rate provider returned non-2xx status"
                );
            }
            Ok(r) => match r.json::<ProviderResponse>().await {
                Ok(data) => return Ok(data),
                Err(e) => {
                    last_err = format!("JSON parse error on attempt {}: {}", attempt + 1, e);
                    tracing::error!(attempt = attempt + 1, error = %e, "Failed to parse exchange rate response");
                }
            },
            Err(e) => {
                last_err = format!("Network error on attempt {}: {}", attempt + 1, e);
                tracing::error!(attempt = attempt + 1, error = %e, "Exchange rate network error");
            }
        }
    }

    Err(last_err)
}

fn unix_now() -> u64 {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map(|d| d.as_secs())
        .unwrap_or(0)
}

// ──────────────────────────────────────────────────────────────────────────────
// Tests
// ──────────────────────────────────────────────────────────────────────────────

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_cache_key_includes_currency_pair() {
        assert_eq!(cache_key("XLM", "USD"), "rates:XLM:USD");
        assert_eq!(cache_key("XLM", "EUR"), "rates:XLM:EUR");
    }

    #[test]
    fn test_stale_cache_key_is_distinct_from_fresh() {
        assert_ne!(cache_key("XLM", "USD"), stale_cache_key("XLM", "USD"));
        assert_eq!(stale_cache_key("XLM", "USD"), "rates:stale:XLM:USD");
    }

    #[test]
    fn test_default_currencies() {
        assert_eq!(default_base(), "XLM");
        assert_eq!(default_quote(), "USD");
    }

    #[test]
    fn test_exchange_rate_round_trips_through_json() {
        let rate = ExchangeRate {
            base: "XLM".to_string(),
            quote: "USD".to_string(),
            rate: 0.11,
        };
        let json = serde_json::to_string(&rate).unwrap();
        let parsed: ExchangeRate = serde_json::from_str(&json).unwrap();
        assert_eq!(parsed.base, "XLM");
        assert_eq!(parsed.quote, "USD");
        assert!((parsed.rate - 0.11).abs() < f64::EPSILON);
    }

    #[test]
    fn test_circuit_breaker_opens_after_threshold() {
        let cb = CircuitBreaker::new();
        assert!(!cb.is_open(), "breaker should start closed");
        for _ in 0..CIRCUIT_BREAKER_THRESHOLD {
            cb.record_failure();
        }
        assert!(cb.is_open(), "breaker should open after threshold failures");
    }

    #[test]
    fn test_circuit_breaker_resets_on_success() {
        let cb = CircuitBreaker::new();
        for _ in 0..CIRCUIT_BREAKER_THRESHOLD {
            cb.record_failure();
        }
        assert!(cb.is_open());
        cb.record_success();
        assert!(!cb.is_open(), "breaker should close after success");
        assert_eq!(cb.failure_count(), 0);
    }

    #[test]
    fn test_circuit_breaker_does_not_open_below_threshold() {
        let cb = CircuitBreaker::new();
        for _ in 0..(CIRCUIT_BREAKER_THRESHOLD - 1) {
            cb.record_failure();
        }
        assert!(
            !cb.is_open(),
            "breaker should remain closed below threshold"
        );
    }

    #[test]
    fn test_circuit_breaker_health_closed() {
        let cb = CircuitBreaker::new();
        let h = circuit_breaker_health(&cb);
        assert_eq!(h.status, "closed");
        assert_eq!(h.failure_count, 0);
        assert_eq!(h.threshold, CIRCUIT_BREAKER_THRESHOLD);
        assert_eq!(h.reset_secs, CIRCUIT_BREAKER_RESET_SECS);
    }

    #[test]
    fn test_circuit_breaker_health_open() {
        let cb = CircuitBreaker::new();
        for _ in 0..CIRCUIT_BREAKER_THRESHOLD {
            cb.record_failure();
        }
        let h = circuit_breaker_health(&cb);
        assert_eq!(h.status, "open");
    }
}
