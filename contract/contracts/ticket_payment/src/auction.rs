//! # Dutch Auction Engine
//!
//! Implements on-chain linear Dutch Auctions for ticket tiers.
//!
//! ## Price Decay
//!
//! ```
//! P(t) = P_start - (t - t0) / T * (P_start - P_reserve)
//! ```
//!
//! Where:
//! - `P_start`   – the starting (highest) price in stroops
//! - `P_reserve` – the floor price; the auction never falls below this
//! - `t0`        – ledger timestamp when the auction starts
//! - `T`         – total duration of the auction in seconds (end_time - start_time)
//! - `t`         – current ledger timestamp (clamped to [t0, end_time])
//!
//! ## Commit-Reveal Anti-Bot Scheme
//!
//! To prevent MEV / high-frequency bots from front-running purchases, ticket
//! buyers use a two-phase commit-reveal protocol:
//!
//! ### Phase 1 – Commit
//! The buyer submits `SHA-256(secret || buyer_address || price_stroops)`.
//! The contract stores the hash with an expiry timestamp.
//!
//! ### Phase 2 – Reveal
//! The buyer submits the plaintext `secret`, `price_stroops`, and the actual
//! payment.  The contract verifies the hash, checks the price is still valid
//! (within `max_slippage_bps`), and finalises the purchase.  Expired or
//! re-used commits are rejected.
//!
//! ## Slippage Guard
//!
//! The caller supplies `max_slippage_bps` (e.g. 300 = 3 %).  If the current
//! price has risen above `committed_price * (10000 + max_slippage_bps) / 10000`
//! the reveal is rejected with `PriceOutsideSlippage`.

#![allow(dead_code)]

use soroban_sdk::xdr::ToXdr;
use soroban_sdk::{contracttype, Address, BytesN, Env, String};

use crate::error::TicketPaymentError;
use crate::keys::PricingKey;

// ──────────────────────────────────────────────────────────────────────────────
// Constants
// ──────────────────────────────────────────────────────────────────────────────

/// Number of seconds a commit is valid before it expires.
pub const COMMIT_EXPIRY_SECS: u64 = 300; // 5 minutes

/// Basis-point denominator used across all BPS calculations.
pub const MAX_BPS: u32 = 10_000;

// ──────────────────────────────────────────────────────────────────────────────
// Storage Types
// ──────────────────────────────────────────────────────────────────────────────

/// On-chain record of a pending commit in the commit-reveal scheme.
#[contracttype]
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct PendingCommit {
    /// SHA-256(secret || buyer_address || price_stroops) provided by the buyer.
    pub hash: BytesN<32>,
    /// Ledger timestamp after which this commit is considered expired.
    pub expires_at: u64,
    /// The price (in stroops) the buyer locked in at commit time.
    pub committed_price: i128,
    /// Maximum slippage the buyer will tolerate (in basis points).
    pub max_slippage_bps: u32,
}

/// Persistent configuration for a Dutch auction attached to a ticket tier.
#[contracttype]
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct DutchAuctionConfig {
    /// Highest price (stroops) at `start_time`.
    pub start_price: i128,
    /// Floor price (stroops); the auction never drops below this.
    pub reserve_price: i128,
    /// Ledger timestamp at which the auction begins.
    pub start_time: u64,
    /// Ledger timestamp at which the auction ends; price is `reserve_price` from here on.
    pub end_time: u64,
    /// Whether the decay is linear (`false`) or exponential (`true`).
    pub exponential: bool,
}

// ──────────────────────────────────────────────────────────────────────────────
// Storage helpers (thin wrappers around env.storage())
// ──────────────────────────────────────────────────────────────────────────────

/// Persist a Dutch auction config for an (event_id, tier_id) pair.
pub fn set_dutch_auction(
    env: &Env,
    event_id: &String,
    tier_id: &String,
    config: &DutchAuctionConfig,
) {
    env.storage().persistent().set(
        &PricingKey::DutchAuction(event_id.clone(), tier_id.clone()),
        config,
    );
}

/// Retrieve the Dutch auction config for an (event_id, tier_id) pair, if any.
pub fn get_dutch_auction(
    env: &Env,
    event_id: &String,
    tier_id: &String,
) -> Option<DutchAuctionConfig> {
    env.storage()
        .persistent()
        .get(&PricingKey::DutchAuction(event_id.clone(), tier_id.clone()))
}

/// Store a pending commit for a buyer on a given (event_id, tier_id).
pub fn set_pending_commit(
    env: &Env,
    event_id: &String,
    tier_id: &String,
    buyer: &Address,
    commit: &PendingCommit,
) {
    env.storage().temporary().set(
        &PricingKey::PendingCommit(event_id.clone(), tier_id.clone(), buyer.clone()),
        commit,
    );
}

/// Retrieve and remove a pending commit (single-use).
pub fn take_pending_commit(
    env: &Env,
    event_id: &String,
    tier_id: &String,
    buyer: &Address,
) -> Option<PendingCommit> {
    let key = PricingKey::PendingCommit(event_id.clone(), tier_id.clone(), buyer.clone());
    let commit: Option<PendingCommit> = env.storage().temporary().get(&key);
    if commit.is_some() {
        env.storage().temporary().remove(&key);
    }
    commit
}

// ──────────────────────────────────────────────────────────────────────────────
// Auction Config Validation
// ──────────────────────────────────────────────────────────────────────────────

/// Validate the parameters for a new Dutch auction.
///
/// Rules:
/// - `start_price` > `reserve_price` > 0
/// - `end_time` > `start_time`
/// - `end_time` must be in the future
pub fn validate_dutch_auction_config(
    env: &Env,
    cfg: &DutchAuctionConfig,
) -> Result<(), TicketPaymentError> {
    if cfg.reserve_price <= 0 {
        return Err(TicketPaymentError::InvalidPrice);
    }
    if cfg.start_price <= cfg.reserve_price {
        return Err(TicketPaymentError::InvalidPrice);
    }
    if cfg.end_time <= cfg.start_time {
        return Err(TicketPaymentError::InvalidPrice);
    }
    let now = env.ledger().timestamp();
    if cfg.end_time <= now {
        return Err(TicketPaymentError::AuctionEnded);
    }
    Ok(())
}

// ──────────────────────────────────────────────────────────────────────────────
// Price Calculation
// ──────────────────────────────────────────────────────────────────────────────

/// Compute the current ticket price for a Dutch auction at the given ledger
/// timestamp using **linear** decay.
///
/// ```text
/// P(t) = P_start - (t - t0) / T * (P_start - P_reserve)
/// ```
///
/// Uses integer arithmetic scaled by `SCALE` to avoid floating-point on-chain.
/// Returns `P_reserve` once `t >= end_time`.
pub fn current_dutch_price_linear(cfg: &DutchAuctionConfig, now: u64) -> i128 {
    if now >= cfg.end_time {
        return cfg.reserve_price;
    }
    if now <= cfg.start_time {
        return cfg.start_price;
    }

    let elapsed = (now - cfg.start_time) as i128;
    let total = (cfg.end_time - cfg.start_time) as i128;
    let spread = cfg.start_price - cfg.reserve_price;

    // P(t) = start_price - (elapsed / total) * spread
    // Using integer arithmetic: price = start_price - elapsed * spread / total
    cfg.start_price - (elapsed * spread / total)
}

/// Compute the current ticket price for a Dutch auction using **exponential**
/// decay approximated via first-order Taylor expansion suitable for on-chain use.
///
/// Approximation: `P(t) ≈ P_reserve + (P_start - P_reserve) * (1 - elapsed/total)`
/// scaled by a decay factor of 2× (price halves over the full duration).
///
/// For a tighter approximation: the remaining fraction is computed as
/// `(total - elapsed) / total`, then squared (polynomial approximation of e^{-λt}).
pub fn current_dutch_price_exponential(cfg: &DutchAuctionConfig, now: u64) -> i128 {
    if now >= cfg.end_time {
        return cfg.reserve_price;
    }
    if now <= cfg.start_time {
        return cfg.start_price;
    }

    let elapsed = (now - cfg.start_time) as i128;
    let total = (cfg.end_time - cfg.start_time) as i128;

    // remaining_fraction = (total - elapsed) / total
    // Approximate e^{-λt} with (remaining_fraction)^2 for a gentler initial
    // decline and sharper price drop near the auction end.
    let scale: i128 = 1_000_000;
    let remaining = total - elapsed;
    // fraction = remaining / total, scaled
    let fraction_scaled = remaining * scale / total;
    // squared approximation
    let exp_factor = fraction_scaled * fraction_scaled / scale;

    let spread = cfg.start_price - cfg.reserve_price;
    cfg.reserve_price + (spread * exp_factor / scale)
}

/// Dispatch to the correct decay function based on `cfg.exponential`.
pub fn current_dutch_price(cfg: &DutchAuctionConfig, now: u64) -> i128 {
    if cfg.exponential {
        current_dutch_price_exponential(cfg, now)
    } else {
        current_dutch_price_linear(cfg, now)
    }
}

// ──────────────────────────────────────────────────────────────────────────────
// Commit-Reveal Protocol
// ──────────────────────────────────────────────────────────────────────────────

/// **Phase 1 – Commit.**
///
/// The buyer submits a hash of their secret purchase intent.  The current
/// auction price is locked into the commit record so it can be validated
/// against the slippage tolerance at reveal time.
///
/// Hash format: `SHA-256(secret || buyer_address_bytes || price_le_bytes)`
///
/// # Arguments
/// * `env`             – Soroban environment
/// * `event_id`        – the event this ticket belongs to
/// * `tier_id`         – the ticket tier being purchased
/// * `buyer`           – buyer's Stellar address (must `require_auth()`)
/// * `commit_hash`     – `SHA-256(secret || buyer || price)` pre-computed by client
/// * `max_slippage_bps`– max acceptable price increase in bps (e.g. 300 = 3 %)
pub fn commit_purchase(
    env: &Env,
    event_id: &String,
    tier_id: &String,
    buyer: &Address,
    commit_hash: BytesN<32>,
    max_slippage_bps: u32,
) -> Result<i128, TicketPaymentError> {
    buyer.require_auth();

    if max_slippage_bps > MAX_BPS {
        return Err(TicketPaymentError::InvalidSlippageBps);
    }

    let cfg =
        get_dutch_auction(env, event_id, tier_id).ok_or(TicketPaymentError::AuctionNotActive)?;

    let now = env.ledger().timestamp();
    if now >= cfg.end_time {
        return Err(TicketPaymentError::AuctionEnded);
    }
    if now < cfg.start_time {
        return Err(TicketPaymentError::AuctionNotActive);
    }

    let committed_price = current_dutch_price(&cfg, now);

    let commit = PendingCommit {
        hash: commit_hash,
        expires_at: now + COMMIT_EXPIRY_SECS,
        committed_price,
        max_slippage_bps,
    };

    set_pending_commit(env, event_id, tier_id, buyer, &commit);

    Ok(committed_price)
}

/// **Phase 2 – Reveal.**
///
/// Verifies the commit hash and price tolerance, then returns the locked-in
/// price for the calling contract function to process the actual payment.
///
/// This function does **not** perform the token transfer itself; it is intended
/// to be called inside the broader `process_payment` flow which handles the
/// Stellar asset transfer.
///
/// # Arguments
/// * `env`          – Soroban environment
/// * `event_id`     – the event this ticket belongs to
/// * `tier_id`      – the ticket tier being purchased
/// * `buyer`        – buyer's Stellar address (must `require_auth()`)
/// * `secret`       – the plaintext secret used when building `commit_hash`
/// * `price_stroops`– the price the buyer agreed to at commit time
///
/// # Returns
/// `Ok(approved_price)` – the price that should be charged for this purchase.
pub fn reveal_purchase(
    env: &Env,
    event_id: &String,
    tier_id: &String,
    buyer: &Address,
    secret: soroban_sdk::Bytes,
    price_stroops: i128,
) -> Result<i128, TicketPaymentError> {
    buyer.require_auth();

    let commit = take_pending_commit(env, event_id, tier_id, buyer)
        .ok_or(TicketPaymentError::InvalidSecret)?;

    let now = env.ledger().timestamp();
    if now > commit.expires_at {
        return Err(TicketPaymentError::CommitExpired);
    }

    // Reconstruct hash: SHA-256(secret || buyer_address_bytes || price_le_bytes)
    let expected_hash = build_commit_hash(env, &secret, buyer, price_stroops);
    if expected_hash != commit.hash {
        return Err(TicketPaymentError::InvalidSecret);
    }

    // Verify the buyer supplied the same price they committed to.
    if price_stroops != commit.committed_price {
        return Err(TicketPaymentError::PriceMismatch);
    }

    // Check the current auction price is still within the buyer's slippage tolerance.
    let cfg =
        get_dutch_auction(env, event_id, tier_id).ok_or(TicketPaymentError::AuctionNotActive)?;

    let current_price = current_dutch_price(&cfg, now);

    // Price must not have dropped so much that we over-charge (protect buyer),
    // nor risen above their slippage ceiling.
    let ceiling = commit.committed_price
        + (commit.committed_price * commit.max_slippage_bps as i128 / MAX_BPS as i128);

    if current_price > ceiling {
        return Err(TicketPaymentError::PriceOutsideSlippage);
    }

    // Charge at most the current price (buyer benefits from any price drop).
    let approved_price = current_price.min(commit.committed_price);

    Ok(approved_price)
}

// ──────────────────────────────────────────────────────────────────────────────
// Internal helpers
// ──────────────────────────────────────────────────────────────────────────────

/// Build the commit hash the contract expects:
/// `SHA-256(secret || buyer_address_bytes || price_le_bytes)`
fn build_commit_hash(
    env: &Env,
    secret: &soroban_sdk::Bytes,
    buyer: &Address,
    price_stroops: i128,
) -> BytesN<32> {
    let mut preimage = soroban_sdk::Bytes::new(env);
    preimage.append(secret);

    // Append the raw bytes of the buyer address via its XDR encoding.
    let buyer_bytes = buyer.to_xdr(env);
    preimage.append(&buyer_bytes);

    // Append the price as little-endian 16-byte representation.
    let price_bytes = i128_to_le_bytes(env, price_stroops);
    preimage.append(&price_bytes);

    env.crypto().sha256(&preimage).into()
}

/// Encode an `i128` value as a 16-byte little-endian `soroban_sdk::Bytes`.
fn i128_to_le_bytes(env: &Env, value: i128) -> soroban_sdk::Bytes {
    let raw = value.to_le_bytes(); // [u8; 16]
    let mut bytes = soroban_sdk::Bytes::new(env);
    for b in raw.iter() {
        bytes.push_back(*b);
    }
    bytes
}

// ──────────────────────────────────────────────────────────────────────────────
// Unit Tests
// ──────────────────────────────────────────────────────────────────────────────

#[cfg(test)]
mod tests {
    use super::*;

    fn make_cfg(start: i128, reserve: i128, t0: u64, t1: u64) -> DutchAuctionConfig {
        DutchAuctionConfig {
            start_price: start,
            reserve_price: reserve,
            start_time: t0,
            end_time: t1,
            exponential: false,
        }
    }

    #[test]
    fn linear_price_at_start() {
        let cfg = make_cfg(1_000_000, 100_000, 0, 1000);
        assert_eq!(current_dutch_price_linear(&cfg, 0), 1_000_000);
    }

    #[test]
    fn linear_price_at_end() {
        let cfg = make_cfg(1_000_000, 100_000, 0, 1000);
        assert_eq!(current_dutch_price_linear(&cfg, 1000), 100_000);
    }

    #[test]
    fn linear_price_at_midpoint() {
        let cfg = make_cfg(1_000_000, 100_000, 0, 1000);
        let mid = current_dutch_price_linear(&cfg, 500);
        // Midpoint: 1_000_000 - 0.5 * 900_000 = 550_000
        assert_eq!(mid, 550_000);
    }

    #[test]
    fn linear_price_never_below_reserve() {
        let cfg = make_cfg(1_000_000, 100_000, 0, 1000);
        assert_eq!(current_dutch_price_linear(&cfg, 9999), 100_000);
    }

    #[test]
    fn exponential_price_at_start_is_start_price() {
        let mut cfg = make_cfg(1_000_000, 100_000, 0, 1000);
        cfg.exponential = true;
        assert_eq!(current_dutch_price_exponential(&cfg, 0), 1_000_000);
    }

    #[test]
    fn exponential_price_at_end_is_reserve() {
        let mut cfg = make_cfg(1_000_000, 100_000, 0, 1000);
        cfg.exponential = true;
        assert_eq!(current_dutch_price_exponential(&cfg, 1000), 100_000);
    }

    #[test]
    fn exponential_price_midpoint_above_linear_midpoint() {
        // Exponential decay should be slower at first (price stays higher longer)
        let cfg_lin = make_cfg(1_000_000, 100_000, 0, 1000);
        let mut cfg_exp = make_cfg(1_000_000, 100_000, 0, 1000);
        cfg_exp.exponential = true;

        let p_lin = current_dutch_price_linear(&cfg_lin, 500);
        let p_exp = current_dutch_price_exponential(&cfg_exp, 500);

        // Exponential decay keeps price higher at the midpoint
        assert!(p_exp > p_lin, "exp={} should be > lin={}", p_exp, p_lin);
    }

    #[test]
    fn validate_rejects_bad_prices() {
        use soroban_sdk::testutils::Ledger;
        let env = soroban_sdk::Env::default();
        env.ledger().set_timestamp(1000);

        let bad_reserve = DutchAuctionConfig {
            start_price: 100,
            reserve_price: -1,
            start_time: 2000,
            end_time: 3000,
            exponential: false,
        };
        assert!(matches!(
            validate_dutch_auction_config(&env, &bad_reserve),
            Err(TicketPaymentError::InvalidPrice)
        ));

        let inverted_prices = DutchAuctionConfig {
            start_price: 100,
            reserve_price: 200,
            start_time: 2000,
            end_time: 3000,
            exponential: false,
        };
        assert!(matches!(
            validate_dutch_auction_config(&env, &inverted_prices),
            Err(TicketPaymentError::InvalidPrice)
        ));

        let zero_duration = DutchAuctionConfig {
            start_price: 1000,
            reserve_price: 100,
            start_time: 2000,
            end_time: 2000,
            exponential: false,
        };
        assert!(matches!(
            validate_dutch_auction_config(&env, &zero_duration),
            Err(TicketPaymentError::InvalidPrice)
        ));
    }

    #[test]
    fn validate_rejects_already_ended_auction() {
        use soroban_sdk::testutils::Ledger;
        let env = soroban_sdk::Env::default();
        // Set ledger time to 5000; auction ends at 3000 – already over.
        env.ledger().set_timestamp(5000);

        let ended = DutchAuctionConfig {
            start_price: 1000,
            reserve_price: 100,
            start_time: 1000,
            end_time: 3000,
            exponential: false,
        };
        assert!(matches!(
            validate_dutch_auction_config(&env, &ended),
            Err(TicketPaymentError::AuctionEnded)
        ));
    }
}
