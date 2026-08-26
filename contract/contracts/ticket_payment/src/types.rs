use soroban_sdk::{contracttype, Address, String};

/// Transfer fee in basis points applied to ticket resale transactions (1 % = 100 bps).
pub const TRANSFER_FEE_BPS: u32 = 100;
/// Maximum basis points value representing 100%. Used for fee calculations.
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
/// Configuration for a Dutch-auction ticket tier.
pub struct AuctionConfig {
    /// Starting bid price in stroops.
    pub start_price: i128,
    /// Unix timestamp when the auction closes; no new bids are accepted after this.
    pub end_time: u64,
    /// Minimum increment above the current highest bid required for a new bid.
    pub min_increment: i128,
}

#[contracttype]
#[derive(Clone, Debug, Eq, PartialEq)]
/// A time-bounded price point used in scheduled pricing tiers.
pub struct PriceSchedule {
    /// Ticket price in stroops for this schedule window.
    pub price: i128,
    /// Unix timestamp until which this price is active.
    pub valid_until: u64,
}

#[contracttype]
#[derive(Clone, Debug, Eq, PartialEq)]
/// Current lifecycle state of a payment / ticket.
pub enum PaymentStatus {
    /// Payment received and held in escrow; ticket not yet confirmed.
    Pending,
    /// Payment confirmed on-chain; ticket is valid.
    Confirmed,
    /// Payment has been refunded to the buyer.
    Refunded,
    /// Payment failed and funds were not escrowed.
    Failed,
    /// Ticket has been scanned at the event gate.
    CheckedIn,
}

#[contracttype]
#[derive(Clone, Debug, Eq, PartialEq)]
/// Full payment record stored in contract persistent storage.
pub struct Payment {
    /// Unique identifier for this payment.
    pub payment_id: String,
    /// The event this ticket belongs to.
    pub event_id: String,
    /// The wallet that submitted the payment transaction.
    pub buyer_address: Address,
    /// The recipient who owns the ticket (may differ from buyer for gifted tickets).
    pub owner_address: Address,
    /// The ticket tier purchased.
    pub ticket_tier_id: String,
    /// Token contract used for payment.
    pub token_address: Address,
    /// Per-ticket amount paid in stroops.
    pub amount: i128,
    /// Platform fee portion retained by Agora.
    pub platform_fee: i128,
    /// Organizer revenue portion after fees.
    pub organizer_amount: i128,
    /// Current lifecycle state of this payment.
    pub status: PaymentStatus,
    /// On-chain transaction hash recorded at confirmation.
    pub transaction_hash: String,
    /// Ledger timestamp when the payment was created.
    pub created_at: u64,
    /// Ledger timestamp when the payment was confirmed, if applicable.
    pub confirmed_at: Option<u64>,
    /// Amount refunded so far (supports partial refunds).
    pub refunded_amount: i128,
    /// Whether this ticket is soulbound (non-transferable).
    pub is_soulbound: bool,
    /// Ledger timestamp of the last check-in scan; 0 if never scanned.
    pub last_checked_in_at: u64,
    /// Referral reward amount paid out from the platform fee.
    pub referral_amount: i128,
    /// Optional referrer wallet address.
    pub referrer: Option<Address>,
}

#[contracttype]
#[derive(Clone, Debug, Eq, PartialEq)]
/// Escrow balances held by the contract for a single event.
pub struct EventBalance {
    /// Amount owed to the organizer, pending withdrawal.
    pub organizer_amount: i128,
    /// Cumulative organizer amount already withdrawn.
    pub total_withdrawn: i128,
    /// Platform fee amount collected for this event.
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
