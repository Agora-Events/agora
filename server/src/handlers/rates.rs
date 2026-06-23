//! # Currency Rates Handler
//!
//! Provides real-time USDC-to-fiat conversion rates for the frontend.
//!
//! ## Endpoint
//! `GET /api/v1/rates` — returns the latest conversion factors relative to USDC.
//!
//! ## Data Source
//! Rates are fetched from the CoinGecko public API and cached in Redis for
//! 60 seconds to avoid hammering the upstream provider.
//!
//! ## Response Example
//! ```json
//! {
//!   "success": true,
//!   "data": {
//!     "base": "USDC",
//!     "rates": {
//!       "USD": 1.0,
//!       "NGN": 1550.0,
//!       "EUR": 0.92,
//!       "GBP": 0.79,
//!       "KES": 129.5
//!     },
//!     "fetched_at": "2026-05-01T12:00:00Z"
//!   }
//! }
//! ```

use axum::{extract::State, response::IntoResponse, response::Response};
use chrono::Utc;
use serde::{Deserialize, Serialize};
use serde_json::Value;
use std::collections::HashMap;
use std::time::Duration;

use crate::cache::RedisCache;
use crate::utils::error::AppError;
use crate::utils::response::success;

/// Cache TTL for exchange rates (5 minutes).
const RATES_CACHE_TTL: Duration = Duration::from_secs(300);
const RATES_CACHE_KEY: &str = "rates:usdc";

/// Cache key and TTL for stale fallback rates (24 hours).
const RATES_STALE_CACHE_KEY: &str = "rates:usdc:stale";
const RATES_STALE_CACHE_TTL: Duration = Duration::from_secs(86400);

/// Currencies to fetch from CoinGecko (vs USDC).
/// USDC is pegged 1:1 to USD, so we fetch USD rates and treat them as USDC rates.
const TARGET_CURRENCIES: &[&str] = &[
    "usd", "ngn", "eur", "gbp", "kes", "ghs", "zar", "cad", "aud", "jpy",
];

/// Application state for the rates handler.
#[derive(Clone)]
pub struct RatesState {
    pub redis: RedisCache,
    pub http: reqwest::Client,
}

/// The response body returned to clients.
#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct RatesResponse {
    /// The base currency (always "USDC").
    pub base: String,
    /// Map of currency code (uppercase) → units per 1 USDC.
    pub rates: HashMap<String, f64>,
    /// ISO 8601 timestamp of when the rates were fetched.
    pub fetched_at: String,
}

/// CoinGecko simple/price response shape.
#[allow(dead_code)]
#[derive(Debug, Deserialize)]
struct CoinGeckoPrice {
    #[serde(flatten)]
    prices: HashMap<String, Value>,
}

/// `GET /api/v1/rates`
///
/// Returns USDC conversion rates for a set of fiat currencies.
/// Results are cached in Redis for 5 minutes.
pub async fn get_rates(State(mut state): State<RatesState>) -> Response {
    // 1. Try cache first
    match state.redis.get::<RatesResponse>(RATES_CACHE_KEY).await {
        Ok(Some(cached)) => {
            tracing::debug!("Cache hit for currency rates");
            return success(cached, "Rates retrieved (cached)").into_response();
        }
        Ok(None) => tracing::debug!("Cache miss for currency rates"),
        Err(e) => tracing::warn!("Redis error fetching rates, falling back to API: {:?}", e),
    }

    // 2. Fetch from CoinGecko
    let vs_currencies = TARGET_CURRENCIES.join(",");
    let url = format!(
        "https://api.coingecko.com/api/v3/simple/price?ids=usd-coin&vs_currencies={vs_currencies}"
    );

    let fetch_result = async {
        let api_response = state
            .http
            .get(&url)
            .timeout(Duration::from_secs(5))
            .send()
            .await
            .map_err(|e| {
                tracing::error!("CoinGecko request failed: {:?}", e);
                AppError::ExternalServiceError(
                    "Failed to fetch exchange rates from provider".to_string(),
                )
            })?;

        if !api_response.status().is_success() {
            tracing::error!("CoinGecko returned status {}", api_response.status());
            return Err(AppError::ExternalServiceError(format!(
                "Exchange rate provider returned status {}",
                api_response.status()
            )));
        }

        let body: Value = api_response.json().await.map_err(|e| {
            tracing::error!("Failed to parse CoinGecko response: {:?}", e);
            AppError::ExternalServiceError("Failed to parse exchange rate response".to_string())
        })?;

        // CoinGecko returns: { "usd-coin": { "usd": 1.0, "ngn": 1550.0, ... } }
        let coin_rates = body
            .get("usd-coin")
            .and_then(|v| v.as_object())
            .ok_or_else(|| {
                tracing::error!("Unexpected CoinGecko response shape: {:?}", body);
                AppError::ExternalServiceError(
                    "Unexpected response shape from exchange rate provider".to_string(),
                )
            })?;

        let mut rates: HashMap<String, f64> = HashMap::new();
        for (currency, value) in coin_rates {
            if let Some(rate) = value.as_f64() {
                rates.insert(currency.to_uppercase(), rate);
            }
        }

        // Ensure USD is always present (USDC ≈ 1 USD)
        rates.entry("USD".to_string()).or_insert(1.0);

        Ok(RatesResponse {
            base: "USDC".to_string(),
            rates,
            fetched_at: Utc::now().to_rfc3339(),
        })
    }
    .await;

    match fetch_result {
        Ok(response) => {
            // 3. Cache the result
            if let Err(e) = state
                .redis
                .set(RATES_CACHE_KEY, &response, RATES_CACHE_TTL)
                .await
            {
                tracing::warn!("Failed to cache currency rates: {:?}", e);
            }
            if let Err(e) = state
                .redis
                .set(RATES_STALE_CACHE_KEY, &response, RATES_STALE_CACHE_TTL)
                .await
            {
                tracing::warn!("Failed to cache stale currency rates: {:?}", e);
            }

            success(response, "Rates retrieved successfully").into_response()
        }
        Err(err) => {
            // Try fallback
            match state
                .redis
                .get::<RatesResponse>(RATES_STALE_CACHE_KEY)
                .await
            {
                Ok(Some(stale)) => {
                    tracing::warn!("Serving stale exchange rates from cache due to API error");
                    let mut resp =
                        success(stale, "Rates retrieved (stale fallback)").into_response();
                    resp.headers_mut().insert(
                        axum::http::header::CACHE_CONTROL,
                        axum::http::HeaderValue::from_static("stale"),
                    );
                    resp
                }
                _ => err.into_response(),
            }
        }
    }
}

// ---------------------------------------------------------------------------
// Tests
// ---------------------------------------------------------------------------

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_rates_response_serialization() {
        let mut rates = HashMap::new();
        rates.insert("USD".to_string(), 1.0_f64);
        rates.insert("NGN".to_string(), 1550.0_f64);

        let resp = RatesResponse {
            base: "USDC".to_string(),
            rates,
            fetched_at: "2026-05-01T12:00:00Z".to_string(),
        };

        let json = serde_json::to_string(&resp).unwrap();
        assert!(json.contains("USDC"));
        assert!(json.contains("NGN"));
    }

    #[test]
    fn test_target_currencies_not_empty() {
        assert!(!TARGET_CURRENCIES.is_empty());
        assert!(TARGET_CURRENCIES.contains(&"usd"));
        assert!(TARGET_CURRENCIES.contains(&"ngn"));
    }

    #[tokio::test]
    async fn test_rates_stale_fallback_header() {
        use axum::http::header::CACHE_CONTROL;
        let mut rates = HashMap::new();
        rates.insert("USD".to_string(), 1.0_f64);
        let stale = RatesResponse {
            base: "USDC".to_string(),
            rates,
            fetched_at: "2026-05-01T12:00:00Z".to_string(),
        };
        let mut resp = success(stale, "Rates retrieved (stale fallback)").into_response();
        resp.headers_mut()
            .insert(CACHE_CONTROL, axum::http::HeaderValue::from_static("stale"));

        assert_eq!(resp.status(), axum::http::StatusCode::OK);
        let cc = resp.headers().get(CACHE_CONTROL).unwrap().to_str().unwrap();
        assert_eq!(cc, "stale");
    }
}
