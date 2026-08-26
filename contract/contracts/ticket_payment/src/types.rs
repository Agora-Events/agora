use soroban_sdk::{contracttype, Address, String};

pub const TRANSFER_FEE_BPS: u32 = 100;
pub const MAX_BPS: u32 = 10000;

// Re-export DataKey from the dedicated keys module so all existing imports continue to work.
pub use crate::keys::DataKey;

// Re-export payment-specific types from the dedicated payment_types module.
pub use crate::payment_types::{DiscountData, HighestBid, PurchaseOptions};

// Re-export governance-related types from the dedicated governance module.
pub use crate::governance::{ParameterChange, ParameterProposal, ProposalStatus};

// Re-export escrow-related types from the dedicated escrow module.
pub use crate::escrow::{EscrowMilestone, EscrowState};

#[contracttype]
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct AuctionConfig {
    pub start_price: i128,
    pub end_time: u64,
    pub min_increment: i128,
}

#[contracttype]
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct PriceSchedule {
    pub price: i128,
    pub valid_until: u64,
}

#[contracttype]
#[derive(Clone, Debug, Eq, PartialEq)]
pub enum PaymentStatus {
    Pending,
    Confirmed,
    Refunded,
    Failed,
    CheckedIn,
}

#[contracttype]
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct Payment {
    pub payment_id: String,
    pub event_id: String,
    pub buyer_address: Address,
    pub owner_address: Address, // The recipient who owns the ticket (can be different from buyer)
    pub ticket_tier_id: String,
    pub token_address: Address,
    pub amount: i128, // Payment token amount in stroops
    pub platform_fee: i128,
    pub organizer_amount: i128,
    pub status: PaymentStatus,
    pub transaction_hash: String,
    pub created_at: u64,
    pub confirmed_at: Option<u64>,
    pub refunded_amount: i128,
    pub is_soulbound: bool,
    pub last_checked_in_at: u64,
    pub referral_amount: i128,
    pub referrer: Option<Address>,
}

#[contracttype]
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct EventBalance {
    pub organizer_amount: i128,
    pub total_withdrawn: i128,
    pub platform_fee: i128,
}

#[contracttype]
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct HighestBid {
    pub bidder: Address,
    pub amount: i128,
}

#[contracttype]
pub enum DataKey {
    Payment(String), // payment_id -> Payment
    /// Individual entry for an event payment (Persistent)
    EventPayment(String, String),
    /// Sharded mapping of event_id to payment_ids (Persistent)
    EventPaymentShard(String, u32),
    /// Total number of payments for an event (Persistent)
    EventPaymentCount(String),
    /// Individual entry for a buyer payment (Persistent)
    BuyerPayment(Address, String),
    /// Sharded mapping of buyer_address to payment_ids (Persistent)
    BuyerPaymentShard(Address, u32),
    /// Total number of payments for a buyer (Persistent)
    BuyerPaymentCount(Address),
    Admin,                               // Contract administrator address
    UsdcToken,                           // USDC token address
    PlatformWallet,                      // Platform wallet address
    EventRegistry,                       // Event Registry contract address
    Initialized,                         // Initialization flag
    TokenWhitelist(Address),             // token_address -> bool
    Balances(String),                    // event_id -> EventBalance (escrow tracking)
    TransferFee(String),                 // event_id -> transfer_fee_bps (u32)
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
    PartialRefundIndex(String),          // event_id -> last processed payment index
    PartialRefundPercentage(String),     // event_id -> active refund percentage in bps
    OracleAddress,                       // Address of oracle contract
    SlippageBps,                         // u32 — slippage tolerance in bps (default 200 = 2%)
    HighestBid(String, String),          // (event_id, tier_id) -> HighestBid
    AuctionClosed(String, String),       // (event_id, tier_id) -> bool
    Governor(Address),                   // Address -> bool (is authorized governor)
    TotalGovernors,                      // u32
    Proposal(u64),                       // id -> ParameterProposal
    ProposalCount,                       // u64
    /// Status index for payments: (event_id, status) -> Vec<payment_id>
    EventPaymentStatus(String, PaymentStatus),
    /// Individual entry for status index: (event_id, status, payment_id) -> bool
    EventPaymentStatusEntry(String, PaymentStatus, String),
    /// Resale escrow listing: payment_id -> ResaleListing
    ResaleListing(String),
    /// Royalty bps override for an event's resale: event_id -> u32
    ResaleRoyaltyBps(String),
}
