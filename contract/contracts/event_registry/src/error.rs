use soroban_sdk::contracterror;

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
    TierLimitExceedsMaxSupply = 13,
    TierNotFound = 14,
    TierSoldOut = 15,
    SupplyUnderflow = 16,
    InvalidQuantity = 17,
    OrganizerBlacklisted = 18,
    OrganizerNotBlacklisted = 19,
    InvalidResaleCapBps = 20,
    InvalidPromoBps = 21,
    EventCancelled = 22,
    EventAlreadyCancelled = 23,
    InvalidGracePeriodEnd = 24,
    EventIsActive = 25,
    // Staking / loyalty
    AlreadyStaked = 26,
    NotStaked = 27,
    NoRewardsAvailable = 28,
    InvalidMilestonePlan = 29,
    ProposalExpired = 30,
    RestockingFeeExceedsPrice = 31,
    // Multisig-specific error codes (must match test expectations)
    AdminAlreadyExists = 33,
    CannotRemoveLastAdmin = 35,
    InvalidThreshold = 36,
    ProposalAlreadyExecuted = 38,
    InvalidTags = 59,
    PerUserLimitExceeded = 60,
    EventNotEnded = 39,
    // Generic variants used internally in lib.rs
    StateError = 43, // bad contract state (e.g. staking not set up)
    ProposalAlreadyApproved = 45,
    MultisigError = 47, // multisig auth failure
    ProposalAlreadyCancelled = 49,
    // Governance / proposals
    InvalidTargetDeadline = 54,
    DeadlineAfterEndTime = 55,
    InsufficientStakeAmount = 56,
    InvalidRewardAmount = 57,
    StakingNotConfigured = 58,
    InvalidDeadline = 61, // deadline validation
    InvalidStakeAmount = 70,
}

impl core::fmt::Display for EventRegistryError {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        match self {
            EventRegistryError::EventAlreadyExists => {
                write!(f, "Event already exists")
            }
            EventRegistryError::EventNotFound => write!(f, "Event not found"),
            EventRegistryError::Unauthorized | EventRegistryError::MultisigError => {
                write!(f, "Caller not authorized for action")
            }
            EventRegistryError::InvalidAddress => {
                write!(f, "Invalid address or input")
            }
            EventRegistryError::InvalidFeePercent => {
                write!(f, "Fee percent must be between 0 and 10000")
            }
            EventRegistryError::EventInactive => {
                write!(f, "Trying to interact with inactive event")
            }
            EventRegistryError::NotInitialized | EventRegistryError::StateError => {
                write!(f, "Contract not initialized or invalid state")
            }
            EventRegistryError::AlreadyInitialized => write!(f, "Contract already initialized"),
            EventRegistryError::InvalidMetadataCid => {
                write!(f, "Invalid IPFS Metadata CID format")
            }
            EventRegistryError::MaxSupplyExceeded => {
                write!(f, "Event has reached its maximum ticket supply")
            }
            EventRegistryError::SupplyOverflow => write!(f, "Supply counter overflow"),
            EventRegistryError::TierLimitExceedsMaxSupply => {
                write!(f, "Sum of tier limits exceeds event max supply")
            }
            EventRegistryError::TierNotFound => {
                write!(
                    f,
                    "The specified ticket tier ID does not exist for this event"
                )
            }
            EventRegistryError::TierSoldOut => write!(
                f,
                "The requested ticket tier has sold out and cannot accept more registrations"
            ),
            EventRegistryError::SupplyUnderflow => write!(f, "Supply counter underflow"),
            EventRegistryError::InvalidQuantity => {
                write!(f, "Quantity must be greater than zero")
            }
            EventRegistryError::OrganizerBlacklisted => {
                write!(f, "Organizer is blacklisted and cannot perform this action")
            }
            EventRegistryError::OrganizerNotBlacklisted => {
                write!(f, "Organizer is not currently blacklisted")
            }
            EventRegistryError::InvalidResaleCapBps => {
                write!(f, "Resale cap must be between 0 and 10000 basis points")
            }
            EventRegistryError::InvalidPromoBps => {
                write!(f, "Promo discount must be between 0 and 10000 basis points")
            }
            EventRegistryError::EventCancelled => {
                write!(f, "The event has been cancelled")
            }
            EventRegistryError::EventAlreadyCancelled => {
                write!(f, "The event is already cancelled")
            }
            EventRegistryError::InvalidGracePeriodEnd => {
                write!(f, "Grace period end timestamp must be in the future")
            }
            EventRegistryError::EventIsActive => {
                write!(f, "Cannot perform action on an active event")
            }
            EventRegistryError::AlreadyStaked => write!(f, "Organizer already has an active stake"),
            EventRegistryError::NotStaked => write!(f, "Organizer does not have an active stake"),
            EventRegistryError::InsufficientStakeAmount => write!(
                f,
                "Stake amount is below the minimum required for Verified status"
            ),
            EventRegistryError::InvalidStakeAmount => {
                write!(f, "Stake amount must be greater than zero")
            }
            EventRegistryError::StakingNotConfigured => {
                write!(f, "Staking has not been configured by the admin")
            }
            EventRegistryError::NoRewardsAvailable => write!(f, "No rewards available to claim"),
            EventRegistryError::InvalidRewardAmount => {
                write!(f, "Reward distribution total must be positive")
            }
            EventRegistryError::InvalidMilestonePlan => {
                write!(f, "Milestone release percentages must not exceed 100%")
            }
            EventRegistryError::RestockingFeeExceedsPrice => {
                write!(
                    f,
                    "Restocking fee must not exceed the original ticket price"
                )
            }
            EventRegistryError::InvalidTags => write!(
                f,
                "Tags are invalid: max 10 tags, each at most 32 characters"
            ),
            EventRegistryError::ProposalExpired => write!(f, "Proposal has expired"),
            EventRegistryError::InvalidTargetDeadline | EventRegistryError::InvalidDeadline => {
                write!(f, "Target deadline must be in the future")
            }
            EventRegistryError::ProposalAlreadyExecuted => {
                write!(f, "Proposal has already been executed")
            }
            EventRegistryError::DeadlineAfterEndTime => write!(
                f,
                "refund_deadline and target_deadline must be before end_time"
            ),
            EventRegistryError::PerUserLimitExceeded => write!(
                f,
                "User has exceeded the maximum number of tickets allowed for this tier"
            ),
            EventRegistryError::EventNotEnded => write!(
                f,
                "Event has not ended yet; feedback CID can only be set after end_time"
            ),
            EventRegistryError::AdminAlreadyExists => {
                write!(f, "Admin already exists in the multisig configuration")
            }
            EventRegistryError::CannotRemoveLastAdmin => {
                write!(
                    f,
                    "Cannot remove the last admin from the multisig configuration"
                )
            }
            EventRegistryError::InvalidThreshold => {
                write!(f, "Threshold must be between 1 and the number of admins")
            }
            EventRegistryError::ProposalAlreadyApproved => {
                write!(f, "Admin has already approved this proposal")
            }
            EventRegistryError::ProposalAlreadyCancelled => {
                write!(f, "Proposal has already been cancelled")
            }
        }
    }
}
