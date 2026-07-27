use soroban_sdk::{contracttype, Address, Map, String, Vec};

/// Role-based access control for event teams.
/// Defines granular permissions for large events with multiple team members.
#[contracttype]
#[derive(Copy, Clone, Debug, Eq, PartialEq)]
#[repr(u32)]
pub enum Role {
    /// Full control over the event: can manage roles, edit all settings, pause/cancel event
    Admin = 1,
    /// Can edit tiers, pause/resume event, but cannot manage roles or cancel event
    Manager = 2,
    /// Can only check in attendees (call check_in functions), no edit permissions
    Scanner = 3,
}

/// Platform-wide category mapping for event discovery.
/// IDs are stable and must not be renumbered once deployed.
#[contracttype]
#[derive(Copy, Clone, Debug, Eq, PartialEq)]
#[repr(u32)]
pub enum Category {
    Music = 1,
    Sports = 2,
    Tech = 3,
    Arts = 4,
    Food = 5,
    Business = 6,
    Health = 7,
    Education = 8,
    Community = 9,
    Other = 10,
}

impl Category {
    /// Returns `Some(Category)` for a valid ID, `None` otherwise.
    pub fn from_id(id: u32) -> Option<Self> {
        match id {
            1 => Some(Self::Music),
            2 => Some(Self::Sports),
            3 => Some(Self::Tech),
            4 => Some(Self::Arts),
            5 => Some(Self::Food),
            6 => Some(Self::Business),
            7 => Some(Self::Health),
            8 => Some(Self::Education),
            9 => Some(Self::Community),
            10 => Some(Self::Other),
            _ => None,
        }
    }
}

/// Represents a series or festival grouping multiple events
#[contracttype]
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct SeriesRegistry {
    /// Unique identifier for the series
    pub series_id: String,
    /// Name or description of the series
    pub name: String,
    /// List of event_ids included in this series
    pub event_ids: Vec<String>,
    /// Organizer address for the series
    pub organizer_address: Address,
    /// Optional metadata (e.g., IPFS CID)
    pub metadata_cid: Option<String>,
}

/// Represents a season pass for a series
#[contracttype]
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct SeriesPass {
    /// Unique pass ID
    pub pass_id: String,
    /// Series this pass is valid for
    pub series_id: String,
    /// Address of the pass holder
    pub holder: Address,
    /// Usage limit (e.g., 5 out of 10 events)
    pub usage_limit: u32,
    /// Number of events attended with this pass
    pub usage_count: u32,
    /// Timestamp when the pass was issued
    pub issued_at: u64,
    /// Expiry timestamp (optional, 0 = no expiry)
    pub expires_at: u64,
}

/// Configuration for an auction ticket tier
#[contracttype]
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct AuctionConfig {
    /// Starting price for the auction in stroops
    pub start_price: i128,
    /// Unix timestamp when the auction ends
    pub end_time: u64,
    /// Minimum increment for a new bid in stroops
    pub min_increment: i128,
}

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
    /// Optional configuration for an auction
    pub auction_config: soroban_sdk::Vec<AuctionConfig>,
    /// Loyalty points multiplier for this tier (e.g., 1 = 1x, 2 = 2x).
    /// A value of 0 is treated as 1x. VIP tiers can award more points.
    pub loyalty_multiplier: u32,
    /// Maximum number of tickets a single user can purchase for this tier
    /// A value of 0 means unlimited (no per-user limit)
    pub max_per_user: u32,
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
    /// Human-readable name for the event (trimmed of leading/trailing whitespace)
    pub name: String,
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
    /// Minimum number of tickets that must be sold for the event to proceed
    pub min_sales_target: i128,
    /// Deadline by which the min_sales_target must be met (Unix timestamp)
    pub target_deadline: u64,
    /// Whether the minimum sales target has been reached
    pub goal_met: bool,
    /// Optional special fee rate for high-volume partners or charitable events (in basis points)
    pub custom_fee_bps: Option<u32>,
    /// Optional IPFS CID for the event banner image
    pub banner_cid: Option<String>,
    /// Optional categorical tags for the event (e.g., "Music", "Tech")
    pub tags: Option<Vec<String>>,
    /// Category IDs for discovery (up to 5). See [`Category`] for the platform-wide mapping.
    pub category_ids: Option<Vec<u32>>,
    /// Unix timestamp when the event starts (0 = not set)
    pub start_time: u64,
    /// Whether the event is private and should be excluded from global public counters.
    /// Private events do not appear in managed event counts, active event counts,
    /// or global tickets sold totals.
    pub is_private: bool,
    /// Unix timestamp when the event ends (0 = not set)
    pub end_time: u64,
    /// Duration in seconds after purchase during which tickets cannot be transferred (0 = no lock)
    pub transfer_lock_duration: u64,
    /// List of whitelisted payment tokens for this event (empty = use global whitelist)
    pub accepted_tokens: Vec<Address>,
    /// Whether to use the global token whitelist instead of event-specific one
    pub use_global_whitelist: bool,
    /// Optional IPFS CID for post-event feedback (only settable after end_time)
    pub feedback_cid: Option<String>,
    /// Optional human-readable reason provided when the event was cancelled
    pub cancellation_reason: Option<String>,
    /// Referral commission rate in basis points (e.g., 500 = 5%)
    pub referral_rate_bps: u32,
}

/// Payment information for an event
#[contracttype]
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct PaymentInfo {
    /// The address where payments for this event should be routed
    pub payment_address: Address,
    /// The percentage fee taken by the platform
    pub platform_fee_percent: u32,
    /// Optional special fee rate for high-volume partners or charitable events
    pub custom_fee_bps: Option<u32>,
    /// Map of tier_id to TicketTier for multi-tiered pricing
    pub tiers: Map<String, TicketTier>,
    /// Referral commission rate in basis points
    pub referral_rate_bps: u32,
}

/// Arguments required to register a new event
#[contracttype]
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct EventRegistrationArgs {
    pub event_id: String,
    /// Human-readable name for the event. Leading/trailing whitespace will be trimmed on registration.
    pub name: String,
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
    /// Minimum number of tickets that must be sold for the event to proceed
    pub min_sales_target: Option<i128>,
    /// Deadline by which the min_sales_target must be met (Unix timestamp)
    pub target_deadline: Option<u64>,
    /// Optional IPFS CID for the event banner image
    pub banner_cid: Option<String>,
    /// Optional categorical tags for the event (e.g., "Music", "Tech")
    pub tags: Option<Vec<String>>,
    /// Category IDs for discovery (up to 5). See [`Category`] for the platform-wide mapping.
    pub category_ids: Option<Vec<u32>>,
    /// Unix timestamp when the event starts (0 = not set)
    pub start_time: u64,
    /// Whether the event is private and should be excluded from global public counters.
    pub is_private: bool,
    /// Unix timestamp when the event ends (0 = not set)
    pub end_time: u64,
    /// Duration in seconds after purchase during which tickets cannot be transferred (0 = no lock)
    pub transfer_lock_duration: u64,
    /// List of whitelisted payment tokens for this event (empty = use global whitelist)
    pub accepted_tokens: Vec<Address>,
    /// Whether to use the global token whitelist instead of event-specific one
    pub use_global_whitelist: bool,
    /// Referral commission rate in basis points (optional)
    pub referral_rate_bps: Option<u32>,
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

/// Minimal receipt for an archived event, preserving basic data for historical lookups.
#[contracttype]
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct EventReceipt {
    pub event_id: String,
    pub organizer_address: Address,
    pub total_sold: i128,
    pub archived_at: u64,
}

/// Multi-signature configuration for admin management
#[contracttype]
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct MultiSigConfig {
    /// List of admin addresses
    pub admins: Vec<Address>,
    /// Number of approvals required to execute a proposal
    pub threshold: u32,
}

/// Represents a governance proposal
#[contracttype]
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct Proposal {
    /// Unique identifier for the proposal
    pub proposal_id: u64,
    /// Address that created the proposal
    pub proposer: Address,
    /// The parameter change being proposed
    pub change: ParameterChange,
    /// Addresses that have approved this proposal
    pub approvals: Vec<Address>,
    /// Whether the proposal has been executed
    pub executed: bool,
    /// Whether the proposal has been cancelled
    pub cancelled: bool,
    /// Timestamp when the proposal was created
    pub created_at: u64,
    /// Timestamp when the proposal expires
    pub expires_at: u64,
}

/// Loyalty profile for a guest (event attendee / ticket buyer)
#[contracttype]
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct GuestProfile {
    /// The wallet address of the guest
    pub guest_address: Address,
    /// Accumulated loyalty score (increases with each purchase)
    pub loyalty_score: u64,
    /// Total number of tickets purchased across all events
    pub total_tickets_purchased: u32,
    /// Total amount spent across all events (in token stroops)
    pub total_spent: i128,
    /// Timestamp of the last loyalty score update
    pub last_updated: u64,
}

/// Represents an organizer's staked collateral for Verified status
#[contracttype]
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct OrganizerStake {
    /// The organizer's wallet address
    pub organizer: Address,
    /// The token contract address used for staking
    pub token: Address,
    /// The amount of tokens staked
    pub amount: i128,
    /// Timestamp when the stake was created
    pub staked_at: u64,
    /// Whether the organizer currently holds Verified status
    pub is_verified: bool,
    /// Accumulated reward balance available to claim
    pub reward_balance: i128,
    /// Total rewards claimed historically
    pub total_rewards_claimed: i128,
}

/// Represents a parameter change proposal for governance
#[contracttype]
#[derive(Clone, Debug, Eq, PartialEq)]
pub enum ParameterChange {
    /// Add a new admin to the multi-sig configuration
    AddAdmin(Address),
    /// Remove an admin from the multi-sig configuration
    RemoveAdmin(Address),
    /// Update the approval threshold for proposals
    SetThreshold(u32),
    /// Update the platform wallet address
    UpdatePlatformWallet(Address),
    /// Update the global platform fee in basis points (0–10000)
    SetPlatformFee(u32),
    /// Update the minimum stake amount required for Verified organizer status
    SetMinStakeAmount(i128),
}

/// Storage keys for the Event Registry contract.
#[contracttype]
pub enum DataKey {
    /// The administrator address for contract management (legacy, kept for backward compatibility)
    Admin,
    /// Multi-signature configuration
    MultiSigConfig,
    /// The platform wallet address for fee collection
    PlatformWallet,
    /// The global platform fee percentage
    PlatformFee,
    /// Initialization flag
    Initialized,
    /// Mapping of event_id to EventInfo (Persistent)
    Event(String),
    /// Individual entry for an organizer's event (Persistent)
    OrganizerEvent(Address, String),
    /// Sharded mapping of organizer address to their event_ids (Persistent)
    OrganizerEventShard(Address, u32),
    /// Total number of events for an organizer (Persistent)
    OrganizerEventCount(Address),
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
    /// Mapping of event_id to EventReceipt (Persistent) for archived events
    EventReceipt(String),
    /// Individual entry for an organizer's archived event receipt (Persistent)
    OrganizerReceipt(Address, String),
    /// Sharded mapping of organizer address to archived event receipt ids (Persistent)
    OrganizerReceiptShard(Address, u32),
    /// Total number of archived event receipts for an organizer (Persistent)
    OrganizerReceiptCount(Address),
    /// Counter for proposal IDs
    ProposalCounter,
    /// Mapping of proposal_id to Proposal
    Proposal(u64),
    /// List of active proposal IDs
    ActiveProposals,
    /// Mapping of (event_id, scanner_address) to bool
    AuthorizedScanner(String, Address),

    /// Mapping of series_id to SeriesRegistry (Persistent)
    Series(String),
    /// Mapping of pass_id to SeriesPass (Persistent)
    SeriesPass(String),
    /// Mapping of (holder, series_id) to pass_id (Persistent)
    HolderSeriesPass(Address, String),
    /// Mapping of (series_id, event_id) to bool (Persistent, for fast lookup)
    SeriesEvent(String, String),

    // ── Loyalty & Staking ──────────────────────────────────────────────
    /// Guest loyalty profile keyed by guest address (Persistent)
    GuestProfile(Address),
    /// Organizer stake record keyed by organizer address (Persistent)
    OrganizerStake(Address),
    /// Minimum token amount required to unlock Verified status
    MinStakeAmount,
    /// Token contract address accepted for staking
    StakingToken,
    /// Sum of all tokens currently staked (i128, Persistent)
    TotalStaked,
    /// List of all currently staked organizer addresses for proportional distribution
    StakersList,
    /// Mapping of token address to whitelist status (Persistent)
    TokenWhitelist(Address),
    /// Mapping of (event_id, token_address) to whitelist status for event-specific tokens (Persistent)
    EventTokenWhitelist(String, Address),
    /// Global counter of all events ever registered on the platform
    GlobalEventCount,
    /// Global counter of currently active events
    GlobalActiveEventCount,
    /// Global counter of all tickets sold across all events
    GlobalTicketsSold,
    /// Mapping of (event_id, tier_id, user_address) to ticket count for per-user limits (Persistent)
    UserTicketCount(String, String, Address),
    /// Mapping of (event_id, user_address) to bool for waitlist membership (Persistent)
    Waitlist(String, Address),
    /// Mapping of event_id to pause status (bool) – whether the event is paused (Persistent)
    EventPaused(String),
    /// The administrator address specifically for organizer whitelisting (Instance)
    ContractAdmin,
    /// Mapping of organizer address to approved status (Instance)
    ApprovedOrganizer(Address),
    /// Index of event_ids tagged with a given category ID (Persistent)
    CategoryEvents(u32),
    /// Mapping of (event_id, team_member_address) to Role for team-based access control (Persistent)
    EventTeamRole(String, Address),
}
