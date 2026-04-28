use soroban_sdk::contracterror;

#[contracterror]
#[derive(Copy, Clone, Debug, Eq, PartialEq)]
pub enum EventRegistryError {
    Unauthorized = 1,
    NotFound = 2,
    InvalidInput = 3,
    LimitExceeded = 4,
    StateError = 5,
    AlreadyExists = 6,
    InvalidDeadline = 7,
    AlreadyCancelled = 8,
    SupplyExceeded = 9,
    MultisigError = 10,
    NotInitialized = 11,
    AlreadyExecuted = 12,
    AlreadyInitialized = 13,
    InvalidTargetDeadline = 14,
    DeadlineAfterEndTime = 15,
    EventInactive = 16,
    MaxSupplyExceeded = 17,
    TierSupplyExceeded = 18,
    TierLimitExceedsMaxSupply = 19,
    OrganizerBlacklisted = 20,
    OrganizerNotBlacklisted = 21,
    InvalidResaleCapBps = 22,
    EventIsActive = 23,
    EventAlreadyCancelled = 24,
    InvalidStakeAmount = 25,
    StakingNotConfigured = 26,
    NotStaked = 27,
    NoRewardsAvailable = 28,
    InvalidMilestonePlan = 29,
    ProposalExpired = 30,
    RestockingFeeExceedsTicketPrice = 31,
    InvalidAddress = 32,
    InvalidFeePercent = 33,
    InvalidMetadataCid = 34,
    InvalidTags = 35,
    InvalidQuantity = 36,
    SupplyOverflow = 37,
    PerUserLimitExceeded = 38,
    EventNotEnded = 39,
    EventCancelled = 40,
    EventNotFound = 41,
    TierNotFound = 42,
    AlreadyStaked = 43,
    EventAlreadyExists = 44,
}

impl core::fmt::Display for EventRegistryError {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        write!(f, "{:?}", self)
    }
}
