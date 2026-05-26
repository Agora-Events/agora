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
}

#[contracttype]
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct HighestBid {
    pub bidder: Address,
    pub token_address: Address,
    pub amount: i128,
}