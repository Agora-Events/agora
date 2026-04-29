use soroban_sdk::{contracttype, Address, String};

/// Enum of all contract event types emitted by the Event Registry.
///
/// Each variant corresponds to a specific action or state change within the
/// platform. These are used as the first element of the event topic tuple when
/// publishing via `env.events().publish(...)`.
#[contracttype]
#[derive(Clone, Debug, Eq, PartialEq)]
pub enum AgoraEvent {
    /// A new event has been registered on the platform.
    EventRegistered,
    /// An event's active/inactive status has been toggled by its organizer.
    EventStatusUpdated,
    /// An event has been permanently cancelled by its organizer.
    EventCancelled,
    /// The global platform fee percentage has been updated by the admin.
    FeeUpdated,
    /// The contract has been initialized with admin, wallet, fee, and token settings.
    ContractInitialized,
    /// The contract WASM has been upgraded to a new version.
    ContractUpgraded,
    /// An event's IPFS metadata CID has been updated by its organizer.
    MetadataUpdated,
    /// Ticket inventory has been incremented (tickets sold) for an event/tier.
    InventoryIncremented,
    /// Ticket inventory has been decremented (refund processed) for an event/tier.
    InventoryDecremented,
    /// An organizer has been added to the platform blacklist by an admin.
    OrganizerBlacklisted,
    /// An organizer has been removed from the platform blacklist by an admin.
    OrganizerRemovedFromBlacklist,
    /// All active events for a blacklisted organizer have been suspended.
    EventsSuspended,
    /// The global promotional discount rate and expiry have been updated.
    GlobalPromoUpdated,
    /// An event has been marked as postponed with a refund grace period.
    EventPostponed,
    /// An event has been archived and its storage reclaimed.
    EventArchived,
    /// A scanner wallet has been authorized for ticket validation at an event.
    ScannerAuthorized,
    /// An event's minimum sales target has been reached.
    GoalMet,
    /// An organizer has staked collateral tokens toward Verified status.
    CollateralStaked,
    /// An organizer has withdrawn their staked collateral tokens.
    CollateralUnstaked,
    /// Staking rewards have been distributed proportionally to all active stakers.
    StakerRewardsDistributed,
    /// An organizer has claimed their accumulated staking rewards.
    StakerRewardsClaimed,
    /// A guest's loyalty score has been updated after a ticket purchase.
    LoyaltyScoreUpdated,
    /// A custom fee override has been set for a specific event by an admin.
    CustomFeeSet,
    AdminUpdated,
    /// Post-event feedback CID has been set by the organizer after event end_time.
    FeedbackCidSet,
    /// An event's token whitelist has been updated (token added or removed).
    TokenWhitelistUpdated,
    /// A governance proposal has been cancelled by the proposer.
    ProposalCancelled,
    /// A user has joined the waitlist for a sold-out event.
    WaitlistJoined,
}

/// Emitted when an event is permanently cancelled.
///
/// Published with topic `(AgoraEvent::EventCancelled,)`.
#[contracttype]
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct EventCancelledEvent {
    /// The unique identifier of the cancelled event.
    pub event_id: String,
    /// The address of the organizer who cancelled the event.
    pub cancelled_by: Address,
    /// The ledger timestamp when the cancellation occurred.
    pub timestamp: u64,
    /// Optional human-readable reason for the cancellation.
    pub reason: Option<String>,
}

/// Emitted when an event is archived and its full storage is reclaimed.
///
/// After archival, only a lightweight `EventReceipt` remains for historical lookups.
/// Published with topic `(AgoraEvent::EventArchived,)`.
#[contracttype]
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct EventArchivedEvent {
    /// The unique identifier of the archived event.
    pub event_id: String,
    /// The address of the event organizer.
    pub organizer_address: Address,
    /// The ledger timestamp when the archival occurred.
    pub timestamp: u64,
}

/// Emitted when a new event is successfully registered on the platform.
///
/// Published with topic `(AgoraEvent::EventRegistered,)`.
#[contracttype]
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct EventRegisteredEvent {
    /// The unique identifier assigned to the new event.
    pub event_id: String,
    /// The wallet address of the event organizer.
    pub organizer_address: Address,
    /// The address where ticket payments for this event will be routed.
    pub payment_address: Address,
    /// The ledger timestamp when the event was registered.
    pub timestamp: u64,
}

/// Emitted when an event's active status is toggled by its organizer.
///
/// Published with topic `(AgoraEvent::EventStatusUpdated,)`.
#[contracttype]
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct EventStatusUpdatedEvent {
    /// The unique identifier of the event whose status changed.
    pub event_id: String,
    /// The new active status (`true` = active, `false` = inactive).
    pub is_active: bool,
    /// The address of the organizer who updated the status.
    pub updated_by: Address,
    /// The ledger timestamp when the status change occurred.
    pub timestamp: u64,
}

/// Emitted when the global platform fee percentage is updated.
///
/// Published with topic `(AgoraEvent::FeeUpdated,)`.
#[contracttype]
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct FeeUpdatedEvent {
    /// The new platform fee in basis points (e.g., 500 = 5%).
    pub new_fee_percent: u32,
}

/// Emitted when the contract is initialized for the first time.
///
/// Published with topic `(AgoraEvent::ContractInitialized,)`.
#[contracttype]
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct InitializationEvent {
    /// The initial administrator address.
    pub admin_address: Address,
    /// The platform wallet address for fee collection.
    pub platform_wallet: Address,
    /// The initial platform fee in basis points.
    pub platform_fee_percent: u32,
    /// The ledger timestamp when initialization occurred.
    pub timestamp: u64,
}

/// Emitted when the contract WASM is upgraded to a new version.
///
/// Published with topic `(AgoraEvent::ContractUpgraded,)`.
#[contracttype]
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct RegistryUpgradedEvent {
    /// The admin address that authorized the upgrade.
    pub admin_address: Address,
    /// The ledger timestamp when the upgrade occurred.
    pub timestamp: u64,
}

/// Emitted when an event's IPFS metadata CID is updated.
///
/// Published with topic `(AgoraEvent::MetadataUpdated,)`.
#[contracttype]
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct MetadataUpdatedEvent {
    /// The unique identifier of the event whose metadata changed.
    pub event_id: String,
    /// The new IPFS CID pointing to updated metadata.
    pub new_metadata_cid: String,
    /// The address of the organizer who performed the update.
    pub updated_by: Address,
    /// The ledger timestamp when the metadata was updated.
    pub timestamp: u64,
}

/// Emitted when tickets are sold and the event's supply counter increases.
///
/// Published with topic `(AgoraEvent::InventoryIncremented,)`.
#[contracttype]
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct InventoryIncrementedEvent {
    /// The unique identifier of the event.
    pub event_id: String,
    /// The new total supply (tickets sold) after the increment.
    pub new_supply: i128,
    /// The ledger timestamp when the increment occurred.
    pub timestamp: u64,
}

/// Emitted when a ticket is refunded and the event's supply counter decreases.
///
/// Published with topic `(AgoraEvent::InventoryDecremented,)`.
#[contracttype]
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct InventoryDecrementedEvent {
    /// The unique identifier of the event.
    pub event_id: String,
    /// The new total supply (tickets sold) after the decrement.
    pub new_supply: i128,
    /// The ledger timestamp when the decrement occurred.
    pub timestamp: u64,
}

/// Emitted when an organizer is added to the platform blacklist.
///
/// Published with topic `(AgoraEvent::OrganizerBlacklisted,)`.
#[contracttype]
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct OrganizerBlacklistedEvent {
    /// The wallet address of the blacklisted organizer.
    pub organizer_address: Address,
    /// The admin address that performed the blacklisting.
    pub admin_address: Address,
    /// The reason provided for blacklisting.
    pub reason: String,
    /// The ledger timestamp when the blacklisting occurred.
    pub timestamp: u64,
}

/// Emitted when an organizer is removed from the platform blacklist.
///
/// Published with topic `(AgoraEvent::OrganizerRemovedFromBlacklist,)`.
#[contracttype]
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct OrganizerRemovedFromBlacklistEvent {
    /// The wallet address of the organizer removed from the blacklist.
    pub organizer_address: Address,
    /// The admin address that performed the removal.
    pub admin_address: Address,
    /// The reason provided for removal.
    pub reason: String,
    /// The ledger timestamp when the removal occurred.
    pub timestamp: u64,
}

/// Emitted when all active events for a blacklisted organizer are suspended.
///
/// This is a ripple effect triggered automatically during blacklisting.
/// Published with topic `(AgoraEvent::EventsSuspended,)`.
#[contracttype]
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct EventsSuspendedEvent {
    /// The wallet address of the organizer whose events were suspended.
    pub organizer_address: Address,
    /// The number of events that were actively suspended.
    pub suspended_event_count: u32,
    /// The admin address that triggered the suspension (via blacklisting).
    pub admin_address: Address,
    /// The ledger timestamp when the suspension occurred.
    pub timestamp: u64,
}

/// Emitted when the global promotional discount is created or updated.
///
/// Published with topic `(AgoraEvent::GlobalPromoUpdated,)`.
#[contracttype]
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct GlobalPromoUpdatedEvent {
    /// The new promotional discount rate in basis points (e.g., 1500 = 15% off).
    pub global_promo_bps: u32,
    /// The Unix timestamp after which the promo is no longer applied.
    pub promo_expiry: u64,
    /// The admin address that set or updated the promo.
    pub admin_address: Address,
    /// The ledger timestamp when the promo was updated.
    pub timestamp: u64,
}

/// Emitted when an event is marked as postponed with a temporary refund grace period.
///
/// During the grace period, all guests may request refunds regardless of their
/// ticket tier's standard refundability rules.
/// Published with topic `(AgoraEvent::EventPostponed,)`.
#[contracttype]
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct EventPostponedEvent {
    /// The unique identifier of the postponed event.
    pub event_id: String,
    /// The wallet address of the event organizer.
    pub organizer_address: Address,
    /// The Unix timestamp when the refund grace period ends.
    pub grace_period_end: u64,
    /// The ledger timestamp when the postponement occurred.
    pub timestamp: u64,
}

/// Emitted when a governance proposal is created by an admin.
///
/// Published with topic `(AgoraEvent::ProposalCreated,)` (used internally).
#[contracttype]
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct ProposalCreatedEvent {
    /// The unique identifier of the newly created proposal.
    pub proposal_id: u64,
    /// The admin address that created the proposal.
    pub proposer: Address,
    /// The ledger timestamp when the proposal was created.
    pub timestamp: u64,
}

/// Emitted when an admin approves a governance proposal.
///
/// Published with topic `(AgoraEvent::ProposalApproved,)` (used internally).
#[contracttype]
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct ProposalApprovedEvent {
    /// The unique identifier of the approved proposal.
    pub proposal_id: u64,
    /// The admin address that approved the proposal.
    pub approver: Address,
    /// The ledger timestamp when the approval occurred.
    pub timestamp: u64,
}

/// Emitted when a governance proposal is executed after meeting the approval threshold.
///
/// Published with topic `(AgoraEvent::ProposalExecuted,)` (used internally).
#[contracttype]
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct ProposalExecutedEvent {
    /// The unique identifier of the executed proposal.
    pub proposal_id: u64,
    /// The admin address that triggered execution.
    pub executor: Address,
    /// The ledger timestamp when the proposal was executed.
    pub timestamp: u64,
}

/// Emitted when a governance proposal is cancelled by the proposer.
///
/// Published with topic `(AgoraEvent::ProposalCancelled,)`.
#[contracttype]
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct ProposalCancelledEvent {
    /// The unique identifier of the cancelled proposal.
    pub proposal_id: u64,
    /// The admin address that cancelled the proposal.
    pub cancelled_by: Address,
    /// The ledger timestamp when the cancellation occurred.
    pub timestamp: u64,
}

/// Emitted when a new admin is added to the multi-sig configuration.
#[contracttype]
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct AdminAddedEvent {
    /// The wallet address of the newly added admin.
    pub admin: Address,
    /// The admin address that authorized the addition.
    pub added_by: Address,
    /// The ledger timestamp when the admin was added.
    pub timestamp: u64,
}

/// Emitted when an admin is removed from the multi-sig configuration.
#[contracttype]
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct AdminRemovedEvent {
    /// The wallet address of the removed admin.
    pub admin: Address,
    /// The admin address that authorized the removal.
    pub removed_by: Address,
    /// The ledger timestamp when the admin was removed.
    pub timestamp: u64,
}

/// Emitted when the multi-sig approval threshold is updated.
#[contracttype]
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct ThresholdUpdatedEvent {
    /// The previous approval threshold value.
    pub old_threshold: u32,
    /// The new approval threshold value.
    pub new_threshold: u32,
    /// The ledger timestamp when the threshold was updated.
    pub timestamp: u64,
}

/// Emitted when a scanner wallet is authorized for ticket validation at an event.
///
/// Scanners are wallets authorized by the organizer to validate/check-in tickets
/// at the event venue. Published with topic `(AgoraEvent::ScannerAuthorized,)`.
#[contracttype]
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct ScannerAuthorizedEvent {
    /// The unique identifier of the event the scanner is authorized for.
    pub event_id: String,
    /// The wallet address of the authorized scanner.
    pub scanner: Address,
    /// The organizer address that authorized the scanner.
    pub authorized_by: Address,
    /// The ledger timestamp when the scanner was authorized.
    pub timestamp: u64,
}

/// Emitted when an event's minimum sales target (goal) is reached.
///
/// This signals that the event has sufficient ticket sales to proceed.
/// Published with topic `(AgoraEvent::GoalMet,)`.
#[contracttype]
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct GoalMetEvent {
    /// The unique identifier of the event that met its goal.
    pub event_id: String,
    /// The minimum number of tickets required for the event to proceed.
    pub min_sales_target: i128,
    /// The current ticket supply at the time the goal was met.
    pub current_supply: i128,
    /// The ledger timestamp when the goal was reached.
    pub timestamp: u64,
}

// ── Loyalty & Staking event structs ───────────────────────────────────────────

/// Emitted when an organizer stakes collateral tokens toward Verified status.
///
/// Organizers stake tokens to gain platform trust and unlock Verified badges.
/// The `is_verified` field indicates whether the staked amount meets the minimum
/// threshold for Verified status. Published with topic `(AgoraEvent::CollateralStaked,)`.
#[contracttype]
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct CollateralStakedEvent {
    /// The wallet address of the organizer who staked.
    pub organizer: Address,
    /// The token contract address used for staking.
    pub token: Address,
    /// The amount of tokens staked (in stroops).
    pub amount: i128,
    /// Whether the organizer achieved Verified status with this stake.
    pub is_verified: bool,
    /// The ledger timestamp when the stake was created.
    pub timestamp: u64,
}

/// Emitted when an organizer withdraws their staked collateral tokens.
///
/// After unstaking, the organizer loses Verified status and is removed from
/// the stakers list. Published with topic `(AgoraEvent::CollateralUnstaked,)`.
#[contracttype]
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct CollateralUnstakedEvent {
    /// The wallet address of the organizer who unstaked.
    pub organizer: Address,
    /// The token contract address of the unstaked tokens.
    pub token: Address,
    /// The amount of tokens returned to the organizer (in stroops).
    pub amount: i128,
    /// The ledger timestamp when the unstake occurred.
    pub timestamp: u64,
}

/// Emitted when staking rewards are distributed proportionally to all active stakers.
///
/// Rewards are allocated based on each staker's share of the total staked amount.
/// Published with topic `(AgoraEvent::StakerRewardsDistributed,)`.
#[contracttype]
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct StakerRewardsDistributedEvent {
    /// The total reward amount distributed across all stakers (in stroops).
    pub total_reward: i128,
    /// The number of active stakers at the time of distribution.
    pub staker_count: u32,
    /// The ledger timestamp when the distribution occurred.
    pub timestamp: u64,
}

/// Emitted when an organizer claims their accumulated staking rewards.
///
/// Published with topic `(AgoraEvent::StakerRewardsClaimed,)`.
#[contracttype]
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct StakerRewardsClaimedEvent {
    /// The wallet address of the organizer claiming rewards.
    pub organizer: Address,
    /// The amount of reward tokens claimed (in stroops).
    pub amount: i128,
    /// The ledger timestamp when the claim occurred.
    pub timestamp: u64,
}

/// Emitted when a guest's loyalty score is updated after a ticket purchase.
///
/// Loyalty scores accumulate over time and unlock platform-fee discounts at
/// defined tiers. Published with topic `(AgoraEvent::LoyaltyScoreUpdated,)`.
#[contracttype]
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct LoyaltyScoreUpdatedEvent {
    /// The wallet address of the guest whose score was updated.
    pub guest: Address,
    /// The guest's new cumulative loyalty score.
    pub new_score: u64,
    /// The number of tickets purchased in this transaction.
    pub tickets_purchased: u32,
    /// The ledger timestamp when the score was updated.
    pub timestamp: u64,
}

/// Emitted when a custom fee override is set for a specific event.
///
/// Custom fees allow admins to grant special rates (e.g., for high-volume partners
/// or charitable events). Published with topic `(AgoraEvent::CustomFeeSet,)`.
#[contracttype]
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct CustomFeeSetEvent {
    /// The unique identifier of the event receiving the custom fee.
    pub event_id: String,
    /// The custom fee rate in basis points, or `None` to clear the override.
    pub custom_fee_bps: Option<u32>,
    /// The admin address that set the custom fee.
    pub admin_address: Address,
    /// The ledger timestamp when the custom fee was set.
    pub timestamp: u64,
}

#[contracttype]
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct AdminUpdatedEvent {
    pub old_admin: Address,
    pub new_admin: Address,
    pub timestamp: u64,
}

/// Emitted when a post-event feedback CID is set by the organizer after end_time.
///
/// Published with topic `(AgoraEvent::FeedbackCidSet,)`.
#[contracttype]
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct FeedbackCidSetEvent {
    /// The unique identifier of the event.
    pub event_id: String,
    /// The IPFS CID pointing to the post-event feedback content.
    pub feedback_cid: String,
    /// The organizer address that set the feedback CID.
    pub updated_by: Address,
    /// The ledger timestamp when the feedback CID was set.
    pub timestamp: u64,
}

/// Emitted when an event's token whitelist is updated (token added or removed).
///
/// Published with topic `(AgoraEvent::TokenWhitelistUpdated,)`.
#[contracttype]
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct TokenWhitelistUpdatedEvent {
    /// The unique identifier of the event.
    pub event_id: String,
    /// The token address that was added or removed.
    pub token: Address,
    /// Whether the token was added (true) or removed (false).
    pub added: bool,
    /// The organizer address that performed the update.
    pub organizer_address: Address,
    /// The ledger timestamp when the whitelist was updated.
    pub timestamp: u64,
}

/// Emitted when a user successfully joins the waitlist for an event.
///
/// Published with topic `(AgoraEvent::WaitlistJoined,)`.
#[contracttype]
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct WaitlistJoinedEvent {
    /// The unique identifier of the event.
    pub event_id: String,
    /// The address of the user who joined the waitlist.
    pub user: Address,
    /// The ledger timestamp when the user joined.
    pub timestamp: u64,
}
