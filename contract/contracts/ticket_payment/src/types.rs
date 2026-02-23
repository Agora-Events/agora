use soroban_sdk::{contracttype, Address, BytesN, String};

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
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct EventBalance {
    pub organizer_amount: i128,
    pub total_withdrawn: i128,
    pub platform_fee: i128,
}

#[contracttype]
pub enum DataKey {
    Payment(String),                     // payment_id -> Payment
    EventPayments(String),               // event_id -> Vec<payment_id>
    BuyerPayments(Address),              // buyer_address -> Vec<payment_id>
    Admin,                               // Contract administrator address
    UsdcToken,                           // USDC token address
    PlatformWallet,                      // Platform wallet address
    EventRegistry,                       // Event Registry contract address
    Initialized,                         // Initialization flag
    TokenWhitelist(Address),             // token_address -> bool
    Balances(String),                    // event_id -> EventBalance (escrow tracking)
    TransferFee(String),                 // event_id -> transfer_fee amount
    BulkRefundIndex(String),             // event_id -> last processed payment index
    PriceSwitched(String, String),       // (event_id, tier_id) -> bool
    TotalVolumeProcessed,                // protocol-wide gross volume from all ticket sales
    TotalFeesCollected(Address),         // cumulative platform fees collected by token
    ActiveEscrowTotal,                   // protocol-wide active escrow across all tokens
    ActiveEscrowByToken(Address),        // active escrow amount per token
    DiscountCodeHash(BytesN<32>),        // sha256_hash -> bool (registered)
    DiscountCodeUsed(BytesN<32>),        // sha256_hash -> bool (spent)
    WithdrawalCap(Address),              // token_address -> max amount per day
    DailyWithdrawalAmount(Address, u64), // (token_address, day_timestamp) -> amount withdrawn
    IsPaused,                            // bool – global circuit breaker flag
    DisputeStatus(String),               // event_id -> bool
}
