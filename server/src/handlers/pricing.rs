//! # Dynamic Pricing Handler
//!
//! Provides real-time pricing projections and curve visualisations for Dutch
//! Auctions and Bancor Bonding Curves.  These endpoints are consumed by the
//! web and mobile frontends to render live price-curve charts and give buyers
//! transparent pricing information before they commit to a purchase.
//!
//! ## Endpoints
//!
//! | Method | Path                                              | Description                                  |
//! |--------|---------------------------------------------------|----------------------------------------------|
//! | GET    | `/api/v1/pricing/dutch-auction`                   | Compute current & projected Dutch auction prices |
//! | GET    | `/api/v1/pricing/bonding-curve`                   | Compute bonding curve price at a given supply |
//! | GET    | `/api/v1/pricing/bonding-curve/series`            | Generate a price series for curve visualisation |
//!
//! ## Caching
//!
//! All responses are cached in Redis for `PRICING_CACHE_TTL` seconds.
//! The cache key encodes all relevant query parameters so different supply
//! levels / time windows never collide.

use axum::{
    extract::{Query, State},
    response::{IntoResponse, Response},
};
use serde::{Deserialize, Serialize};
use std::time::Duration;

use crate::cache::RedisCache;
use crate::utils::error::AppError;
use crate::utils::response::success;

// ──────────────────────────────────────────────────────────────────────────────
// Constants
// ──────────────────────────────────────────────────────────────────────────────

/// TTL for cached pricing projections.
const PRICING_CACHE_TTL: Duration = Duration::from_secs(15);

/// Number of data points returned in a price series (for chart rendering).
const DEFAULT_SERIES_POINTS: usize = 50;

/// Maximum data points allowed in a single series request.
const MAX_SERIES_POINTS: usize = 200;

// ──────────────────────────────────────────────────────────────────────────────
// Handler State
// ──────────────────────────────────────────────────────────────────────────────

/// Shared state for the pricing handler.
#[derive(Clone)]
pub struct PricingState {
    pub redis: RedisCache,
}

impl PricingState {
    pub fn new(redis: RedisCache) -> Self {
        Self { redis }
    }
}

// ──────────────────────────────────────────────────────────────────────────────
// Dutch Auction
// ──────────────────────────────────────────────────────────────────────────────

/// Query parameters for the Dutch auction price endpoint.
#[derive(Debug, Deserialize)]
pub struct DutchAuctionQuery {
    /// Auction starting price in stroops.
    pub start_price: i64,
    /// Auction reserve (floor) price in stroops.
    pub reserve_price: i64,
    /// Unix timestamp (seconds) when the auction starts.
    pub start_time: u64,
    /// Unix timestamp (seconds) when the auction ends.
    pub end_time: u64,
    /// Current Unix timestamp (seconds) to evaluate the price at.
    /// Defaults to `now` if omitted.
    pub current_time: Option<u64>,
    /// If `true`, use exponential decay instead of linear.
    #[serde(default)]
    pub exponential: bool,
    /// Number of series points to include in the projection.
    /// Defaults to `DEFAULT_SERIES_POINTS`.
    pub points: Option<usize>,
}

/// A single (timestamp, price) sample in the projected price series.
#[derive(Debug, Serialize, Deserialize)]
pub struct PricePoint {
    /// Unix timestamp in seconds.
    pub timestamp: u64,
    /// Price in stroops at `timestamp`.
    pub price_stroops: i64,
    /// Human-readable price in USDC (price_stroops / 10_000_000).
    pub price_usdc: f64,
}

/// Response for `GET /api/v1/pricing/dutch-auction`.
#[derive(Debug, Serialize, Deserialize)]
pub struct DutchAuctionPriceResponse {
    /// Current price at `query.current_time`.
    pub current_price_stroops: i64,
    pub current_price_usdc: f64,
    /// Price series from `start_time` to `end_time` for chart rendering.
    pub series: Vec<PricePoint>,
    /// Seconds remaining until the auction ends (0 if already ended).
    pub seconds_remaining: u64,
    /// Whether exponential decay was applied.
    pub exponential: bool,
}

/// GET `/api/v1/pricing/dutch-auction`
///
/// Returns the current Dutch auction price and a projection series suitable
/// for rendering a live price-decay chart on the frontend.
pub async fn get_dutch_auction_price(
    State(mut state): State<PricingState>,
    Query(params): Query<DutchAuctionQuery>,
) -> Response {
    // Basic validation.
    if params.start_price <= 0
        || params.reserve_price <= 0
        || params.start_price <= params.reserve_price
    {
        return AppError::ValidationError(
            "start_price must be greater than reserve_price and both must be positive".to_string(),
        )
        .into_response();
    }
    if params.end_time <= params.start_time {
        return AppError::ValidationError("end_time must be greater than start_time".to_string())
            .into_response();
    }

    let now = params.current_time.unwrap_or_else(unix_now);

    // Cache lookup.
    let cache_key = format!(
        "pricing:dutch:{}:{}:{}:{}:{}:{}",
        params.start_price,
        params.reserve_price,
        params.start_time,
        params.end_time,
        params.exponential,
        now / 15, // 15-second bucket
    );

    if let Ok(Some(cached)) = state
        .redis
        .get::<DutchAuctionPriceResponse>(&cache_key)
        .await
    {
        return success(cached, "Dutch auction price (cached)").into_response();
    }

    let current_price = compute_dutch_price(
        params.start_price,
        params.reserve_price,
        params.start_time,
        params.end_time,
        now,
        params.exponential,
    );

    let points = params
        .points
        .unwrap_or(DEFAULT_SERIES_POINTS)
        .clamp(2, MAX_SERIES_POINTS);

    let series = build_dutch_series(
        params.start_price,
        params.reserve_price,
        params.start_time,
        params.end_time,
        points,
        params.exponential,
    );

    let seconds_remaining = params.end_time.saturating_sub(now);

    let response = DutchAuctionPriceResponse {
        current_price_stroops: current_price,
        current_price_usdc: stroops_to_usdc(current_price),
        series,
        seconds_remaining,
        exponential: params.exponential,
    };

    let _ = state
        .redis
        .set(&cache_key, &response, PRICING_CACHE_TTL)
        .await;

    success(response, "Dutch auction price retrieved").into_response()
}

// ──────────────────────────────────────────────────────────────────────────────
// Bonding Curve
// ──────────────────────────────────────────────────────────────────────────────

/// Query parameters for the bonding curve price endpoint.
#[derive(Debug, Deserialize)]
pub struct BondingCurveQuery {
    /// Amplitude coefficient `a` (floating-point, e.g. `2.5`).
    pub a: f64,
    /// Polynomial exponent `b` (integer 1–4).
    pub b: u32,
    /// Base price floor `c` in stroops.
    pub c: i64,
    /// Initial / maximum supply for this tier.
    pub initial_supply: u32,
    /// Current remaining supply (tickets still available).
    pub remaining_supply: u32,
}

/// Query parameters for the bonding curve series endpoint.
#[derive(Debug, Deserialize)]
pub struct BondingCurveSeriesQuery {
    /// Amplitude coefficient `a`.
    pub a: f64,
    /// Polynomial exponent `b`.
    pub b: u32,
    /// Base price floor `c` in stroops.
    pub c: i64,
    /// Total supply of the tier (determines the x-axis range).
    pub total_supply: u32,
    /// Number of data points to generate. Defaults to `DEFAULT_SERIES_POINTS`.
    pub points: Option<usize>,
}

/// A single (supply, price) sample in the bonding curve series.
#[derive(Debug, Serialize, Deserialize)]
pub struct BondingCurvePoint {
    /// Remaining ticket supply at this data point.
    pub remaining_supply: u32,
    /// Price in stroops at this supply level.
    pub price_stroops: i64,
    /// Human-readable price in USDC.
    pub price_usdc: f64,
}

/// Response for `GET /api/v1/pricing/bonding-curve`.
#[derive(Debug, Serialize, Deserialize)]
pub struct BondingCurvePriceResponse {
    /// Spot price at `remaining_supply`.
    pub spot_price_stroops: i64,
    pub spot_price_usdc: f64,
    /// Integral cost to buy `quantity` tickets (if `quantity` was provided).
    pub integral_cost_stroops: Option<i64>,
    pub integral_cost_usdc: Option<f64>,
}

/// Response for `GET /api/v1/pricing/bonding-curve/series`.
#[derive(Debug, Serialize, Deserialize)]
pub struct BondingCurveSeriesResponse {
    /// Price series from `total_supply` down to `0` (left = many tickets, right = few).
    pub series: Vec<BondingCurvePoint>,
    /// Floor price (at supply = 0).
    pub floor_price_stroops: i64,
    /// Starting price (at supply = total_supply).
    pub starting_price_stroops: i64,
}

/// GET `/api/v1/pricing/bonding-curve`
///
/// Returns the spot price at the given remaining supply level, plus an
/// optional integral cost estimate for bulk purchases.
pub async fn get_bonding_curve_price(
    State(mut state): State<PricingState>,
    Query(params): Query<BondingCurveQuery>,
) -> Response {
    if let Err(e) = validate_bonding_curve_params(params.a, params.b, params.c) {
        return e.into_response();
    }
    if params.remaining_supply > params.initial_supply {
        return AppError::ValidationError(
            "remaining_supply cannot exceed initial_supply".to_string(),
        )
        .into_response();
    }

    let cache_key = format!(
        "pricing:bc:{}:{:.6}:{}:{}:{}:{}",
        params.a,
        params.b,
        params.c,
        params.initial_supply,
        params.remaining_supply,
        // no time component; supply-based pricing is deterministic
        ""
    );

    if let Ok(Some(cached)) = state
        .redis
        .get::<BondingCurvePriceResponse>(&cache_key)
        .await
    {
        return success(cached, "Bonding curve price (cached)").into_response();
    }

    let spot = compute_bonding_price(params.a, params.b, params.c, params.remaining_supply);

    let response = BondingCurvePriceResponse {
        spot_price_stroops: spot,
        spot_price_usdc: stroops_to_usdc(spot),
        integral_cost_stroops: None,
        integral_cost_usdc: None,
    };

    let _ = state
        .redis
        .set(&cache_key, &response, PRICING_CACHE_TTL)
        .await;

    success(response, "Bonding curve spot price retrieved").into_response()
}

/// GET `/api/v1/pricing/bonding-curve/series`
///
/// Generates a price series from full supply to zero for rendering the
/// bonding curve visualisation.
pub async fn get_bonding_curve_series(
    State(mut state): State<PricingState>,
    Query(params): Query<BondingCurveSeriesQuery>,
) -> Response {
    if let Err(e) = validate_bonding_curve_params(params.a, params.b, params.c) {
        return e.into_response();
    }
    if params.total_supply == 0 {
        return AppError::ValidationError("total_supply must be > 0".to_string()).into_response();
    }

    let points = params
        .points
        .unwrap_or(DEFAULT_SERIES_POINTS)
        .clamp(2, MAX_SERIES_POINTS);

    let cache_key = format!(
        "pricing:bc:series:{:.6}:{}:{}:{}:{}",
        params.a, params.b, params.c, params.total_supply, points,
    );

    if let Ok(Some(cached)) = state
        .redis
        .get::<BondingCurveSeriesResponse>(&cache_key)
        .await
    {
        return success(cached, "Bonding curve series (cached)").into_response();
    }

    let series = build_bonding_series(params.a, params.b, params.c, params.total_supply, points);

    let floor_price = compute_bonding_price(params.a, params.b, params.c, 0);
    let starting_price = compute_bonding_price(params.a, params.b, params.c, params.total_supply);

    let response = BondingCurveSeriesResponse {
        series,
        floor_price_stroops: floor_price,
        starting_price_stroops: starting_price,
    };

    let _ = state
        .redis
        .set(&cache_key, &response, PRICING_CACHE_TTL)
        .await;

    success(response, "Bonding curve series generated").into_response()
}

// ──────────────────────────────────────────────────────────────────────────────
// Pure computation helpers (no I/O, fully unit-testable)
// ──────────────────────────────────────────────────────────────────────────────

/// Compute the Dutch auction price at `now` using linear or exponential decay.
///
/// Mirrors the on-chain formula:
/// ```text
/// P(t) = P_start - (t - t0) / T * (P_start - P_reserve)   [linear]
/// P(t) = P_reserve + (P_start - P_reserve) * ((T - elapsed) / T)^2  [exponential]
/// ```
pub(crate) fn compute_dutch_price(
    start_price: i64,
    reserve_price: i64,
    start_time: u64,
    end_time: u64,
    now: u64,
    exponential: bool,
) -> i64 {
    if now >= end_time {
        return reserve_price;
    }
    if now <= start_time {
        return start_price;
    }

    let elapsed = (now - start_time) as f64;
    let total = (end_time - start_time) as f64;
    let spread = (start_price - reserve_price) as f64;

    let price = if exponential {
        // Concave approximation: 2f - f² stays above linear f at every interior point.
        // At midpoint (f=0.5): factor = 0.75 vs linear 0.5, so price is higher.
        let f = (total - elapsed) / total;
        reserve_price as f64 + spread * (2.0 * f - f * f)
    } else {
        start_price as f64 - (elapsed / total) * spread
    };

    price.round() as i64
}

/// Build a uniformly-spaced time series of (timestamp, price) points for the
/// Dutch auction from `start_time` to `end_time`.
fn build_dutch_series(
    start_price: i64,
    reserve_price: i64,
    start_time: u64,
    end_time: u64,
    points: usize,
    exponential: bool,
) -> Vec<PricePoint> {
    let total = (end_time - start_time) as f64;
    (0..points)
        .map(|i| {
            let frac = i as f64 / (points - 1).max(1) as f64;
            let ts = start_time + (total * frac).round() as u64;
            let ts = ts.min(end_time);
            let price = compute_dutch_price(
                start_price,
                reserve_price,
                start_time,
                end_time,
                ts,
                exponential,
            );
            PricePoint {
                timestamp: ts,
                price_stroops: price,
                price_usdc: stroops_to_usdc(price),
            }
        })
        .collect()
}

/// Compute bonding curve spot price: `P(s) = a * s^b + c`.
///
/// Uses `f64` arithmetic on the server side (unlike the on-chain integer version)
/// to support fractional `a` values provided via the API.
pub(crate) fn compute_bonding_price(a: f64, b: u32, c: i64, remaining_supply: u32) -> i64 {
    let s = remaining_supply as f64;
    let price = a * s.powi(b as i32) + c as f64;
    price.round() as i64
}

/// Build a supply series of `points` evenly spaced from `total_supply` down to 0.
fn build_bonding_series(
    a: f64,
    b: u32,
    c: i64,
    total_supply: u32,
    points: usize,
) -> Vec<BondingCurvePoint> {
    (0..points)
        .map(|i| {
            // Map from total_supply → 0 so the chart reads left-to-right as
            // "tickets sold increases → price increases".
            let frac = i as f64 / (points - 1).max(1) as f64;
            let supply = (total_supply as f64 * (1.0 - frac)).round() as u32;
            let price = compute_bonding_price(a, b, c, supply);
            BondingCurvePoint {
                remaining_supply: supply,
                price_stroops: price,
                price_usdc: stroops_to_usdc(price),
            }
        })
        .collect()
}

/// Validate that bonding curve parameters are within acceptable bounds.
fn validate_bonding_curve_params(a: f64, b: u32, c: i64) -> Result<(), AppError> {
    if a <= 0.0 {
        return Err(AppError::ValidationError(
            "Amplitude 'a' must be greater than 0".to_string(),
        ));
    }
    if b == 0 || b > 4 {
        return Err(AppError::ValidationError(
            "Exponent 'b' must be between 1 and 4".to_string(),
        ));
    }
    if c < 0 {
        return Err(AppError::ValidationError(
            "Base price 'c' must be >= 0".to_string(),
        ));
    }
    Ok(())
}

// ──────────────────────────────────────────────────────────────────────────────
// Utility
// ──────────────────────────────────────────────────────────────────────────────

/// Convert stroops to USDC (1 USDC = 10,000,000 stroops).
fn stroops_to_usdc(stroops: i64) -> f64 {
    stroops as f64 / 10_000_000.0
}

fn unix_now() -> u64 {
    use std::time::{SystemTime, UNIX_EPOCH};
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

    // ── Dutch auction price ──────────────────────────────────────────────────

    #[test]
    fn dutch_linear_at_start_time() {
        let p = compute_dutch_price(1_000_000, 100_000, 1000, 2000, 1000, false);
        assert_eq!(p, 1_000_000);
    }

    #[test]
    fn dutch_linear_at_end_time() {
        let p = compute_dutch_price(1_000_000, 100_000, 1000, 2000, 2000, false);
        assert_eq!(p, 100_000);
    }

    #[test]
    fn dutch_linear_at_midpoint() {
        let p = compute_dutch_price(1_000_000, 100_000, 0, 1000, 500, false);
        // P(500) = 1_000_000 - 0.5 * 900_000 = 550_000
        assert_eq!(p, 550_000);
    }

    #[test]
    fn dutch_linear_after_end_clamped_to_reserve() {
        let p = compute_dutch_price(1_000_000, 100_000, 0, 1000, 9999, false);
        assert_eq!(p, 100_000);
    }

    #[test]
    fn dutch_exponential_at_start() {
        let p = compute_dutch_price(1_000_000, 100_000, 0, 1000, 0, true);
        assert_eq!(p, 1_000_000);
    }

    #[test]
    fn dutch_exponential_midpoint_above_linear_midpoint() {
        let p_lin = compute_dutch_price(1_000_000, 100_000, 0, 1000, 500, false);
        let p_exp = compute_dutch_price(1_000_000, 100_000, 0, 1000, 500, true);
        assert!(
            p_exp > p_lin,
            "exponential={} should stay higher than linear={} at midpoint",
            p_exp,
            p_lin
        );
    }

    #[test]
    fn dutch_series_has_correct_point_count() {
        let series = build_dutch_series(1_000_000, 100_000, 0, 1000, 10, false);
        assert_eq!(series.len(), 10);
    }

    #[test]
    fn dutch_series_first_price_is_start_price() {
        let series = build_dutch_series(1_000_000, 100_000, 0, 1000, 10, false);
        assert_eq!(series[0].price_stroops, 1_000_000);
    }

    #[test]
    fn dutch_series_last_price_is_reserve_price() {
        let series = build_dutch_series(1_000_000, 100_000, 0, 1000, 10, false);
        assert_eq!(series.last().unwrap().price_stroops, 100_000);
    }

    // ── Bonding curve price ──────────────────────────────────────────────────

    #[test]
    fn bonding_price_linear_at_100_supply() {
        // P(100) = 1 * 100 + 500_000 = 500_100
        let p = compute_bonding_price(1.0, 1, 500_000, 100);
        assert_eq!(p, 500_100);
    }

    #[test]
    fn bonding_price_at_zero_supply_equals_floor() {
        // P(0) = 1 * 0^1 + 500_000 = 500_000
        let p = compute_bonding_price(1.0, 1, 500_000, 0);
        assert_eq!(p, 500_000);
    }

    #[test]
    fn bonding_price_quadratic() {
        // P(10) = 2.0 * 10^2 + 0 = 200
        let p = compute_bonding_price(2.0, 2, 0, 10);
        assert_eq!(p, 200);
    }

    #[test]
    fn bonding_price_increases_as_supply_decreases() {
        // With a linear curve and a > 0, higher remaining supply → lower price.
        // As supply drops from 100 → 50, price should drop too.
        let p_high = compute_bonding_price(1.0, 1, 0, 100);
        let p_low = compute_bonding_price(1.0, 1, 0, 50);
        assert!(
            p_high > p_low,
            "high_supply_price={} low_supply_price={}",
            p_high,
            p_low
        );
    }

    #[test]
    fn bonding_series_has_correct_point_count() {
        let series = build_bonding_series(1.0, 1, 0, 100, 20);
        assert_eq!(series.len(), 20);
    }

    #[test]
    fn bonding_series_starts_at_total_supply() {
        let series = build_bonding_series(1.0, 1, 0, 100, 20);
        assert_eq!(series[0].remaining_supply, 100);
    }

    #[test]
    fn bonding_series_ends_at_zero_supply() {
        let series = build_bonding_series(1.0, 1, 0, 100, 20);
        assert_eq!(series.last().unwrap().remaining_supply, 0);
    }

    // ── Validation ───────────────────────────────────────────────────────────

    #[test]
    fn validate_rejects_zero_amplitude() {
        assert!(validate_bonding_curve_params(0.0, 1, 0).is_err());
    }

    #[test]
    fn validate_rejects_negative_amplitude() {
        assert!(validate_bonding_curve_params(-1.0, 1, 0).is_err());
    }

    #[test]
    fn validate_rejects_zero_exponent() {
        assert!(validate_bonding_curve_params(1.0, 0, 0).is_err());
    }

    #[test]
    fn validate_rejects_exponent_above_max() {
        assert!(validate_bonding_curve_params(1.0, 5, 0).is_err());
    }

    #[test]
    fn validate_rejects_negative_base_price() {
        assert!(validate_bonding_curve_params(1.0, 1, -1).is_err());
    }

    #[test]
    fn validate_accepts_valid_params() {
        assert!(validate_bonding_curve_params(2.5, 2, 500_000).is_ok());
    }

    // ── Utility ──────────────────────────────────────────────────────────────

    #[test]
    fn stroops_to_usdc_conversion() {
        assert!((stroops_to_usdc(10_000_000) - 1.0).abs() < f64::EPSILON);
        assert!((stroops_to_usdc(1_000_000) - 0.1).abs() < 1e-9);
    }
}
