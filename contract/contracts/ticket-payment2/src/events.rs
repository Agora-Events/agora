use crate::PaymentStatus;
use soroban_sdk::{contracttype, Address, String};

#[contracttype]
#[derive(Clone, Debug, Eq, PartialEq)]
pub enum AgoraEvent {
    PaymentProcessed,
    PaymentStatusChanged,
}

#[contracttype]
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct PaymentProcessedEvent {
    pub payment_id: String,
    pub event_id: String,
    pub buyer_address: Address,
    pub amount: i128,
    pub platform_fee: i128,
    pub timestamp: u64,
}

#[contracttype]
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct PaymentStatusChangedEvent {
    pub payment_id: String,
    pub old_status: PaymentStatus,
    pub new_status: PaymentStatus,
    pub transaction_hash: String,
    pub timestamp: u64,
}
