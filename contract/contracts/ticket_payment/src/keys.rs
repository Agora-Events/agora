use soroban_sdk::{contracttype, Address, BytesN, String};

use crate::types::PaymentStatus;

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
    ProSubscriptionContract,             // Pro Subscription contract address
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
    DiscountCode(String, String),        // (event_id, code) -> DiscountData
    WithdrawalCap(Address),              // token_address -> max amount per day
    DailyWithdrawalAmount(Address, u64), // (token_address, day_timestamp) -> amount withdrawn
    IsPaused,                            // bool – global circuit breaker flag
    DisputeStatus(String),               // event_id -> bool
    EventCancelledForRefund(String),     // event_id -> bool
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
    /// SHA-256 hash of the ticket secret: payment_id -> BytesN<32>
    ValidationHash(String),
    /// Per-event affiliate commission rate: (event_id, affiliate_addr) -> rate_bps (u32)
    AffiliateRate(String, Address),
}
