use soroban_sdk::{contracttype, Address, String};

#[contracttype]
#[derive(Clone, Debug, Eq, PartialEq)]
pub enum AgoraEvent {
    EventRegistered,
    EventStatusUpdated,
    FeeUpdated,
    ContractInitialized,
    ContractUpgraded,
    MetadataUpdated,
    InventoryIncremented,
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
    pub max_supply: i128,
    pub timestamp: u64,
}
