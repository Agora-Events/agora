use soroban_sdk::contracterror;

#[contracterror]
#[derive(Copy, Clone, Debug, Eq, PartialEq, PartialOrd, Ord)]
#[repr(u32)]
pub enum EventRegistryError {
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
    UnauthorizedCaller = 12,
    TierLimitExceedsMaxSupply = 13,
    TierNotFound = 14,
    TierSupplyExceeded = 15,
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
    // ── Loyalty & Staking errors ───────────────────────────────────────
    /// Organizer already has an active stake
    AlreadyStaked = 26,
    /// Organizer does not have an active stake
    NotStaked = 27,
    /// Stake amount is below the minimum required for Verified status
    InsufficientStakeAmount = 28,
    /// Stake amount must be greater than zero
    InvalidStakeAmount = 29,
    /// Staking has not been configured by the admin
    StakingNotConfigured = 30,
    /// No rewards available to claim
    NoRewardsAvailable = 31,
    /// Reward distribution total must be positive
    InvalidRewardAmount = 32,
    /// Milestone release percentages sum exceeds 100%
    InvalidMilestonePlan = 41,
    /// Restocking fee exceeds the ticket price
    RestockingFeeExceedsTicketPrice = 42,
    /// Tags list is invalid (too many tags or a tag string is too long)
    InvalidTags = 43,
    // ── Governance / Multi-Sig errors ──────────────────────────────────
    /// Admin already exists in the multi-sig configuration
    AdminAlreadyExists = 33,
    /// Admin not found in the multi-sig configuration
    AdminNotFound = 34,
    /// Cannot remove the last admin
    CannotRemoveLastAdmin = 35,
    /// Invalid threshold value
    InvalidThreshold = 36,
    /// Proposal not found
    ProposalNotFound = 37,
    /// Proposal has already been executed
    ProposalAlreadyExecuted = 38,
    /// Proposal has expired
    ProposalExpired = 39,
    /// Insufficient approvals to execute proposal
    InsufficientApprovals = 40,
    /// Target deadline must be in the future
    InvalidTargetDeadline = 44,
    /// Admin has already approved this proposal
    AlreadyApproved = 45,
}

impl core::fmt::Display for EventRegistryError {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        match self {
            EventRegistryError::EventAlreadyExists => write!(f, "Event already exists"),
            EventRegistryError::EventNotFound => write!(f, "Event not found"),
            EventRegistryError::Unauthorized => write!(f, "Caller not authorized for action"),
            EventRegistryError::InvalidAddress => write!(f, "Invalid Stellar address"),
            EventRegistryError::InvalidFeePercent => {
                write!(f, "Fee percent must be between 0 and 10000")
            }
            EventRegistryError::EventInactive => {
                write!(f, "Trying to interact with inactive event")
            }
            EventRegistryError::NotInitialized => write!(f, "Contract not initialized"),
            EventRegistryError::AlreadyInitialized => write!(f, "Contract already initialized"),
            EventRegistryError::InvalidMetadataCid => write!(f, "Invalid IPFS Metadata CID format"),
            EventRegistryError::MaxSupplyExceeded => {
                write!(f, "Event has reached its maximum ticket supply")
            }
            EventRegistryError::SupplyOverflow => {
                write!(f, "Supply counter overflow")
            }
            EventRegistryError::UnauthorizedCaller => {
                write!(f, "Caller is not the authorized TicketPayment contract")
            }
            EventRegistryError::TierLimitExceedsMaxSupply => {
                write!(f, "Sum of tier limits exceeds event max supply")
            }
            EventRegistryError::TierNotFound => {
                write!(
                    f,
                    "The specified ticket tier ID does not exist for this event"
                )
            }
            EventRegistryError::TierSupplyExceeded => {
                write!(
                    f,
                    "The requested ticket tier has sold out and cannot accept more registrations"
                )
            }
            EventRegistryError::SupplyUnderflow => {
                write!(f, "Supply counter underflow")
            }
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
            EventRegistryError::AlreadyStaked => {
                write!(f, "Organizer already has an active stake")
            }
            EventRegistryError::NotStaked => {
                write!(f, "Organizer does not have an active stake")
            }
            EventRegistryError::InsufficientStakeAmount => {
                write!(
                    f,
                    "Stake amount is below the minimum required for Verified status"
                )
            }
            EventRegistryError::InvalidStakeAmount => {
                write!(f, "Stake amount must be greater than zero")
            }
            EventRegistryError::StakingNotConfigured => {
                write!(f, "Staking has not been configured by the admin")
            }
            EventRegistryError::NoRewardsAvailable => {
                write!(f, "No rewards available to claim")
            }
            EventRegistryError::InvalidRewardAmount => {
                write!(f, "Reward distribution total must be positive")
            }
            EventRegistryError::InvalidMilestonePlan => {
                write!(f, "Milestone release percentages must not exceed 100%")
            }
            EventRegistryError::RestockingFeeExceedsTicketPrice => {
                write!(
                    f,
                    "Restocking fee must not exceed the original ticket price"
                )
            }
            EventRegistryError::InvalidTags => {
                write!(
                    f,
                    "Tags are invalid: max 10 tags, each at most 32 characters"
                )
            }
            EventRegistryError::AdminAlreadyExists => {
                write!(f, "Admin already exists in the multi-sig configuration")
            }
            EventRegistryError::AdminNotFound => {
                write!(f, "Admin not found in the multi-sig configuration")
            }
            EventRegistryError::CannotRemoveLastAdmin => {
                write!(f, "Cannot remove the last admin")
            }
            EventRegistryError::InvalidThreshold => {
                write!(f, "Invalid threshold value")
            }
            EventRegistryError::ProposalNotFound => {
                write!(f, "Proposal not found")
            }
            EventRegistryError::ProposalAlreadyExecuted => {
                write!(f, "Proposal has already been executed")
            }
            EventRegistryError::ProposalExpired => {
                write!(f, "Proposal has expired")
            }
            EventRegistryError::InsufficientApprovals => {
                write!(f, "Proposal does not have enough approvals to be executed")
            }
            EventRegistryError::InvalidTargetDeadline => {
                write!(f, "Target deadline must be in the future")
            }
            EventRegistryError::AlreadyApproved => {
                write!(f, "Admin has already approved this proposal")
            }
        }
    }
}
