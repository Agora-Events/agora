//! # Capped Secondary Market (Resale)
//!
//! Anti-scalping resale for confirmed tickets. A holder lists their ticket at a
//! price that the contract validates against a hard ceiling derived from what
//! they originally paid, and a buyer settles it in a single atomic invocation
//! that moves USDC and ticket ownership together — there is no window in which
//! one has happened without the other.
//!
//! ## Price ceiling
//!
//! The ceiling comes from the organizer's `resale_cap_bps` on the event
//! registry entry. Unlike [`crate::contract::TicketPaymentContract::transfer_ticket`],
//! which treats an unset cap as an uncapped free market, the marketplace falls
//! back to [`DEFAULT_MAX_RESALE_MARKUP_BPS`] so that listings are *always*
//! bounded. Organizers who genuinely want a free market can set a large
//! explicit cap; leaving it unset no longer opts out of scalping protection.
//!
//! Face value is taken from `payment.amount` — the price actually paid for this
//! specific ticket — rather than the tier's current price. Tier prices drift
//! (early-bird expiry, scheduled price steps), and pinning the cap to the live
//! tier price would let a holder who bought at an early-bird discount resell at
//! a markup over the *later*, higher price.
//!
//! ## Royalty
//!
//! Every completed resale routes `royalty_bps` of the sale price to the event's
//! `payment_address` — the same destination primary sales settle to — before
//! the seller is paid. The rate defaults to [`DEFAULT_RESALE_ROYALTY_BPS`] and
//! organizers can change it via `set_resale_royalty_bps`.
//!
//! ## What is deliberately *not* on-chain
//!
//! The ticket's check-in secret is never written here. Ownership transfer alone
//! does not let the buyer through the gate — they also need the secret whose
//! SHA-256 digest is stored as the payment's `ValidationHash`. That secret is
//! handed over off-chain, sealed to the buyer's X25519 public key, via the
//! server's marketplace key-envelope endpoints (`server/src/handlers/marketplace.rs`).
//! The contract is the settlement layer; the server is a blind relay.

use soroban_sdk::{contracttype, token, Address, Env, String};

use crate::contract::{event_registry, validate_recipient};
use crate::error::TicketPaymentError;
use crate::events::{AgoraEvent, ResaleCancelledEvent, ResaleListedEvent, ResalePurchasedEvent};
use crate::storage::{
    add_payment_to_buyer_index, add_to_total_volume_processed, get_event_registry, get_payment,
    is_initialized, is_paused, remove_payment_from_buyer_index,
};
use crate::types::{DataKey, PaymentStatus, MAX_BPS};

/// Markup ceiling applied when an organizer has not set `resale_cap_bps`.
/// 1000 bps = 110% of face value, the example given in the protocol spec.
pub const DEFAULT_MAX_RESALE_MARKUP_BPS: u32 = 1000;

/// Organizer royalty applied when an event has no explicit rate. 500 bps = 5%.
pub const DEFAULT_RESALE_ROYALTY_BPS: u32 = 500;

/// Upper bound on a configurable royalty. Anything higher would leave the
/// seller with less than half the sale price and is almost certainly a
/// fat-finger, so it is rejected at configuration time rather than at sale time.
pub const MAX_RESALE_ROYALTY_BPS: u32 = 5000;

#[contracttype]
#[derive(Clone, Debug, Eq, PartialEq)]
pub enum ResaleStatus {
    /// Listed and purchasable.
    Active,
    /// Withdrawn by the seller before it sold.
    Cancelled,
    /// Settled — ownership has moved to the buyer.
    Sold,
}

/// A secondary-market listing. Keyed by the `payment_id` of the ticket being
/// sold, so a ticket can only ever have one listing at a time.
#[contracttype]
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct ResaleListing {
    pub payment_id: String,
    pub event_id: String,
    pub seller: Address,
    /// Asking price in `token_address` base units.
    pub price: i128,
    /// Token the buyer must pay in — pinned to the token of the original
    /// purchase so a listing cannot be settled in a cheaper asset.
    pub token_address: Address,
    /// Ceiling this listing was validated against, recorded so clients can
    /// render remaining headroom without re-deriving it from the registry.
    pub max_price: i128,
    /// Royalty rate captured at listing time.
    pub royalty_bps: u32,
    pub status: ResaleStatus,
    pub created_at: u64,
    pub updated_at: u64,
}

// ── Storage accessors ────────────────────────────────────────────────────────

pub fn get_listing(env: &Env, payment_id: &String) -> Option<ResaleListing> {
    env.storage()
        .persistent()
        .get(&DataKey::ResaleListing(payment_id.clone()))
}

fn set_listing(env: &Env, listing: &ResaleListing) {
    env.storage()
        .persistent()
        .set(&DataKey::ResaleListing(listing.payment_id.clone()), listing);
}

pub fn get_royalty_bps(env: &Env, event_id: &String) -> u32 {
    env.storage()
        .persistent()
        .get(&DataKey::ResaleRoyaltyBps(event_id.clone()))
        .unwrap_or(DEFAULT_RESALE_ROYALTY_BPS)
}

/// Sets the organizer royalty for an event. Caller must be the organizer.
pub fn set_royalty_bps(
    env: &Env,
    event_id: &String,
    royalty_bps: u32,
) -> Result<(), TicketPaymentError> {
    if !is_initialized(env) {
        return Err(TicketPaymentError::NotInitialized);
    }
    if royalty_bps > MAX_RESALE_ROYALTY_BPS {
        return Err(TicketPaymentError::InvalidRoyaltyBps);
    }

    let event_info = load_event(env, event_id)?;
    event_info.organizer_address.require_auth();

    env.storage()
        .persistent()
        .set(&DataKey::ResaleRoyaltyBps(event_id.clone()), &royalty_bps);
    Ok(())
}

// ── Pricing ─────────────────────────────────────────────────────────────────

fn load_event(
    env: &Env,
    event_id: &String,
) -> Result<event_registry::EventInfo, TicketPaymentError> {
    let registry_client = event_registry::Client::new(env, &get_event_registry(env));
    match registry_client.try_get_event(event_id) {
        Ok(Ok(Some(info))) => Ok(info),
        _ => Err(TicketPaymentError::EventNotFound),
    }
}

/// Highest price a ticket bought for `face_value` may be resold at.
///
/// Returns `face_value * (10000 + cap_bps) / 10000`, using the organizer's
/// `resale_cap_bps` when set and [`DEFAULT_MAX_RESALE_MARKUP_BPS`] otherwise.
pub fn max_resale_price(
    face_value: i128,
    resale_cap_bps: Option<u32>,
) -> Result<i128, TicketPaymentError> {
    let cap_bps = resale_cap_bps.unwrap_or(DEFAULT_MAX_RESALE_MARKUP_BPS);

    let multiplier = (MAX_BPS as i128)
        .checked_add(cap_bps as i128)
        .ok_or(TicketPaymentError::ArithmeticError)?;

    face_value
        .checked_mul(multiplier)
        .ok_or(TicketPaymentError::ArithmeticError)?
        .checked_div(MAX_BPS as i128)
        .ok_or(TicketPaymentError::ArithmeticError)
}

/// Splits a sale price into the organizer's royalty and the seller's net
/// proceeds. Rounds the royalty down, so any rounding dust favours the seller.
pub fn split_proceeds(price: i128, royalty_bps: u32) -> Result<(i128, i128), TicketPaymentError> {
    let royalty = price
        .checked_mul(royalty_bps as i128)
        .ok_or(TicketPaymentError::ArithmeticError)?
        .checked_div(MAX_BPS as i128)
        .ok_or(TicketPaymentError::ArithmeticError)?;

    let seller_proceeds = price
        .checked_sub(royalty)
        .ok_or(TicketPaymentError::ArithmeticError)?;

    Ok((royalty, seller_proceeds))
}

// ── Shared guards ───────────────────────────────────────────────────────────

/// Rejects events that must not host secondary trading: inactive, cancelled,
/// disputed, or already over.
fn require_tradeable_event(
    env: &Env,
    event_info: &event_registry::EventInfo,
) -> Result<(), TicketPaymentError> {
    if matches!(event_info.status, event_registry::EventStatus::Cancelled) {
        return Err(TicketPaymentError::EventCancelled);
    }
    if !event_info.is_active {
        return Err(TicketPaymentError::EventInactive);
    }
    let registry_addr = crate::storage::get_event_registry(env);
    let registry_client = event_registry::Client::new(env, &registry_addr);
    if let Ok(Ok(Some(dispute))) = registry_client.try_get_dispute(&event_info.event_id) {
        if matches!(dispute.status, event_registry::DisputeStatus::Open)
            || matches!(dispute.status, event_registry::DisputeStatus::Voting)
        {
            return Err(TicketPaymentError::EventDisputed);
        }
    }
    if event_info.end_time > 0 && env.ledger().timestamp() >= event_info.end_time {
        return Err(TicketPaymentError::EventEnded);
    }
    Ok(())
}

// ── list_for_resale ─────────────────────────────────────────────────────────

/// Lists a confirmed, transferable ticket on the secondary market.
///
/// Authorized by the current holder. Fails if the asking price exceeds the
/// event's ceiling, if the ticket has already been checked in, or if the
/// ticket already has an active listing.
pub fn list_for_resale(
    env: &Env,
    payment_id: String,
    price: i128,
) -> Result<ResaleListing, TicketPaymentError> {
    if !is_initialized(env) {
        return Err(TicketPaymentError::NotInitialized);
    }
    if is_paused(env) {
        return Err(TicketPaymentError::ContractPaused);
    }
    if price <= 0 {
        return Err(TicketPaymentError::InvalidPrice);
    }

    let payment =
        get_payment(env, payment_id.clone()).ok_or(TicketPaymentError::PaymentNotFound)?;

    // Only a live, unused ticket can be sold on. `CheckedIn` is explicitly
    // excluded: the ticket has already been consumed at the gate.
    if payment.status != PaymentStatus::Confirmed {
        return Err(TicketPaymentError::InvalidPaymentStatus);
    }
    if payment.is_soulbound {
        return Err(TicketPaymentError::NonTransferable);
    }

    let seller = payment.buyer_address.clone();
    seller.require_auth();

    if let Some(existing) = get_listing(env, &payment_id) {
        if existing.status == ResaleStatus::Active {
            return Err(TicketPaymentError::TicketAlreadyListed);
        }
    }

    let event_info = load_event(env, &payment.event_id)?;
    require_tradeable_event(env, &event_info)?;

    let max_price = max_resale_price(payment.amount, event_info.resale_cap_bps)?;
    if price > max_price {
        return Err(TicketPaymentError::ResalePriceExceedsCap);
    }

    let now = env.ledger().timestamp();
    let listing = ResaleListing {
        payment_id: payment_id.clone(),
        event_id: payment.event_id.clone(),
        seller: seller.clone(),
        price,
        token_address: payment.token_address.clone(),
        max_price,
        royalty_bps: get_royalty_bps(env, &payment.event_id),
        status: ResaleStatus::Active,
        created_at: now,
        updated_at: now,
    };
    set_listing(env, &listing);

    #[allow(deprecated)]
    env.events().publish(
        (AgoraEvent::ResaleListed,),
        ResaleListedEvent {
            payment_id,
            event_id: payment.event_id,
            seller,
            price,
            max_price,
            timestamp: now,
        },
    );

    Ok(listing)
}

// ── cancel_resale_listing ───────────────────────────────────────────────────

/// Withdraws an active listing. Only the seller who created it may cancel.
pub fn cancel_resale_listing(env: &Env, payment_id: String) -> Result<(), TicketPaymentError> {
    if !is_initialized(env) {
        return Err(TicketPaymentError::NotInitialized);
    }

    let mut listing =
        get_listing(env, &payment_id).ok_or(TicketPaymentError::ResaleListingNotFound)?;

    if listing.status != ResaleStatus::Active {
        return Err(TicketPaymentError::ResaleListingNotActive);
    }

    listing.seller.require_auth();

    let now = env.ledger().timestamp();
    listing.status = ResaleStatus::Cancelled;
    listing.updated_at = now;
    set_listing(env, &listing);

    #[allow(deprecated)]
    env.events().publish(
        (AgoraEvent::ResaleCancelled,),
        ResaleCancelledEvent {
            payment_id,
            event_id: listing.event_id.clone(),
            seller: listing.seller.clone(),
            timestamp: now,
        },
    );

    Ok(())
}

// ── purchase_resale_ticket ──────────────────────────────────────────────────

/// Settles a listing atomically: pulls the full price from the buyer, pays the
/// organizer royalty and the seller's net proceeds, and moves ticket ownership
/// — all inside one invocation, so a failure at any step reverts the whole
/// swap and neither the money nor the ticket moves.
///
/// The buyer must have granted this contract a USDC allowance covering `price`
/// beforehand, matching the approve-then-call pattern `process_payment` uses.
pub fn purchase_resale_ticket(
    env: &Env,
    payment_id: String,
    buyer: Address,
) -> Result<ResaleListing, TicketPaymentError> {
    if !is_initialized(env) {
        return Err(TicketPaymentError::NotInitialized);
    }
    if is_paused(env) {
        return Err(TicketPaymentError::ContractPaused);
    }

    let mut listing =
        get_listing(env, &payment_id).ok_or(TicketPaymentError::ResaleListingNotFound)?;
    if listing.status != ResaleStatus::Active {
        return Err(TicketPaymentError::ResaleListingNotActive);
    }

    buyer.require_auth();

    let mut payment =
        get_payment(env, payment_id.clone()).ok_or(TicketPaymentError::PaymentNotFound)?;
    if payment.status != PaymentStatus::Confirmed {
        return Err(TicketPaymentError::InvalidPaymentStatus);
    }
    if payment.is_soulbound {
        return Err(TicketPaymentError::NonTransferable);
    }

    // The listing is stale if the ticket moved on by some other route (a direct
    // `transfer_ticket`, say) after it was listed. Refuse rather than pay a
    // seller who no longer holds the ticket.
    let seller = listing.seller.clone();
    if payment.buyer_address != seller {
        return Err(TicketPaymentError::ResaleListingNotActive);
    }
    if buyer == seller {
        return Err(TicketPaymentError::InvalidAddress);
    }
    validate_recipient(env, &buyer)?;

    let event_info = load_event(env, &listing.event_id)?;
    require_tradeable_event(env, &event_info)?;

    // Re-check the ceiling at settlement time: the organizer may have tightened
    // `resale_cap_bps` after this listing went up, and the tighter cap wins.
    let max_price = max_resale_price(payment.amount, event_info.resale_cap_bps)?;
    if listing.price > max_price {
        return Err(TicketPaymentError::ResalePriceExceedsCap);
    }

    let (royalty, seller_proceeds) = split_proceeds(listing.price, listing.royalty_bps)?;

    // ── Atomic settlement ────────────────────────────────────────────────
    let token_client = token::Client::new(env, &listing.token_address);
    let contract_address = env.current_contract_address();

    if token_client.allowance(&buyer, &contract_address) < listing.price {
        return Err(TicketPaymentError::InsufficientAllowance);
    }

    let balance_before = token_client.balance(&contract_address);
    token_client.transfer_from(&contract_address, &buyer, &contract_address, &listing.price);
    let balance_after = token_client.balance(&contract_address);

    if balance_after
        .checked_sub(balance_before)
        .ok_or(TicketPaymentError::ArithmeticError)?
        != listing.price
    {
        return Err(TicketPaymentError::TransferVerificationFailed);
    }

    if royalty > 0 {
        token_client.transfer(&contract_address, &event_info.payment_address, &royalty);
    }
    if seller_proceeds > 0 {
        token_client.transfer(&contract_address, &seller, &seller_proceeds);
    }

    // ── Ownership transfer ───────────────────────────────────────────────
    payment.buyer_address = buyer.clone();
    payment.owner_address = buyer.clone();
    env.storage()
        .persistent()
        .set(&DataKey::Payment(payment_id.clone()), &payment);

    remove_payment_from_buyer_index(env, seller.clone(), payment_id.clone());
    add_payment_to_buyer_index(env, buyer.clone(), payment_id.clone());

    add_to_total_volume_processed(env, listing.price);

    let now = env.ledger().timestamp();
    listing.status = ResaleStatus::Sold;
    listing.updated_at = now;
    set_listing(env, &listing);

    #[allow(deprecated)]
    env.events().publish(
        (AgoraEvent::ResalePurchased,),
        ResalePurchasedEvent {
            payment_id,
            event_id: listing.event_id.clone(),
            seller,
            buyer,
            price: listing.price,
            royalty,
            seller_proceeds,
            timestamp: now,
        },
    );

    Ok(listing)
}
