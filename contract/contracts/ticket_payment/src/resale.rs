use crate::{error::TicketPaymentError, types::DataKey};
use soroban_sdk::{contracttype, Address, Env, String};

/// Default royalty in basis points paid to the event organizer on each resale (5%).
pub const DEFAULT_RESALE_ROYALTY_BPS: u32 = 500;

/// Maximum royalty in basis points the admin can configure (50%).
pub const MAX_RESALE_ROYALTY_BPS: u32 = 5_000;

#[contracttype]
#[derive(Clone, Debug, Eq, PartialEq)]
pub enum ResaleStatus {
    Active,
    Cancelled,
    Completed,
    /// Alias for Completed used in some tests.
    Sold,
}

#[contracttype]
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct ResaleListing {
    pub payment_id: String,
    pub event_id: String,
    pub seller: Address,
    /// Asking price in token base units. Also stored as `price` for convenience.
    pub ask_price: i128,
    /// Duplicate of ask_price kept for API compatibility with listing accessor tests.
    pub price: i128,
    /// The validated cap at listing time (ask_price after cap enforcement).
    pub max_price: i128,
    /// Royalty rate in basis points captured at listing time.
    pub royalty_bps: u32,
    pub status: ResaleStatus,
    pub listed_at: u64,
}

pub fn get_resale_listing(env: &Env, payment_id: String) -> Option<ResaleListing> {
    env.storage()
        .persistent()
        .get(&DataKey::ResaleListing(payment_id))
}

pub fn store_resale_listing(env: &Env, listing: &ResaleListing) {
    env.storage()
        .persistent()
        .set(&DataKey::ResaleListing(listing.payment_id.clone()), listing);
}

pub fn remove_resale_listing(env: &Env, payment_id: String) {
    env.storage()
        .persistent()
        .remove(&DataKey::ResaleListing(payment_id));
}

pub fn get_resale_royalty_bps(env: &Env, event_id: String) -> u32 {
    env.storage()
        .persistent()
        .get(&DataKey::ResaleRoyaltyBps(event_id))
        .unwrap_or(DEFAULT_RESALE_ROYALTY_BPS)
}

pub fn set_resale_royalty_bps(
    env: &Env,
    event_id: String,
    bps: u32,
) -> Result<(), TicketPaymentError> {
    if bps > MAX_RESALE_ROYALTY_BPS {
        return Err(TicketPaymentError::InvalidRoyaltyBps);
    }
    env.storage()
        .persistent()
        .set(&DataKey::ResaleRoyaltyBps(event_id), &bps);
    Ok(())
}
