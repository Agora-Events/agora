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
    FeeSettled,
    GlobalPromoApplied,
    ContractPaused,
    DisputeStatusChanged,
    PartialRefundProcessed,
    TicketCheckedIn,
    BidPlaced,
    AuctionClosed,
    ProposalCreated,
    ProposalVoted,
    GovernanceActionExecuted,
    ContractVerificationFailed,
    EventCancelled,
    CancellationRefundClaimed,
    PoapMinted,
    ResaleListed,
    ResaleCancelled,
    ResalePurchased,
    // Dynamic pricing events (Issue #1175)
    DutchAuctionCreated,
    DutchAuctionPurchaseCommitted,
    DutchAuctionPurchaseRevealed,
    BondingCurveCreated,
    BondingCurvePurchase,
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

#[contracttype]
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct FeeSettledEvent {
    pub event_id: String,
    pub platform_wallet: Address,
    pub fee_amount: i128,
    pub fee_bps: u32,
    pub timestamp: u64,
}

#[contracttype]
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct GlobalPromoAppliedEvent {
    pub payment_id: String,
    pub event_id: String,
    pub promo_bps: u32,
    pub discount_amount: i128,
    pub timestamp: u64,
}

#[contracttype]
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct ContractPausedEvent {
    pub paused: bool,
    pub timestamp: u64,
}

#[contracttype]
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct DisputeStatusChangedEvent {
    pub event_id: String,
    pub is_disputed: bool,
    pub timestamp: u64,
}

#[contracttype]
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct PartialRefundProcessedEvent {
    pub event_id: String,
    pub refund_count: u32,
    pub total_refunded: i128,
    pub percentage_bps: u32,
    pub timestamp: u64,
}

#[contracttype]
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct TicketCheckedInEvent {
    pub payment_id: String,
    pub event_id: String,
    pub attendee: Address,
    pub scanner: Address,
    pub timestamp: u64,
}

#[contracttype]
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct BidPlacedEvent {
    pub event_id: String,
    pub tier_id: String,
    pub bidder: Address,
    pub amount: i128,
    pub timestamp: u64,
}

#[contracttype]
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct AuctionClosedEvent {
    pub event_id: String,
    pub tier_id: String,
    pub winner: Address,
    pub amount: i128,
    pub timestamp: u64,
}

#[contracttype]
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct ProposalCreatedEvent {
    pub proposal_id: u64,
    pub proposer: Address,
    pub change: crate::types::ParameterChange,
    pub timestamp: u64,
}

#[contracttype]
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct ProposalVotedEvent {
    pub proposal_id: u64,
    pub voter: Address,
    pub timestamp: u64,
}

#[contracttype]
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct GovernanceActionExecutedEvent {
    pub proposal_id: u64,
    pub change: crate::types::ParameterChange,
    pub timestamp: u64,
}

#[contracttype]
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct ContractVerificationFailedEvent {
    pub missing_key: String,
    pub timestamp: u64,
}

#[contracttype]
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct EventCancelledEvent {
    pub event_id: String,
    pub organizer: Address,
    pub timestamp: u64,
}

#[contracttype]
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct CancellationRefundClaimedEvent {
    pub payment_id: String,
    pub event_id: String,
    pub buyer: Address,
    pub amount: i128,
    pub timestamp: u64,
}

/// Emitted when a non-transferable POAP NFT is minted for a successful check-in.
#[contracttype]
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct PoapMintedEvent {
    /// The payment / ticket that triggered the mint.
    pub payment_id: String,
    pub event_id: String,
    /// The attendee who earned the POAP.
    pub attendee: Address,
    /// Ledger timestamp at mint time.
    pub timestamp: u64,
}

/// Emitted when a ticket holder lists their ticket on the secondary market.
#[contracttype]
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct ResaleListedEvent {
    pub payment_id: String,
    pub event_id: String,
    pub seller: Address,
    /// Asking price in token base units.
    pub price: i128,
    /// The cap this listing was validated against, so indexers can show
    /// how much headroom the organizer allows without re-reading the registry.
    pub max_price: i128,
    pub timestamp: u64,
}

/// Emitted when a seller withdraws their own listing before it sells.
#[contracttype]
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct ResaleCancelledEvent {
    pub payment_id: String,
    pub event_id: String,
    pub seller: Address,
    pub timestamp: u64,
}

/// Emitted on a completed atomic resale: payment settled and ownership moved.
#[contracttype]
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct ResalePurchasedEvent {
    pub payment_id: String,
    pub event_id: String,
    pub seller: Address,
    pub buyer: Address,
    /// Gross price paid by the buyer.
    pub price: i128,
    /// Portion routed to the organizer as a royalty.
    pub royalty: i128,
    /// Net proceeds received by the seller (`price - royalty`).
    pub seller_proceeds: i128,
    pub timestamp: u64,
}

// ── Dynamic Pricing Events (Issue #1175) ──────────────────────────────────────

#[contracttype]
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct DutchAuctionCreatedEvent {
    pub event_id: String,
    pub tier_id: String,
    pub start_price: i128,
    pub reserve_price: i128,
    pub start_time: u64,
    pub end_time: u64,
    pub exponential: bool,
    pub timestamp: u64,
}

#[contracttype]
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct DutchAuctionCommitEvent {
    pub event_id: String,
    pub tier_id: String,
    pub buyer: Address,
    /// Price locked in at commit time (stroops).
    pub committed_price: i128,
    /// Timestamp at which the commit expires.
    pub expires_at: u64,
}

#[contracttype]
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct DutchAuctionRevealEvent {
    pub event_id: String,
    pub tier_id: String,
    pub buyer: Address,
    /// Final approved price charged to the buyer (stroops).
    pub approved_price: i128,
    pub timestamp: u64,
}

#[contracttype]
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct BondingCurveCreatedEvent {
    pub event_id: String,
    pub tier_id: String,
    /// Amplitude `a` (scaled by PARAM_SCALE).
    pub a_scaled: i128,
    pub b_exponent: u32,
    /// Base price floor `c` (stroops).
    pub c_base: i128,
    pub initial_supply: u32,
    pub timestamp: u64,
}

#[contracttype]
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct BondingCurvePurchaseEvent {
    pub event_id: String,
    pub tier_id: String,
    pub buyer: Address,
    pub quantity: u32,
    /// Total cost charged (stroops).
    pub total_cost: i128,
    /// Remaining supply after this purchase.
    pub remaining_supply: u32,
    pub timestamp: u64,
}
