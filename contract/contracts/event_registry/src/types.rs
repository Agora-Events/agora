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
    /// IPFS Content Identifier storing rich metadata details
    pub metadata_cid: String,
    /// Maximum number of tickets available for this event (0 = unlimited)
    pub max_supply: i128,
    /// Current number of tickets that have been successfully purchased
    pub current_supply: i128,
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
    /// The platform wallet address for fee collection
    PlatformWallet,
    /// The global platform fee percentage
    PlatformFee,
    /// Initialization flag
    Initialized,
    /// Mapping of event_id to EventInfo (Persistent)
    Event(String),
    /// Mapping of organizer_address to a list of their event_ids (Persistent)
    OrganizerEvents(Address),
    /// The authorized TicketPayment contract address for inventory updates
    TicketPaymentContract,
}
