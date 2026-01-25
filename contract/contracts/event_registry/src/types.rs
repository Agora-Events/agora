use soroban_sdk::{contracttype, Address, String};

/// Represents information about an event in the registry.
#[contracttype]
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct EventInfo {
    /// Unique identifier for the event
    pub event_id: String,
    /// The wallet address of the event organizer
    pub organizer_address: Address,
    /// The address where payments for this event should be routed
    pub payment_address: Address,
    /// The percentage fee taken by the platform (e.g., 5 for 5%)
    pub platform_fee_percent: u32,
    /// Whether the event is currently active and accepting payments
    pub is_active: bool,
    /// Timestamp when the event was created
    pub created_at: u64,
}

/// Payment information for an event
#[contracttype]
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct PaymentInfo {
    /// The address where payments for this event should be routed
    pub payment_address: Address,
    /// The percentage fee taken by the platform
    pub platform_fee_percent: u32,
}

/// Storage keys for the Event Registry contract.
#[contracttype]
pub enum DataKey {
    /// The administrator address for contract management
    Admin,
    /// The global platform fee percentage
    PlatformFee,
    /// Mapping of event_id to EventInfo (Persistent)
    Event(String),
    /// Mapping of organizer_address to a list of their event_ids (Persistent)
    OrganizerEvents(Address),
}
