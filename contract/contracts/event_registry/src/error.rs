use soroban_sdk::contracterror;

// NOTE: `#[contracterror]` enums are capped at 50 variants by the Soroban
// SDK's spec-XDR generation (empirically confirmed — 50 compiles, 51
// panics the macro with `LengthExceedsMax`). This enum previously had 53;
// the six variants removed below (DeadlinePastEnd, InsufficientStake,
// InvalidCategoryId, PropAlreadyApproved, RestockingFeeHigh, StateError)
// were confirmed unused anywhere in this contract before removal.
#[contracterror]
#[derive(Copy, Clone, Debug, Eq, PartialEq)]
pub enum EventRegistryError {
    // Core event errors
    EventAlreadyExists = 1,
    EventNotFound = 2,
    Unauthorized = 3,
    InvalidAddress = 4,
    InvalidFeePercent = 5,
    EventInactive = 6,
    NotInitialized = 7,
    AlreadyInitialized = 8,
    InvalidMetadataCid = 9,
    MaxSupplyExceeded = 10,
    SupplyOverflow = 11,
    TierLimitExceeds = 13,
    TierNotFound = 14,
    TierSoldOut = 15,
    SupplyUnderflow = 16,
    InvalidQuantity = 17,
    OrganizerBlacklisted = 18,
    OrgNotBlacklisted = 19,
    InvalidResaleCapBps = 20,
    InvalidPromoBps = 21,
    EventCancelled = 22,
    EventAlreadyCanceled = 23,
    InvalidGracePeriod = 24,
    EventIsActive = 25,
    // Staking / loyalty
    AlreadyStaked = 26,
    NotStaked = 27,
    InvalidStakeAmount = 29,
    StakingNotConfigured = 30,
    NoRewardsAvailable = 31,
    InvalidRewardAmount = 32,
    AdminAlreadyExists = 33,
    CannotRemoveLast = 35,
    InvalidThreshold = 36,
    PropAlreadyExecuted = 38,
    EventNotEnded = 39,
    InvalidMilestonePlan = 41,
    InvalidTags = 43,
    ProposalExpired = 44,
    MultisigError = 47,
    PropAlreadyCanceled = 49,
    PerUserLimitExceeded = 60,
    InvalidDeadline = 61,
    AlreadyOnWaitlist = 75,
    NotOnWaitlist = 76,
    TooManyTiers = 80,
    // TooManyIds = 81,
    /// Issue #851: payment token is not in the event's accepted_tokens list.
    TokenNotAccepted = 82,
}
