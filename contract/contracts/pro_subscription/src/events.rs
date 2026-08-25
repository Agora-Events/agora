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
    pub admin: Address,
    pub platform_wallet: Address,
    pub payment_token: Address,
    pub pro_monthly_price: i128,
    pub timestamp: u64,
}

/// Emitted when a new subscription is created
#[contracttype]
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct SubscriptionCreatedEvent {
    pub organizer: Address,
    pub tier: SubscriptionTier,
    pub amount_paid: i128,
    pub expires_at: u64,
    pub timestamp: u64,
}

/// Emitted when a subscription is renewed
#[contracttype]
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct SubscriptionRenewedEvent {
    pub organizer: Address,
    pub amount_paid: i128,
    pub new_expiry: u64,
    pub timestamp: u64,
}

/// Emitted when a subscription is cancelled
#[contracttype]
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct SubscriptionCancelledEvent {
    pub organizer: Address,
    pub cancelled_by: Address,
    pub timestamp: u64,
}

/// Emitted when the pro monthly price is updated
#[contracttype]
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct PriceUpdatedEvent {
    pub old_price: i128,
    pub new_price: i128,
    pub updated_by: Address,
    pub timestamp: u64,
}

/// Emitted when an organizer is added to the pro members list
#[contracttype]
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct ProMemberAddedEvent {
    pub organizer: Address,
    pub timestamp: u64,
}

/// Emitted when an organizer is removed from the pro members list
#[contracttype]
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct ProMemberRemovedEvent {
    pub organizer: Address,
    pub timestamp: u64,
}
