use soroban_sdk::{contracttype, Address};

use crate::types::SubscriptionTier;

/// Event types emitted by the Pro Subscription contract
#[contracttype]
#[derive(Clone, Debug, Eq, PartialEq)]
pub enum ProSubscriptionEvent {
    /// Contract initialized
    ContractInitialized,
    /// New subscription created
    SubscriptionCreated,
    /// Subscription renewed
    SubscriptionRenewed,
    /// Subscription cancelled
    SubscriptionCancelled,
    /// Subscription expired
    SubscriptionExpired,
    /// Pro monthly price updated
    PriceUpdated,
    /// Organizer added to the pro members list
    ProMemberAdded,
    /// Organizer removed from the pro members list
    ProMemberRemoved,
}

/// Emitted when the contract is initialized
#[contracttype]
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct InitializationEvent {
    /// The contract admin address set at initialization
    pub admin: Address,
    /// The platform wallet address that receives subscription payments
    pub platform_wallet: Address,
    /// The payment token contract address (e.g., USDC)
    pub payment_token: Address,
    /// The initial monthly Pro subscription price in stroops
    pub pro_monthly_price: i128,
    /// Ledger timestamp of initialization
    pub timestamp: u64,
}

/// Emitted when a new subscription is created
#[contracttype]
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct SubscriptionCreatedEvent {
    /// The organizer's wallet address
    pub organizer: Address,
    /// The subscription tier purchased
    pub tier: SubscriptionTier,
    /// Total amount paid in stroops
    pub amount_paid: i128,
    /// Ledger timestamp when the subscription expires
    pub expires_at: u64,
    /// Ledger timestamp of the subscription creation
    pub timestamp: u64,
}

/// Emitted when a subscription is renewed
#[contracttype]
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct SubscriptionRenewedEvent {
    /// The organizer's wallet address
    pub organizer: Address,
    /// Amount paid for the renewal in stroops
    pub amount_paid: i128,
    /// New expiry timestamp after renewal
    pub new_expiry: u64,
    /// Ledger timestamp of the renewal
    pub timestamp: u64,
}

/// Emitted when a subscription is cancelled
#[contracttype]
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct SubscriptionCancelledEvent {
    /// The organizer whose subscription was cancelled
    pub organizer: Address,
    /// The admin address that performed the cancellation
    pub cancelled_by: Address,
    /// Ledger timestamp of the cancellation
    pub timestamp: u64,
}

/// Emitted when the pro monthly price is updated
#[contracttype]
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct PriceUpdatedEvent {
    /// Previous monthly price in stroops
    pub old_price: i128,
    /// New monthly price in stroops
    pub new_price: i128,
    /// Admin address that performed the update
    pub updated_by: Address,
    /// Ledger timestamp of the update
    pub timestamp: u64,
}

/// Emitted when an organizer is added to the pro members list
#[contracttype]
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct ProMemberAddedEvent {
    /// The organizer's wallet address
    pub organizer: Address,
    /// Ledger timestamp of the event
    pub timestamp: u64,
}

/// Emitted when an organizer is removed from the pro members list
#[contracttype]
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct ProMemberRemovedEvent {
    /// The organizer's wallet address
    pub organizer: Address,
    /// Ledger timestamp of the event
    pub timestamp: u64,
}
