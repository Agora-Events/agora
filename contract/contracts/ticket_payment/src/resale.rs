use crate::{error::TicketPaymentError, types::DataKey};
use soroban_sdk::{contracttype, Address, Env, String};

#[contracttype]
#[derive(Clone, Debug, Eq, PartialEq)]
pub enum ResaleStatus {
    Active,
    Cancelled,
    Completed,
}

#[contracttype]
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct ResaleListing {
    pub payment_id: String,
    pub event_id: String,
    pub seller: Address,
    pub ask_price: i128,
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
        .unwrap_or(0u32)
}

pub fn set_resale_royalty_bps(env: &Env, event_id: String, bps: u32) -> Result<(), TicketPaymentError> {
    if bps > crate::types::MAX_BPS {
        return Err(TicketPaymentError::InvalidRoyaltyBps);
    }
    env.storage()
        .persistent()
        .set(&DataKey::ResaleRoyaltyBps(event_id), &bps);
    Ok(())
}
