//! # Exchange Rate Handlers
//!
//! Fetches XLM exchange rates from an external provider and caches the
//! result in Redis so that repeated requests within the TTL window don't
//! hit the provider (and its rate limits) again.

use axum::{
    extract::Query,
    response::{IntoResponse, Response},
};
use redis::AsyncCommands;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::sync::OnceLock;

use crate::utils::error::AppError;
use crate::utils::response::success;

/// TTL for cached live rates.
const CACHE_TTL_SECONDS: u64 = 60;
const DEFAULT_BASE_CURRENCY: &str = "XLM";
const DEFAULT_QUOTE_CURRENCY: &str = "USD";

static REDIS_CLIENT: OnceLock<Option<redis::Client>> = OnceLock::new();

/// Lazily builds a Redis client from `REDIS_URL`. Returns `None` (and the
/// handler falls back to always fetching from the provider) if the client
/// can't be constructed, e.g. because `REDIS_URL` isn't configured.
fn redis_client() -> Option<&'static redis::Client> {
    REDIS_CLIENT
        .get_or_init(|| {
            std::env::var("REDIS_URL")
                .ok()
                .and_then(|url| redis::Client::open(url).ok())
        })
        .as_ref()
}

fn default_base() -> String {
    DEFAULT_BASE_CURRENCY.to_string()
}

fn default_quote() -> String {
    DEFAULT_QUOTE_CURRENCY.to_string()
}

fn cache_key(base: &str, quote: &str) -> String {
    format!("rates:{}:{}", base, quote)
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

/// Get the current exchange rate for a currency pair, serving from a Redis
/// cache when available.
///
/// # Endpoint
/// GET `/api/v1/rates?base=XLM&quote=USD`
pub async fn get_rates(Query(params): Query<RatesQuery>) -> Response {
    let base = params.base.to_uppercase();
    let quote = params.quote.to_uppercase();
    let key = cache_key(&base, &quote);

    if let Some(client) = redis_client() {
        if let Ok(mut conn) = client.get_multiplexed_async_connection().await {
            let cached: Option<String> = conn.get(&key).await.unwrap_or(None);
            if let Some(raw) = cached {
                if let Ok(rate) = serde_json::from_str::<ExchangeRate>(&raw) {
                    return success(rate, "Exchange rate retrieved from cache").into_response();
                }
            }
        }
    }

    let provider_url = std::env::var("RATES_PROVIDER_URL")
        .unwrap_or_else(|_| "https://api.exchangerate.host/latest".to_string());

    let provider_response = match reqwest::Client::new()
        .get(&provider_url)
        .query(&[("base", base.as_str())])
        .send()
        .await
    {
        Ok(resp) => resp,
        Err(e) => {
            tracing::error!("Failed to reach exchange rate provider: {:?}", e);
            return AppError::ExternalServiceError(
                "Unable to reach exchange rate provider".to_string(),
            )
            .into_response();
        }
    };

    let provider_data = match provider_response.json::<ProviderResponse>().await {
        Ok(data) => data,
        Err(e) => {
            tracing::error!("Failed to parse exchange rate provider response: {:?}", e);
            return AppError::ExternalServiceError(
                "Invalid response from exchange rate provider".to_string(),
            )
            .into_response();
        }
    };

    let rate_value = match provider_data.rates.get(&quote) {
        Some(rate) => *rate,
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

    if let Some(client) = redis_client() {
        if let Ok(mut conn) = client.get_multiplexed_async_connection().await {
            if let Ok(json) = serde_json::to_string(&rate) {
                let _: Result<(), _> = conn.set_ex(&key, json, CACHE_TTL_SECONDS).await;
            }
        }
    }

    success(rate, "Exchange rate retrieved successfully").into_response()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_cache_key_includes_currency_pair() {
        assert_eq!(cache_key("XLM", "USD"), "rates:XLM:USD");
        assert_eq!(cache_key("XLM", "EUR"), "rates:XLM:EUR");
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
}
