use soroban_sdk::{contracttype, Address, String};

pub const TRANSFER_FEE_BPS: u32 = 100;
pub const MAX_BPS: u32 = 10000;

// Re-export DataKey from the dedicated keys module so all existing imports continue to work.
pub use crate::keys::DataKey;

// Re-export payment-specific types from the dedicated payment_types module.
pub use crate::payment_types::{DiscountData, HighestBid, PurchaseOptions};

// Re-export governance-related types from the dedicated governance module.
pub use crate::governance::{ParameterChange, ParameterProposal, ProposalStatus};

#[contracttype]
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct AuctionConfig {
    pub start_price: i128,
    pub end_time: u64,
    pub min_increment: i128,
}

#[contracttype]
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct PriceSchedule {
    pub price: i128,
    pub valid_until: u64,
}

#[contracttype]
#[derive(Clone, Debug, Eq, PartialEq)]
pub enum PaymentStatus {
    Pending,
    Confirmed,
    Refunded,
    Failed,
    CheckedIn,
}

#[contracttype]
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct Payment {
    pub payment_id: String,
    pub event_id: String,
    pub buyer_address: Address,
    pub owner_address: Address, // The recipient who owns the ticket (can be different from buyer)
    pub ticket_tier_id: String,
    pub token_address: Address,
    pub amount: i128, // Payment token amount in stroops
    pub platform_fee: i128,
    pub organizer_amount: i128,
    pub status: PaymentStatus,
    pub transaction_hash: String,
    pub created_at: u64,
    pub confirmed_at: Option<u64>,
    pub refunded_amount: i128,
    pub is_soulbound: bool,
    pub last_checked_in_at: u64,
    pub referral_amount: i128,
    pub referrer: Option<Address>,
}

#[contracttype]
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct EventBalance {
    pub organizer_amount: i128,
    pub total_withdrawn: i128,
    pub platform_fee: i128,
}
