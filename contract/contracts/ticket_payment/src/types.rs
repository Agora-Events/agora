use soroban_sdk::{contracttype, Address, String};

#[contracttype]
#[derive(Clone, Debug, Eq, PartialEq)]
pub enum PaymentStatus {
    Pending,
    Confirmed,
    Refunded,
    Failed,
}

#[contracttype]
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct Payment {
    pub payment_id: String,
    pub event_id: String,
    pub buyer_address: Address,
    pub ticket_tier_id: String,
    pub amount: i128, // USDC amount in stroops
    pub platform_fee: i128,
    pub organizer_amount: i128,
    pub status: PaymentStatus,
    pub transaction_hash: String,
    pub created_at: u64,
    pub confirmed_at: Option<u64>,
}

#[contracttype]
pub enum DataKey {
    Payment(String),        // payment_id -> Payment
    EventPayments(String),  // event_id -> Vec<payment_id>
    BuyerPayments(Address), // buyer_address -> Vec<payment_id>
    Admin,                  // Contract administrator address
    UsdcToken,              // USDC token address
    PlatformWallet,         // Platform wallet address
    EventRegistry,          // Event Registry contract address
    Initialized,            // Initialization flag
}
