use soroban_sdk::{contracttype, Address, Map, String, Vec};

/// Represents a ticket tier with its own pricing and supply
#[contracttype]
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct TicketTier {
    /// Name of the tier (e.g., "General", "VIP", "Reserved")
    pub name: String,
    /// Price for this tier in stroops
    pub price: i128,
    /// Maximum tickets available for this tier
    pub tier_limit: i128,
    /// Current number of tickets sold for this tier
    pub current_sold: i128,
    /// Indicates whether tickets in this tier can be refunded by the buyer
    pub is_refundable: bool,
}

/// Represents an early revenue release milestone.
#[contracttype]
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct Milestone {
    /// The number of tickets sold to reach this milestone
    pub sales_threshold: i128,
    /// Percentage of the available revenue to release (in basis points, 10000 = 100%)
    pub release_percent: u32,
}

#[contracttype]
#[derive(Clone, Debug, Eq, PartialEq)]
pub enum EventStatus {
    Active,
    Inactive,
    Cancelled,
}

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
    /// The current status of the event
    pub status: EventStatus,
    /// Timestamp when the event was created
    pub created_at: u64,
    /// IPFS Content Identifier storing rich metadata details
    pub metadata_cid: String,
    /// Maximum number of tickets available for this event (0 = unlimited)
    pub max_supply: i128,
    /// Current number of tickets that have been successfully purchased
    pub current_supply: i128,
    /// Optional milestone plan for early revenue release
    pub milestone_plan: Option<Vec<Milestone>>,
    /// Map of tier_id to TicketTier for multi-tiered pricing
    pub tiers: Map<String, TicketTier>,
    /// Deadline for guests to request a refund (Unix timestamp)
    pub refund_deadline: u64,
    /// Fee deducted from refund amount
    pub restocking_fee: i128,
    /// Optional resale price cap in basis points above face value.
    /// None = no cap (free market), Some(0) = no markup, Some(1000) = max 10% above face value.
    pub resale_cap_bps: Option<u32>,
    /// Indicates whether the event is currently postponed (date shifted)
    /// and in a temporary refund grace period window.
    pub is_postponed: bool,
    /// Timestamp (Unix) when the temporary refund grace period for a
    /// postponed event ends. 0 means no grace period active.
    pub grace_period_end: u64,
}

/// Payment information for an event
#[contracttype]
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct PaymentInfo {
    /// The address where payments for this event should be routed
    pub payment_address: Address,
    /// The percentage fee taken by the platform
    pub platform_fee_percent: u32,
    /// Map of tier_id to TicketTier for multi-tiered pricing
    pub tiers: Map<String, TicketTier>,
}

/// Arguments required to register a new event
#[contracttype]
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct EventRegistrationArgs {
    pub event_id: String,
    pub organizer_address: Address,
    pub payment_address: Address,
    pub metadata_cid: String,
    pub max_supply: i128,
    pub milestone_plan: Option<Vec<Milestone>>,
    pub tiers: Map<String, TicketTier>,
    pub refund_deadline: u64,
    pub restocking_fee: i128,
    /// Optional resale price cap in basis points above face value.
    pub resale_cap_bps: Option<u32>,
}

/// Audit log entry for blacklist actions
#[contracttype]
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct BlacklistAuditEntry {
    /// The organizer address that was blacklisted or removed from blacklist
    pub organizer_address: Address,
    /// Whether the organizer was added (true) or removed (false) from blacklist
    pub added_to_blacklist: bool,
    /// The admin who performed the action
    pub admin_address: Address,
    /// Reason for the blacklist action
    pub reason: String,
    /// Timestamp when the action was performed
    pub timestamp: u64,
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
    /// Mapping of organizer address to blacklist status (Persistent)
    BlacklistedOrganizer(Address),
    /// List of blacklisted organizer addresses for audit purposes (Persistent)
    BlacklistLog,
    /// Global promotional discount in basis points (e.g., 1500 = 15%)
    GlobalPromoBps,
    /// Expiry timestamp for the global promotional discount
    PromoExpiry,
}
