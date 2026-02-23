use soroban_sdk::{contracttype, Address, String};

#[contracttype]
#[derive(Clone, Debug, Eq, PartialEq)]
pub enum AgoraEvent {
    EventRegistered,
    EventStatusUpdated,
    EventCancelled,
    FeeUpdated,
    ContractInitialized,
    ContractUpgraded,
    MetadataUpdated,
    InventoryIncremented,
    InventoryDecremented,
    OrganizerBlacklisted,
    OrganizerRemovedFromBlacklist,
    EventsSuspended,
    GlobalPromoUpdated,
    EventPostponed,
}

#[contracttype]
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct EventCancelledEvent {
    pub event_id: String,
    pub cancelled_by: Address,
    pub timestamp: u64,
}

#[contracttype]
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct EventRegisteredEvent {
    pub event_id: String,
    pub organizer_address: Address,
    pub payment_address: Address,
    pub timestamp: u64,
}

#[contracttype]
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct EventStatusUpdatedEvent {
    pub event_id: String,
    pub is_active: bool,
    pub updated_by: Address,
    pub timestamp: u64,
}

#[contracttype]
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct FeeUpdatedEvent {
    pub new_fee_percent: u32,
}

#[contracttype]
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct InitializationEvent {
    pub admin_address: Address,
    pub platform_wallet: Address,
    pub platform_fee_percent: u32,
    pub timestamp: u64,
}

#[contracttype]
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct RegistryUpgradedEvent {
    pub admin_address: Address,
    pub timestamp: u64,
}

#[contracttype]
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct MetadataUpdatedEvent {
    pub event_id: String,
    pub new_metadata_cid: String,
    pub updated_by: Address,
    pub timestamp: u64,
}

#[contracttype]
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct InventoryIncrementedEvent {
    pub event_id: String,
    pub new_supply: i128,
    pub timestamp: u64,
}

#[contracttype]
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct InventoryDecrementedEvent {
    pub event_id: String,
    pub new_supply: i128,
    pub timestamp: u64,
}

#[contracttype]
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct OrganizerBlacklistedEvent {
    pub organizer_address: Address,
    pub admin_address: Address,
    pub reason: String,
    pub timestamp: u64,
}

#[contracttype]
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct OrganizerRemovedFromBlacklistEvent {
    pub organizer_address: Address,
    pub admin_address: Address,
    pub reason: String,
    pub timestamp: u64,
}

#[contracttype]
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct EventsSuspendedEvent {
    pub organizer_address: Address,
    pub suspended_event_count: u32,
    pub admin_address: Address,
    pub timestamp: u64,
}

#[contracttype]
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct GlobalPromoUpdatedEvent {
    pub global_promo_bps: u32,
    pub promo_expiry: u64,
    pub admin_address: Address,
    pub timestamp: u64,
}

#[contracttype]
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct EventPostponedEvent {
    pub event_id: String,
    pub organizer_address: Address,
    pub grace_period_end: u64,
    pub timestamp: u64,
}
