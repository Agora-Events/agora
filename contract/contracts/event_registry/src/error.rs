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
        }
    }
}
