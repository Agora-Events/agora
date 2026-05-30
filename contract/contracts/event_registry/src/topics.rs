use soroban_sdk::contracttype;

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
    /// A scanner wallet's authorization has been revoked for an event.
    ScannerRevoked,
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
