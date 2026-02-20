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
        }
    }
}
