use soroban_sdk::contracterror;

#[contracterror]
#[derive(Copy, Clone, Debug, Eq, PartialEq, PartialOrd, Ord)]
#[repr(u32)]
pub enum TicketPaymentError {
    AlreadyInitialized = 1,
    InvalidAddress = 2,
    NotInitialized = 3,
    EventNotFound = 4,
    EventInactive = 5,
    TokenNotWhitelisted = 6,
    MaxSupplyExceeded = 7,
    PaymentNotFound = 8,
    InvalidPaymentStatus = 9,
    TicketNotRefundable = 10,
    TierNotFound = 11,
    InsufficientAllowance = 12,
    TransferVerificationFailed = 13,
    ArithmeticError = 14,
    SelfReferralNotAllowed = 15,
    PriceMismatch = 16,
    InvalidPrice = 17,
    InvalidDiscountCode = 18,
    DiscountCodeAlreadyUsed = 19,
    Unauthorized = 20,
    EventNotCompleted = 21,
    NoFundsAvailable = 22,
    RefundDeadlinePassed = 23,
    WithdrawalCapExceeded = 24,
    InsufficientFees = 25,
    ResalePriceExceedsCap = 26,
    ContractPaused = 27,
    EventCancelled = 35,
    EventDisputed = 36,
    UnauthorizedScanner = 37,
    TicketAlreadyUsed = 38,
}

impl core::fmt::Display for TicketPaymentError {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        match self {
            TicketPaymentError::AlreadyInitialized => {
                write!(f, "Contract already initialized")
            }
            TicketPaymentError::InvalidAddress => write!(f, "Invalid Stellar address"),
            TicketPaymentError::NotInitialized => write!(f, "Contract not initialized"),
            TicketPaymentError::EventNotFound => write!(f, "Event not found in registry"),
            TicketPaymentError::EventInactive => write!(f, "Event is inactive"),
            TicketPaymentError::TokenNotWhitelisted => write!(f, "Token not whitelisted"),
            TicketPaymentError::MaxSupplyExceeded => write!(f, "Ticket supply exceeded"),
            TicketPaymentError::PaymentNotFound => write!(f, "Payment not found"),
            TicketPaymentError::InvalidPaymentStatus => {
                write!(f, "Invalid payment status for refund")
            }
            TicketPaymentError::TicketNotRefundable => write!(f, "Ticket is not refundable"),
            TicketPaymentError::TierNotFound => write!(f, "Ticket tier not found"),
            TicketPaymentError::InsufficientAllowance => {
                write!(f, "Insufficient token allowance")
            }
            TicketPaymentError::TransferVerificationFailed => {
                write!(f, "Transfer verification failed")
            }
            TicketPaymentError::ArithmeticError => {
                write!(f, "Arithmetic error during calculation")
            }
            TicketPaymentError::SelfReferralNotAllowed => {
                write!(f, "Self-referral is not allowed")
            }
            TicketPaymentError::PriceMismatch => {
                write!(f, "Price mismatch")
            }
            TicketPaymentError::InvalidPrice => {
                write!(
                    f,
                    "Paid amount does not match the active price for this tier"
                )
            }
            TicketPaymentError::InvalidDiscountCode => {
                write!(f, "Discount code is invalid or not registered")
            }
            TicketPaymentError::DiscountCodeAlreadyUsed => {
                write!(f, "Discount code has already been used")
            }
            TicketPaymentError::Unauthorized => write!(f, "Unauthorized caller"),
            TicketPaymentError::EventNotCompleted => write!(f, "Event is not completed"),
            TicketPaymentError::NoFundsAvailable => write!(f, "No funds available to claim"),
            TicketPaymentError::RefundDeadlinePassed => write!(f, "Refund deadline has passed"),
            TicketPaymentError::WithdrawalCapExceeded => write!(f, "Daily withdrawal cap exceeded"),
            TicketPaymentError::InsufficientFees => {
                write!(f, "Insufficient platform fees accumulated")
            }
            TicketPaymentError::ResalePriceExceedsCap => {
                write!(f, "Resale price exceeds the event's resale cap")
            }
            TicketPaymentError::ContractPaused => {
                write!(f, "Contract is paused")
            }
            TicketPaymentError::EventCancelled => {
                write!(f, "The event has been cancelled")
            }
            TicketPaymentError::EventDisputed => {
                write!(f, "The event is currently under dispute")
            }
            TicketPaymentError::UnauthorizedScanner => {
                write!(f, "Caller is not an authorized scanner for this event")
            }
            TicketPaymentError::TicketAlreadyUsed => {
                write!(f, "Ticket has already been checked in/used")
            }
        }
    }
}
