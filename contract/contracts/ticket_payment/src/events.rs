use crate::types::PaymentStatus;
use soroban_sdk::{contracttype, Address, BytesN, String};

#[contracttype]
#[derive(Clone, Debug, Eq, PartialEq)]
pub enum AgoraEvent {
    PaymentProcessed,
    PaymentStatusChanged,
    ContractInitialized,
    ContractUpgraded,
    TicketTransferred,
    PriceSwitched,
    BulkRefundProcessed,
    DiscountCodeApplied,
    RevenueClaimed,
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

#[contracttype]
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct InitializationEvent {
    pub usdc_token: Address,
    pub platform_wallet: Address,
    pub event_registry: Address,
}

#[contracttype]
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct ContractUpgraded {
    pub old_wasm_hash: BytesN<32>,
    pub new_wasm_hash: BytesN<32>,
}
#[contracttype]
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct TicketTransferredEvent {
    pub payment_id: String,
    pub from: Address,
    pub to: Address,
    pub transfer_fee: i128,
    pub timestamp: u64,
}

#[contracttype]
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct PriceSwitchedEvent {
    pub event_id: String,
    pub tier_id: String,
    pub new_price: i128,
    pub timestamp: u64,
}

#[contracttype]
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct BulkRefundProcessedEvent {
    pub event_id: String,
    pub refund_count: u32,
    pub total_refunded: i128,
    pub timestamp: u64,
}

#[contracttype]
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct DiscountCodeAppliedEvent {
    pub payment_id: String,
    pub event_id: String,
    pub code_hash: BytesN<32>,
    pub discount_amount: i128,
    pub timestamp: u64,
}

#[contracttype]
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct RevenueClaimedEvent {
    pub event_id: String,
    pub organizer_address: Address,
    pub amount: i128,
    pub timestamp: u64,
}
