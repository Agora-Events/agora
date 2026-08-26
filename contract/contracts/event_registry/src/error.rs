use soroban_sdk::contracterror;

#[contracterror]
#[derive(Copy, Clone, Debug, Eq, PartialEq)]
/// Error codes returned by the Event Registry contract.
pub enum EventRegistryError {
    // Core event errors
    /// An event with the given ID already exists.
    EventAlreadyExists = 1,
    /// No event found with the given ID.
    EventNotFound = 2,
    /// Caller is not authorized to perform this action.
    Unauthorized = 3,
    /// The provided address is invalid.
    InvalidAddress = 4,
    /// Platform fee percent is out of range (0–10000 bps).
    InvalidFeePercent = 5,
    /// The event is currently inactive.
    EventInactive = 6,
    /// Contract has not been initialized.
    NotInitialized = 7,
    /// Contract has already been initialized.
    AlreadyInitialized = 8,
    /// The metadata CID is not a valid IPFS CIDv1.
    InvalidMetadataCid = 9,
    /// The event has reached its maximum ticket supply.
    MaxSupplyExceeded = 10,
    /// Supply counter would overflow.
    SupplyOverflow = 11,
    /// Combined tier limits exceed the event's max supply.
    TierLimitExceeds = 13,
    /// The requested tier was not found.
    TierNotFound = 14,
    /// The requested tier is sold out.
    TierSoldOut = 15,
    /// Supply counter would underflow below zero.
    SupplyUnderflow = 16,
    /// Quantity must be greater than zero.
    InvalidQuantity = 17,
    /// The organizer is on the blacklist and cannot register events.
    OrganizerBlacklisted = 18,
    /// The organizer is not currently blacklisted.
    OrgNotBlacklisted = 19,
    /// The resale cap basis points value exceeds 10000.
    InvalidResaleCapBps = 20,
    /// The promotional discount basis points value exceeds 10000.
    InvalidPromoBps = 21,
    /// The event has been cancelled.
    EventCancelled = 22,
    /// The event has already been cancelled.
    EventAlreadyCanceled = 23,
    /// The grace period timestamp is invalid.
    InvalidGracePeriod = 24,
    /// The event is still active and cannot be archived.
    EventIsActive = 25,
    // Staking / loyalty
    /// Organizer has already staked collateral.
    AlreadyStaked = 26,
    /// Organizer has not staked collateral.
    NotStaked = 27,
    /// Staked amount is below the required minimum.
    InsufficientStake = 28,
    /// Stake amount must be positive.
    InvalidStakeAmount = 29,
    /// Staking has not been configured (no token or min amount set).
    StakingNotConfigured = 30,
    /// No staking rewards are available to claim.
    NoRewardsAvailable = 31,
    /// Reward amount must be positive.
    InvalidRewardAmount = 32,
    /// The admin address already exists in the multi-sig configuration.
    AdminAlreadyExists = 33,
    /// Cannot remove the last admin from the multi-sig.
    CannotRemoveLast = 35,
    /// The proposed threshold is invalid.
    InvalidThreshold = 36,
    /// The proposal has already been executed.
    PropAlreadyExecuted = 38,
    /// The event has not ended yet.
    EventNotEnded = 39,
    /// The milestone release percentages sum exceeds 10000 bps (100%).
    InvalidMilestonePlan = 41,
    /// The restocking fee is too high relative to the ticket price.
    RestockingFeeHigh = 42,
    /// One or more event tags are invalid.
    InvalidTags = 43,
    /// The governance proposal has expired.
    ProposalExpired = 44,
    /// The admin has already approved this proposal.
    PropAlreadyApproved = 45,
    /// Internal state error.
    StateError = 46,
    /// Multi-sig operation failed (e.g., no approved proposal found).
    MultisigError = 47,
    /// The proposal has already been cancelled.
    PropAlreadyCanceled = 49,
    /// Refund deadline is past the event end time.
    DeadlinePastEnd = 55,
    /// Buyer has exceeded the per-user ticket limit for this tier.
    PerUserLimitExceeded = 60,
    /// The provided start/end time deadline is invalid.
    InvalidDeadline = 61,
    /// The provided category ID is invalid.
    InvalidCategoryId = 71,
    /// The address is already on the waitlist for this event.
    AlreadyOnWaitlist = 75,
    /// The address is not on the waitlist for this event.
    NotOnWaitlist = 76,
    /// Too many tiers specified (exceeds `MAX_TIERS_PER_EVENT`).
    TooManyTiers = 80,
    // TooManyIds = 81,
    /// Issue #851: payment token is not in the event's accepted_tokens list.
    TokenNotAccepted = 82,
}
