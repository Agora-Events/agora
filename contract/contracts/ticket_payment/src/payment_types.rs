// Payment-related helper types – not part of the primary documentation surface.
#![allow(missing_docs)]
use soroban_sdk::{contracttype, Address};

#[contracttype]
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct DiscountData {
    pub percentage: u32,
    pub expires_at: u64,
    pub max_uses: u32,
    pub current_uses: u32,
}

/// Optional parameters for `process_payment` that bundle rarely-used fields
/// to stay within Soroban's 10-argument limit.
#[contracttype]
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct PurchaseOptions {
    /// SHA-256 preimage for the legacy global discount code system.
    pub code_preimage: Option<soroban_sdk::Bytes>,
    /// Optional referrer address for referral rewards.
    pub referrer: Option<soroban_sdk::Address>,
    /// Per-event limited-time discount code string.
    pub discount_code: Option<soroban_sdk::String>,
    /// Optional affiliate address for direct commission payouts.
    pub affiliate_address: Option<soroban_sdk::Address>,
}

#[contracttype]
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct HighestBid {
    pub bidder: Address,
    pub token_address: Address,
    pub amount: i128,
}

/// Outcome of validating a single ticket within a bulk check-in request.
#[contracttype]
#[derive(Clone, Debug, Eq, PartialEq)]
pub enum BulkCheckInStatus {
    /// The ticket exists and its validation hash matches; it can be checked in.
    Valid,
    /// No payment record is associated with the supplied numeric ticket id.
    NotFound,
    /// The supplied validation hash does not match the stored hash for the ticket.
    InvalidSecret,
    /// The ticket has already been checked in.
    AlreadyCheckedIn,
    /// The ticket is in a status that cannot be checked in (e.g. refunded/failed).
    InvalidStatus,
}

/// Per-ticket result returned by `bulk_check_in_tickets`, paired with the
/// numeric ticket id that was supplied so callers can correlate the response.
#[contracttype]
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct BulkCheckInResult {
    pub ticket_id: u64,
    pub status: BulkCheckInStatus,
}
