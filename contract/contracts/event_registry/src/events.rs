use soroban_sdk::{contractevent, Address, String};

#[contractevent]
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct EventRegistered {
    pub event_id: String,
    pub organizer_address: Address,
    pub payment_address: Address,
    pub timestamp: u64,
}

#[contractevent]
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct EventStatusUpdated {
    pub event_id: String,
    pub is_active: bool,
    pub updated_by: Address,
    pub timestamp: u64,
}

#[contractevent]
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct FeeUpdated {
    pub new_fee_percent: u32,
}
