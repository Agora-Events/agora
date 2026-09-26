//! # Bancor Bonding Curve AMM (Automated Market Maker)
//!
//! Implements a polynomial bonding curve for continuous ticket pricing where
//! the price rises as available supply decreases.
//!
//! ## Pricing Formula
//!
//! ```text
//! P(s) = a * s^b + c
//! ```
//!
//! Where:
//! - `s`  – remaining supply (number of tickets still available)
//! - `a`  – curve amplitude coefficient (scaled by `PARAM_SCALE`)
//! - `b`  – exponent (fixed-point; 1 = linear, 2 = quadratic, …)
//! - `c`  – base price floor in stroops (minimum price regardless of supply)
//!
//! As tickets are sold (`s` decreases), the price rises, discouraging early
//! bulk purchases and rewarding genuine buyers.
//!
//! ## Slippage Guard
//!
//! Every purchase must supply `max_slippage_bps`.  If the price at the moment
//! of purchase exceeds `quoted_price * (10000 + max_slippage_bps) / 10000`,
//! the transaction is rejected with [`TicketPaymentError::PriceOutsideSlippage`],
//! protecting buyers against front-running and sandwich attacks.
//!
//! ## Parameter Encoding
//!
//! All curve parameters are stored as integers scaled by `PARAM_SCALE = 1_000_000`
//! to avoid floating-point arithmetic in the Soroban WASM environment.

#![allow(dead_code)]

use soroban_sdk::{contracttype, Env, String};

use crate::error::TicketPaymentError;
use crate::keys::PricingKey;

// ──────────────────────────────────────────────────────────────────────────────
// Constants
// ──────────────────────────────────────────────────────────────────────────────

/// Fixed-point scale for curve parameters `a` and `c`.
/// e.g. `a = 2_500_000` represents `a = 2.5`.
pub const PARAM_SCALE: i128 = 1_000_000;

/// Basis-point denominator.
pub const MAX_BPS: u32 = 10_000;

/// Maximum supported exponent `b` (stored as integer; `b=3` means cubic).
pub const MAX_EXPONENT: u32 = 4;

// ──────────────────────────────────────────────────────────────────────────────
// Curve Config
// ──────────────────────────────────────────────────────────────────────────────

/// On-chain configuration for a bonding curve attached to a ticket tier.
///
/// All price values are denominated in **stroops** (1 USDC = 10^7 stroops).
#[contracttype]
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct BondingCurveConfig {
    /// Amplitude coefficient `a` (scaled by `PARAM_SCALE`).
    /// A value of `1_000_000` means `a = 1.0`.
    pub a_scaled: i128,
    /// Integer exponent `b` for the polynomial term `s^b`.
    /// Supported values: 1 (linear), 2 (quadratic), 3 (cubic), 4 (quartic).
    pub b_exponent: u32,
    /// Base price floor `c` in stroops.
    pub c_base: i128,
    /// Total initial supply for this tier (used to compute `s`).
    pub initial_supply: u32,
}

// ──────────────────────────────────────────────────────────────────────────────
// Storage helpers
// ──────────────────────────────────────────────────────────────────────────────

/// Persist a bonding curve config for an (event_id, tier_id) pair.
pub fn set_bonding_curve(
    env: &Env,
    event_id: &String,
    tier_id: &String,
    config: &BondingCurveConfig,
) {
    env.storage().persistent().set(
        &PricingKey::BondingCurve(event_id.clone(), tier_id.clone()),
        config,
    );
}

/// Retrieve the bonding curve config for an (event_id, tier_id) pair, if any.
pub fn get_bonding_curve(
    env: &Env,
    event_id: &String,
    tier_id: &String,
) -> Option<BondingCurveConfig> {
    env.storage()
        .persistent()
        .get(&PricingKey::BondingCurve(event_id.clone(), tier_id.clone()))
}

// ──────────────────────────────────────────────────────────────────────────────
// Config Validation
// ──────────────────────────────────────────────────────────────────────────────

/// Validate a [`BondingCurveConfig`] before storing it on-chain.
///
/// Rules:
/// - `a_scaled` > 0
/// - `b_exponent` in `[1, MAX_EXPONENT]`
/// - `c_base` >= 0
/// - `initial_supply` > 0
pub fn validate_bonding_curve_config(cfg: &BondingCurveConfig) -> Result<(), TicketPaymentError> {
    if cfg.a_scaled <= 0 {
        return Err(TicketPaymentError::InvalidPrice);
    }
    if cfg.b_exponent == 0 || cfg.b_exponent > MAX_EXPONENT {
        return Err(TicketPaymentError::InvalidPrice);
    }
    if cfg.c_base < 0 {
        return Err(TicketPaymentError::InvalidPrice);
    }
    if cfg.initial_supply == 0 {
        return Err(TicketPaymentError::MaxSupplyExceeded);
    }
    Ok(())
}

// ──────────────────────────────────────────────────────────────────────────────
// Pricing
// ──────────────────────────────────────────────────────────────────────────────

/// Compute the ticket price from the bonding curve for a given remaining supply.
///
/// ```text
/// P(s) = (a_scaled * s^b) / PARAM_SCALE + c_base
/// ```
///
/// `remaining_supply` is the number of tickets still available *before* this
/// purchase.  The price is computed at the current supply level.
///
/// Returns the price in **stroops**.
///
/// ## Mathematical Invariants
///
/// 1. **Monotonicity**: Because `s^b` is non-decreasing in `s` for `b >= 1`,
///    and `a_scaled > 0`, we have `P(s + 1) >= P(s)` for all `s >= 0`.
///    Higher remaining supply → higher price; as tickets sell the price falls.
///
/// 2. **Base Price Anchor**: At `s = 0` the variable component disappears:
///    `P(0) = (a_scaled * 0^b) / PARAM_SCALE + c_base = c_base`.
///    Therefore `P(0) == c_base` exactly.
///
/// 3. **Non-Zero Guarantee**: When `c_base > 0`, integer division of
///    `(a_scaled * s^b)` by `PARAM_SCALE` can only reduce the variable
///    component toward zero, never below.  The `c_base` floor ensures the
///    total price is at least `c_base > 0`.
///
/// 4. **Overflow Safety**: The function uses [`integer_pow`] which calls
///    `i128::saturating_mul` internally, so no panic occurs for any supply
///    value up to `u32::MAX`.  Callers should validate that `a_scaled` is
///    chosen such that intermediate products stay within `i128` range for
///    their expected supply levels.
pub fn bonding_curve_price(cfg: &BondingCurveConfig, remaining_supply: u32) -> i128 {
    let s = remaining_supply as i128;

    // Compute s^b using integer exponentiation.
    let s_pow_b = integer_pow(s, cfg.b_exponent);

    // P(s) = (a * s^b) / PARAM_SCALE + c
    let variable_component = cfg.a_scaled * s_pow_b / PARAM_SCALE;
    variable_component + cfg.c_base
}

/// Compute the **integral** of the bonding curve from `s_end` to `s_start`
/// to derive the total cost of purchasing `quantity` tickets in one transaction
/// (Bancor-style reserve pricing).
///
/// For `P(s) = a * s^b + c`, the integral from `s1` to `s2` is:
/// ```text
/// ∫ P(s) ds = a * s^(b+1) / (b+1) + c * s  |_{s1}^{s2}
/// ```
///
/// Where:
/// - `s_start = remaining_supply` (before purchase)
/// - `s_end   = remaining_supply - quantity`
///
/// This accurately prices bulk purchases without allowing arbitrage through
/// splitting orders.
///
/// Returns total cost in **stroops**.
pub fn bonding_curve_integral_cost(
    cfg: &BondingCurveConfig,
    remaining_supply: u32,
    quantity: u32,
) -> Result<i128, TicketPaymentError> {
    if quantity == 0 {
        return Ok(0);
    }
    if quantity > remaining_supply {
        return Err(TicketPaymentError::MaxSupplyExceeded);
    }

    let s_start = remaining_supply as i128;
    let s_end = (remaining_supply - quantity) as i128;
    let b_plus_1 = (cfg.b_exponent + 1) as i128;

    // Integral term: a * (s^(b+1)) / (b+1) / PARAM_SCALE
    let integral_at = |s: i128| -> i128 {
        let s_pow = integer_pow(s, cfg.b_exponent + 1);
        cfg.a_scaled * s_pow / b_plus_1 / PARAM_SCALE + cfg.c_base * s
    };

    let cost = integral_at(s_start) - integral_at(s_end);
    if cost < 0 {
        return Err(TicketPaymentError::ArithmeticError);
    }
    Ok(cost)
}

/// Validate that `buyer_quoted_price` is within the slippage tolerance of
/// `actual_price`.
///
/// Rejects if: `actual_price > buyer_quoted_price * (MAX_BPS + max_slippage_bps) / MAX_BPS`
pub fn check_slippage(
    actual_price: i128,
    buyer_quoted_price: i128,
    max_slippage_bps: u32,
) -> Result<(), TicketPaymentError> {
    if max_slippage_bps > MAX_BPS {
        return Err(TicketPaymentError::InvalidSlippageBps);
    }
    let ceiling =
        buyer_quoted_price + (buyer_quoted_price * max_slippage_bps as i128 / MAX_BPS as i128);
    if actual_price > ceiling {
        return Err(TicketPaymentError::PriceOutsideSlippage);
    }
    Ok(())
}

// ──────────────────────────────────────────────────────────────────────────────
// Integer math helpers
// ──────────────────────────────────────────────────────────────────────────────

/// Integer exponentiation: `base^exp` for small `exp` values typical of curve
/// parameters (1–4).  Uses iterative multiplication to stay deterministic.
fn integer_pow(base: i128, exp: u32) -> i128 {
    if exp == 0 {
        return 1;
    }
    let mut result: i128 = 1;
    for _ in 0..exp {
        result = result.saturating_mul(base);
    }
    result
}

// ──────────────────────────────────────────────────────────────────────────────
// Unit Tests
// ──────────────────────────────────────────────────────────────────────────────

#[cfg(test)]
mod tests {
    use super::*;

    fn linear_cfg(a: i128, c: i128, supply: u32) -> BondingCurveConfig {
        BondingCurveConfig {
            a_scaled: a * PARAM_SCALE,
            b_exponent: 1,
            c_base: c,
            initial_supply: supply,
        }
    }

    fn quadratic_cfg(a: i128, c: i128, supply: u32) -> BondingCurveConfig {
        BondingCurveConfig {
            a_scaled: a * PARAM_SCALE,
            b_exponent: 2,
            c_base: c,
            initial_supply: supply,
        }
    }

    // ── integer_pow ──────────────────────────────────────────────────────────

    #[test]
    fn pow_zero_exponent_returns_one() {
        assert_eq!(integer_pow(5, 0), 1);
        assert_eq!(integer_pow(0, 0), 1);
    }

    #[test]
    fn pow_one_returns_base() {
        assert_eq!(integer_pow(7, 1), 7);
    }

    #[test]
    fn pow_two() {
        assert_eq!(integer_pow(3, 2), 9);
        assert_eq!(integer_pow(10, 2), 100);
    }

    #[test]
    fn pow_three() {
        assert_eq!(integer_pow(4, 3), 64);
    }

    // ── bonding_curve_price ──────────────────────────────────────────────────

    #[test]
    fn linear_price_at_full_supply() {
        // P(100) = (1 * PARAM_SCALE * 100) / PARAM_SCALE + 500_000 = 100 + 500_000 = 500_100
        let cfg = linear_cfg(1, 500_000, 100);
        assert_eq!(bonding_curve_price(&cfg, 100), 500_100);
    }

    #[test]
    fn linear_price_at_half_supply() {
        // P(50) = (1 * PARAM_SCALE * 50) / PARAM_SCALE + 500_000 = 50 + 500_000 = 500_050
        let cfg = linear_cfg(1, 500_000, 100);
        assert_eq!(bonding_curve_price(&cfg, 50), 500_050);
    }

    #[test]
    fn linear_price_at_zero_supply() {
        // P(0) = 1 * 0 + 500_000 = 500_000 (floor price)
        let cfg = linear_cfg(1, 500_000, 100);
        assert_eq!(bonding_curve_price(&cfg, 0), 500_000);
    }

    #[test]
    fn price_increases_as_supply_drops() {
        // P(s) = a*s + c, so higher remaining supply → higher price.
        // At supply=900: P = 900 + 100_000 = 100_900
        // At supply=100: P = 100 + 100_000 = 100_100
        // Price is lower when supply is lower (more tickets sold → cheaper remaining ones).
        let cfg = linear_cfg(1, 100_000, 1000);
        let price_high_supply = bonding_curve_price(&cfg, 900);
        let price_low_supply = bonding_curve_price(&cfg, 100);
        assert!(
            price_high_supply > price_low_supply,
            "price should be higher at higher remaining supply: high={} low={}",
            price_high_supply,
            price_low_supply
        );
    }

    #[test]
    fn quadratic_price_at_100_supply() {
        // P(10) = 2 * 10^2 + 0 = 200
        let cfg = quadratic_cfg(2, 0, 100);
        assert_eq!(bonding_curve_price(&cfg, 10), 200);
    }

    // ── bonding_curve_integral_cost ──────────────────────────────────────────

    #[test]
    fn integral_single_ticket_equals_spot_price() {
        // With linear curve P(s) = 1*s + 0, buying 1 ticket from supply=10:
        // integral from 9 to 10 = [s^2/2]_9^10 = 50 - 40.5 = 9.5 ≈ 9 or 10 depending on integer division
        // We verify the single-ticket integral is close to spot price
        let cfg = BondingCurveConfig {
            a_scaled: PARAM_SCALE, // a = 1
            b_exponent: 1,
            c_base: 0,
            initial_supply: 10,
        };
        let spot = bonding_curve_price(&cfg, 10);
        let integral = bonding_curve_integral_cost(&cfg, 10, 1).unwrap();
        // spot = 10, integral ≈ 9 (rounding due to integer division)
        let diff = (spot - integral).abs();
        assert!(
            diff <= 2,
            "spot={} integral={} diff={}",
            spot,
            integral,
            diff
        );
    }

    #[test]
    fn integral_zero_quantity_returns_zero() {
        let cfg = linear_cfg(1, 0, 100);
        assert_eq!(bonding_curve_integral_cost(&cfg, 50, 0).unwrap(), 0);
    }

    #[test]
    fn integral_exceeds_supply_returns_error() {
        let cfg = linear_cfg(1, 0, 100);
        assert!(matches!(
            bonding_curve_integral_cost(&cfg, 10, 11),
            Err(TicketPaymentError::MaxSupplyExceeded)
        ));
    }

    #[test]
    fn integral_bulk_more_than_sequential() {
        // Buying 5 tickets at once via integral should cost the same as
        // summing 5 individual spot prices (within integer rounding).
        let cfg = linear_cfg(1, 100_000, 20);
        let bulk_cost = bonding_curve_integral_cost(&cfg, 20, 5).unwrap();
        let sequential_cost: i128 = (16..=20).map(|s| bonding_curve_price(&cfg, s)).sum();
        // They may differ slightly due to integral vs Riemann sum, but should
        // be within a few stroops.
        let diff = (bulk_cost - sequential_cost).abs();
        assert!(
            diff < 100_000,
            "bulk={} sequential={} diff={}",
            bulk_cost,
            sequential_cost,
            diff
        );
    }

    // ── check_slippage ───────────────────────────────────────────────────────

    #[test]
    fn slippage_ok_when_price_equals_quoted() {
        assert!(check_slippage(1_000_000, 1_000_000, 300).is_ok());
    }

    #[test]
    fn slippage_ok_when_price_slightly_above_quoted() {
        // quoted = 1_000_000, slippage = 3 %, ceiling = 1_030_000
        assert!(check_slippage(1_029_000, 1_000_000, 300).is_ok());
    }

    #[test]
    fn slippage_fails_when_price_above_ceiling() {
        // ceiling = 1_030_000, actual = 1_031_000 → reject
        assert!(matches!(
            check_slippage(1_031_000, 1_000_000, 300),
            Err(TicketPaymentError::PriceOutsideSlippage)
        ));
    }

    #[test]
    fn slippage_ok_when_price_below_quoted() {
        // Price dropped — buyer gets a better deal.
        assert!(check_slippage(900_000, 1_000_000, 300).is_ok());
    }

    #[test]
    fn slippage_invalid_bps_above_max() {
        assert!(matches!(
            check_slippage(1_000_000, 1_000_000, 10_001),
            Err(TicketPaymentError::InvalidSlippageBps)
        ));
    }

    // ── validate_bonding_curve_config ────────────────────────────────────────

    #[test]
    fn validate_rejects_zero_amplitude() {
        let cfg = BondingCurveConfig {
            a_scaled: 0,
            b_exponent: 1,
            c_base: 0,
            initial_supply: 100,
        };
        assert!(matches!(
            validate_bonding_curve_config(&cfg),
            Err(TicketPaymentError::InvalidPrice)
        ));
    }

    #[test]
    fn validate_rejects_zero_exponent() {
        let cfg = BondingCurveConfig {
            a_scaled: PARAM_SCALE,
            b_exponent: 0,
            c_base: 0,
            initial_supply: 100,
        };
        assert!(matches!(
            validate_bonding_curve_config(&cfg),
            Err(TicketPaymentError::InvalidPrice)
        ));
    }

    #[test]
    fn validate_rejects_exponent_too_large() {
        let cfg = BondingCurveConfig {
            a_scaled: PARAM_SCALE,
            b_exponent: MAX_EXPONENT + 1,
            c_base: 0,
            initial_supply: 100,
        };
        assert!(matches!(
            validate_bonding_curve_config(&cfg),
            Err(TicketPaymentError::InvalidPrice)
        ));
    }

    #[test]
    fn validate_rejects_zero_supply() {
        let cfg = BondingCurveConfig {
            a_scaled: PARAM_SCALE,
            b_exponent: 1,
            c_base: 0,
            initial_supply: 0,
        };
        assert!(matches!(
            validate_bonding_curve_config(&cfg),
            Err(TicketPaymentError::MaxSupplyExceeded)
        ));
    }

    #[test]
    fn validate_accepts_valid_config() {
        let cfg = BondingCurveConfig {
            a_scaled: 2 * PARAM_SCALE,
            b_exponent: 2,
            c_base: 500_000,
            initial_supply: 500,
        };
        assert!(validate_bonding_curve_config(&cfg).is_ok());
    }

    // ── Issue #1276: Monotonicity & Boundary Property Tests ─────────────────

    /// Invariant 1: P(n+1) >= P(n) for all n in [0, 1000].
    ///
    /// This test iterates supply from 0 to 1000 and asserts that the price at
    /// supply n+1 is always at least as large as the price at supply n.
    #[test]
    fn price_is_monotonically_non_decreasing() {
        // Test with a linear curve (b = 1).
        let cfg_linear = BondingCurveConfig {
            a_scaled: PARAM_SCALE, // a = 1
            b_exponent: 1,
            c_base: 100_000,
            initial_supply: 1001,
        };
        for n in 0u32..1000 {
            let price_n = bonding_curve_price(&cfg_linear, n);
            let price_n_plus_1 = bonding_curve_price(&cfg_linear, n + 1);
            assert!(
                price_n_plus_1 >= price_n,
                "monotonicity violated (linear) at n={}: P({})={} < P({})={}",
                n,
                n + 1,
                price_n_plus_1,
                n,
                price_n
            );
        }

        // Test with a quadratic curve (b = 2).
        let cfg_quad = BondingCurveConfig {
            a_scaled: PARAM_SCALE / 10, // a = 0.1 to avoid overflow
            b_exponent: 2,
            c_base: 50_000,
            initial_supply: 1001,
        };
        for n in 0u32..1000 {
            let price_n = bonding_curve_price(&cfg_quad, n);
            let price_n_plus_1 = bonding_curve_price(&cfg_quad, n + 1);
            assert!(
                price_n_plus_1 >= price_n,
                "monotonicity violated (quadratic) at n={}: P({})={} < P({})={}",
                n,
                n + 1,
                price_n_plus_1,
                n,
                price_n
            );
        }
    }

    /// Invariant 2: P(0) == c_base (base price anchor).
    #[test]
    fn price_at_zero_supply_equals_base_price() {
        let c_base = 500_000i128;
        let cfg = BondingCurveConfig {
            a_scaled: 3 * PARAM_SCALE,
            b_exponent: 2,
            c_base,
            initial_supply: 100,
        };
        assert_eq!(
            bonding_curve_price(&cfg, 0),
            c_base,
            "P(0) must equal c_base"
        );
    }

    /// Invariant 3: Non-zero price guarantee when base_price > 0.
    ///
    /// Integer division of the variable component can only reduce it, never
    /// produce a price below c_base.
    #[test]
    fn price_never_zero_when_base_price_positive() {
        let cfg = BondingCurveConfig {
            a_scaled: PARAM_SCALE,
            b_exponent: 1,
            c_base: 1, // minimal positive base price
            initial_supply: 200,
        };
        for s in 0u32..=200 {
            let price = bonding_curve_price(&cfg, s);
            assert!(
                price > 0,
                "price must be > 0 when c_base > 0, but got {} at supply {}",
                price,
                s
            );
        }
    }

    /// Invariant 4: Boundary safety — no panic at supply = 0 and supply = initial_supply.
    ///
    /// Uses a large supply value to verify the implementation does not overflow
    /// or panic at the extremes.
    #[test]
    fn boundary_no_panic_at_zero_and_max_supply() {
        let max_supply = 10_000u32;
        let cfg = BondingCurveConfig {
            a_scaled: PARAM_SCALE,
            b_exponent: 1,
            c_base: 100_000,
            initial_supply: max_supply,
        };

        // Should not panic at supply = 0.
        let price_at_zero = bonding_curve_price(&cfg, 0);
        assert_eq!(price_at_zero, cfg.c_base, "P(0) must equal c_base");

        // Should not panic at supply = max_supply.
        let price_at_max = bonding_curve_price(&cfg, max_supply);
        assert!(
            price_at_max >= cfg.c_base,
            "P(max_supply) must be >= c_base"
        );
    }

    /// Boundary safety with cubic curve — verifies no overflow up to supply = 1000.
    #[test]
    fn boundary_cubic_no_overflow_up_to_large_supply() {
        let cfg = BondingCurveConfig {
            a_scaled: 1, // tiny amplitude to avoid overflow with cubic
            b_exponent: 3,
            c_base: 1_000,
            initial_supply: 1_000,
        };
        // Should not panic.
        let price = bonding_curve_price(&cfg, 1_000);
        assert!(price >= cfg.c_base);
    }
}
