/// Error codes returned by the Ticket Payment contract.
///
/// Each variant maps to a unique `u32` discriminant used in Soroban contract errors.
#[derive(Copy, Clone, Debug, Eq, PartialEq, PartialOrd, Ord)]
#[repr(u32)]
pub enum TicketPaymentError {
    /// Contract has already been initialized.
    AlreadyInitialized = 1,
    /// The provided address is invalid.
    InvalidAddress = 2,
    /// Contract has not been initialized.
    NotInitialized = 3,
    /// The requested event was not found in the registry.
    EventNotFound = 4,
    /// The event is currently inactive.
    EventInactive = 5,
    /// The payment token is not on the whitelist.
    TokenNotWhitelisted = 6,
    /// The event has sold out (max supply reached).
    MaxSupplyExceeded = 7,
    /// The payment record was not found.
    PaymentNotFound = 8,
    /// The payment is in an invalid status for the requested operation.
    InvalidPaymentStatus = 9,
    /// The ticket tier does not support refunds.
    TicketNotRefundable = 10,
    /// The requested ticket tier was not found.
    TierNotFound = 11,
    /// The buyer has not approved sufficient token allowance.
    InsufficientAllowance = 12,
    /// On-chain balance after transfer did not match expected amount.
    TransferVerificationFailed = 13,
    /// An arithmetic overflow or underflow occurred.
    ArithmeticError = 14,
    /// Buyer attempted to refer themselves.
    SelfReferralNotAllowed = 15,
    /// The supplied amount did not match the tier price.
    PriceMismatch = 16,
    /// The supplied price is invalid (e.g., negative or zero).
    InvalidPrice = 17,
    /// The discount code preimage is invalid.
    InvalidDiscountCode = 18,
    /// The discount code has already been used.
    DiscountCodeUsed = 19,
    /// Caller is not authorized to perform this action.
    Unauthorized = 20,
    /// The event has not yet completed.
    EventNotCompleted = 21,
    /// There are no funds available to withdraw.
    NoFundsAvailable = 22,
    /// The refund deadline has passed.
    RefundDeadlinePassed = 23,
    /// The requested withdrawal exceeds the daily cap.
    WithdrawalCapExceeded = 24,
    /// Platform fee balance is insufficient.
    InsufficientFees = 25,
    /// Resale price exceeds the event's configured cap.
    ResalePriceExceedsCap = 26,
    /// The contract is currently paused.
    ContractPaused = 27,
    /// The event has been cancelled.
    EventCancelled = 35,
    /// The event is under dispute.
    EventDisputed = 36,
    /// The scanner is not authorized for this event.
    UnauthorizedScanner = 37,
    /// The ticket has already been used (checked in).
    TicketAlreadyUsed = 38,
    /// The event's minimum sales goal has not been met.
    GoalNotMet = 39,
    /// No oracle contract has been configured.
    OracleNotConfigured = 40,
    /// The oracle price feed is unavailable.
    OraclePriceUnavailable = 41,
    /// The supplied amount is outside the slippage tolerance.
    PriceOutsideSlippage = 42,
    /// The slippage basis points value is invalid.
    InvalidSlippageBps = 43,
    /// No active auction exists for the specified tier.
    AuctionNotActive = 44,
    /// Bid is too low (below minimum increment or start price).
    BidTooLow = 45,
    /// The auction has already ended.
    AuctionEnded = 46,
    /// The auction has not ended yet.
    AuctionNotEnded = 47,
    /// The tier is not an auction tier.
    NotAuctionTier = 48,
    /// Caller is not a recognized governor.
    NotGovernor = 49,
    /// The governance proposal does not exist.
    InvalidProposal = 50,
    /// The governance proposal is not in an active state.
    ProposalNotActive = 51,
    /// The caller has already voted on this proposal.
    AlreadyVoted = 52,
    /// The governance voting period has not elapsed yet.
    VotingPeriodNotMet = 53,
    /// The proposal does not have enough votes to execute.
    InsufficientVotes = 54,
    /// The governance proposal has expired.
    ProposalExpired = 55,
    /// The oracle price data is stale (older than the maximum allowed age).
    OraclePriceStale = 56,
    /// Cannot remove the last remaining governor.
    CannotRemoveLastGovernor = 57,
    /// The fee percentage is out of range.
    InvalidFeePercent = 58,
    /// The event has already ended and no longer accepts tickets.
    EventEnded = 59,
    TicketAlreadyListed = 60,
    ResaleListingNotFound = 61,
    ResaleListingNotActive = 62,
    NonTransferable = 63,
    InvalidRoyaltyBps = 64,
    /// Returned when a bulk operation receives an empty input vector.
    EmptyBatch = 65,
    /// Returned when a bulk operation input exceeds `MAX_BATCH_SIZE`.
    BatchTooLarge = 66,
}

impl From<TicketPaymentError> for soroban_sdk::Error {
    fn from(err: TicketPaymentError) -> soroban_sdk::Error {
        soroban_sdk::Error::from_contract_error(err as u32)
    }
}

impl From<&TicketPaymentError> for soroban_sdk::Error {
    fn from(err: &TicketPaymentError) -> soroban_sdk::Error {
        soroban_sdk::Error::from_contract_error(*err as u32)
    }
}

impl From<soroban_sdk::Error> for TicketPaymentError {
    fn from(err: soroban_sdk::Error) -> TicketPaymentError {
        match err.get_code() {
            1 => TicketPaymentError::AlreadyInitialized,
            2 => TicketPaymentError::InvalidAddress,
            3 => TicketPaymentError::NotInitialized,
            4 => TicketPaymentError::EventNotFound,
            5 => TicketPaymentError::EventInactive,
            6 => TicketPaymentError::TokenNotWhitelisted,
            7 => TicketPaymentError::MaxSupplyExceeded,
            8 => TicketPaymentError::PaymentNotFound,
            9 => TicketPaymentError::InvalidPaymentStatus,
            10 => TicketPaymentError::TicketNotRefundable,
            11 => TicketPaymentError::TierNotFound,
            12 => TicketPaymentError::InsufficientAllowance,
            13 => TicketPaymentError::TransferVerificationFailed,
            14 => TicketPaymentError::ArithmeticError,
            15 => TicketPaymentError::SelfReferralNotAllowed,
            16 => TicketPaymentError::PriceMismatch,
            17 => TicketPaymentError::InvalidPrice,
            18 => TicketPaymentError::InvalidDiscountCode,
            19 => TicketPaymentError::DiscountCodeUsed,
            20 => TicketPaymentError::Unauthorized,
            21 => TicketPaymentError::EventNotCompleted,
            22 => TicketPaymentError::NoFundsAvailable,
            23 => TicketPaymentError::RefundDeadlinePassed,
            24 => TicketPaymentError::WithdrawalCapExceeded,
            25 => TicketPaymentError::InsufficientFees,
            26 => TicketPaymentError::ResalePriceExceedsCap,
            27 => TicketPaymentError::ContractPaused,
            28 => TicketPaymentError::InvalidSecret,
            29 => TicketPaymentError::CommitExpired,
            30 => TicketPaymentError::DiscountExpired,
            31 => TicketPaymentError::DiscountMaxUsesReached,
            32 => TicketPaymentError::DisputeNotResolved,
            33 => TicketPaymentError::EscrowNotInitialized,
            34 => TicketPaymentError::InvalidAmount,
            35 => TicketPaymentError::EventCancelled,
            36 => TicketPaymentError::EventDisputed,
            37 => TicketPaymentError::UnauthorizedScanner,
            38 => TicketPaymentError::TicketAlreadyUsed,
            39 => TicketPaymentError::GoalNotMet,
            40 => TicketPaymentError::OracleNotConfigured,
            41 => TicketPaymentError::OraclePriceUnavailable,
            42 => TicketPaymentError::PriceOutsideSlippage,
            43 => TicketPaymentError::InvalidSlippageBps,
            44 => TicketPaymentError::AuctionNotActive,
            45 => TicketPaymentError::BidTooLow,
            46 => TicketPaymentError::AuctionEnded,
            47 => TicketPaymentError::AuctionNotEnded,
            48 => TicketPaymentError::NotAuctionTier,
            49 => TicketPaymentError::NotGovernor,
            50 => TicketPaymentError::InvalidProposal,
            51 => TicketPaymentError::ProposalNotActive,
            52 => TicketPaymentError::AlreadyVoted,
            53 => TicketPaymentError::VotingPeriodNotMet,
            54 => TicketPaymentError::InsufficientVotes,
            55 => TicketPaymentError::ProposalExpired,
            56 => TicketPaymentError::OraclePriceStale,
            57 => TicketPaymentError::CannotRemoveLastGovernor,
            58 => TicketPaymentError::InvalidFeePercent,
            59 => TicketPaymentError::EventEnded,
            60 => TicketPaymentError::TicketAlreadyListed,
            61 => TicketPaymentError::ResaleListingNotFound,
            62 => TicketPaymentError::ResaleListingNotActive,
            63 => TicketPaymentError::NonTransferable,
            64 => TicketPaymentError::InvalidRoyaltyBps,
            65 => TicketPaymentError::EmptyBatch,
            66 => TicketPaymentError::BatchTooLarge,
            _ => TicketPaymentError::ArithmeticError,
        }
    }
}
