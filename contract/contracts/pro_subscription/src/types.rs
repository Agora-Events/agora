use soroban_sdk::{contracttype, Address};

pub const SECONDS_PER_MONTH: u64 = 30 * 24 * 60 * 60; // 30 days

/// Subscription plan tiers
#[contracttype]
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
#[derive(Copy, Clone, Debug, Eq, PartialEq)]
#[repr(u32)]
pub enum SubscriptionTier {
    /// Basic tier - no benefits
    Basic = 0,
    /// Pro tier - 0% platform fees
    Pro = 1,
}

/// Subscription record for an organizer
#[contracttype]
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct Subscription {
    /// The organizer's wallet address
    pub organizer: Address,
    /// The subscription tier
    pub tier: SubscriptionTier,
    /// Unix timestamp when the subscription started
    pub started_at: u64,
    /// Unix timestamp when the subscription expires (0 = never expires)
    pub expires_at: u64,
    /// Whether the subscription is currently active
    pub is_active: bool,
    /// Total amount paid for this subscription period
    pub amount_paid: i128,
    /// Token used for payment
    pub payment_token: Address,
}

/// Storage keys for the contract
#[contracttype]
pub enum DataKey {
    /// Contract administrator address
    Admin,
    /// Initialization flag
    Initialized,
    /// Platform wallet for receiving subscription payments
    PlatformWallet,
    /// Monthly subscription price in stroops (for Pro tier)
    ProMonthlyPrice,
    /// Subscription record: organizer_address -> Subscription
    Subscription(Address),
    /// List of all active pro members
    ProMembersList,
    /// Total number of active pro subscriptions
    TotalProSubscriptions,
    /// Accepted payment token (e.g., USDC)
    PaymentToken,
}
