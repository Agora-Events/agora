# Ticket Payment Contract

A Soroban smart contract for handling ticket purchases, refunds, and payment distribution on the Stellar blockchain.

## Overview

The Ticket Payment contract manages all financial transactions for event ticketing, including:

- **Ticket Purchases**: Buy tickets with support for tiered pricing and early bird discounts
- **Refund Processing**: Automated refund handling with configurable policies
- **Payment Distribution**: Split payments between organizers and platform
- **Auction System**: Dynamic pricing through auction mechanisms
- **Resale Marketplace**: Secondary market for ticket resales
- **Staker Rewards**: Distribute rewards to event stakers
- **Price Oracle Integration**: USD price feeds for stable pricing

## Key Features

### Payment Processing
- Multi-tier pricing with automatic tier selection
- Early bird discount calculations
- Platform fee and custom fee handling
- Atomic payment splits (organizer + platform)
- Support for Stellar native token (XLM) and SAC-20 tokens

### Refund System
- Configurable refund deadlines
- Restocking fee deductions
- Resale cap enforcement
- Automatic inventory updates
- Audit trail for all refunds

### Auction Mechanism
- Dutch auction support for dynamic pricing
- Configurable auction parameters
- Automatic price adjustments
- Winner determination and settlement

### Resale Marketplace
- List tickets for resale (`resale.rs`)
- Hard price cap enforcement — organizer-set `resale_cap_bps`, defaulting to
  110% of face value when the organizer has not set one
- Automatic organizer royalty (default 5%) deducted at settlement
- Atomic swap: payment, royalty and ownership transfer in one invocation
- The ticket's check-in secret is handed over off-chain, sealed to the buyer's
  X25519 key — see `server/src/handlers/marketplace.rs`

### Rewards Distribution
- Track staker contributions
- Distribute event profits to stakers
- Claim rewards functionality
- Pro-rata distribution based on stake

## Functions

### Purchase Operations
- `buy_ticket(event_id, tier_id, quantity, buyer)` - Purchase tickets
- `buy_ticket_with_auction(event_id, tier_id, quantity, buyer, bid_price)` - Buy via auction
- `batch_buy_tickets(purchases)` - Purchase multiple tickets in one transaction

### Refund Operations
- `request_refund(event_id, tier_id, quantity, buyer)` - Request ticket refund
- `process_refund(event_id, tier_id, payment_id)` - Process approved refund
- `batch_refund(refund_requests)` - Process multiple refunds

### Resale Operations
Listings are keyed by the `payment_id` of the ticket being sold, so a ticket
has at most one listing at a time.

- `list_for_resale(payment_id, price_usdc)` - List ticket on resale market
- `cancel_resale_listing(payment_id)` - Cancel a listing (seller only)
- `purchase_resale_ticket(payment_id, buyer)` - Buy from resale market
- `get_resale_listing(payment_id)` - Read a listing (including sold/cancelled)
- `get_max_resale_price(payment_id)` - Highest legal listing price
- `set_resale_royalty_bps(event_id, royalty_bps)` - Organizer royalty rate
- `get_resale_royalty_bps(event_id)` - Read the royalty rate

The buyer must `approve` the contract for the listing price before calling
`purchase_resale_ticket`, the same two-transaction pattern `process_payment`
uses.

### Auction Operations
- `create_auction(event_id, tier_id, start_price, end_price, duration)` - Create auction
- `place_bid(auction_id, amount)` - Place auction bid
- `settle_auction(auction_id)` - Settle completed auction

### Administration
- `set_price_oracle_contract(address)` - Set price oracle contract
- `set_event_registry_contract(address)` - Set event registry contract
- `set_transfer_fee(fee)` - Set token transfer fee
- `pause()` - Pause contract operations
- `unpause()` - Resume operations

### Reward Operations
- `distribute_staker_rewards(event_id)` - Distribute rewards to stakers
- `claim_staker_rewards(staker, event_id)` - Claim accumulated rewards

## Data Structures

### Payment
```rust
pub struct Payment {
    pub payment_id: u64,
    pub event_id: String,
    pub tier_id: String,
    pub buyer: Address,
    pub quantity: u32,
    pub total_amount: i128,
    pub platform_fee: i128,
    pub organizer_amount: i128,
    pub status: PaymentStatus,
    pub timestamp: u64,
}
```

### PaymentStatus
```rust
pub enum PaymentStatus {
    Pending,
    Completed,
    Refunded,
    Failed,
}
```

### ResaleListing
```rust
pub struct ResaleListing {
    pub payment_id: String,
    pub event_id: String,
    pub seller: Address,
    pub price: i128,
    pub token_address: Address,
    /// Ceiling this listing was validated against.
    pub max_price: i128,
    pub royalty_bps: u32,
    pub status: ResaleStatus, // Active | Cancelled | Sold
    pub created_at: u64,
    pub updated_at: u64,
}
```

### Auction
```rust
pub struct Auction {
    pub auction_id: u64,
    pub event_id: String,
    pub tier_id: String,
    pub start_price: i128,
    pub end_price: i128,
    pub start_time: u64,
    pub end_time: u64,
    pub status: AuctionStatus,
    pub winner: Option<Address>,
    pub winning_bid: Option<i128>,
}
```

## Events Emitted

The contract emits events for tracking and integration:

- `TicketPurchased` - Ticket purchase completed
- `RefundRequested` - Refund initiated
- `RefundProcessed` - Refund completed
- `PaymentDistributed` - Payment split to parties
- `ResaleListed` - Ticket listed for resale
- `ResaleCancelled` - Listing withdrawn by the seller
- `ResalePurchased` - Resale settled (carries price, royalty, seller proceeds)
- `AuctionCreated` - New auction started
- `BidPlaced` - Auction bid submitted
- `AuctionSettled` - Auction completed
- `StakerRewardsDistributed` - Rewards distributed
- `StakerRewardsClaimed` - Rewards claimed
- `ContractPaused` - Contract paused
- `ContractUnpaused` - Contract resumed
- `ParameterChanged` - Contract parameter updated

## Development

### Prerequisites
- Rust toolchain with `wasm32-unknown-unknown` target
- Soroban CLI
- Stellar testnet access (optional for integration tests)

### Building

```bash
cd contract
cargo build --target wasm32-unknown-unknown --release
```

### Testing

Run unit tests:

```bash
cargo test -p ticket-payment
```

Run end-to-end tests:

```bash
cargo test -p ticket-payment --test test_e2e
```

Run with output:

```bash
cargo test -p ticket-payment -- --nocapture
```

### Linting

```bash
cargo clippy --all-targets
```

### Coverage

Generate coverage artifacts:

```bash
./scripts/generate_coverage.sh
```

## Integration

The Ticket Payment contract integrates with:

1. **Event Registry Contract**: Validates events, checks inventory, retrieves pricing
2. **Price Oracle Contract**: Fetches USD exchange rates for stable pricing
3. **Stellar Token Contracts**: Handles XLM and SAC-20 token transfers

### Example Usage

```rust
// Setup
let env = Env::default();
let contract_id = env.register(TicketPaymentContract, ());
let client = TicketPaymentContractClient::new(&env, &contract_id);

// Configure contracts
client.set_event_registry_contract(&event_registry_address);
client.set_price_oracle_contract(&price_oracle_address);

// Purchase a ticket
let payment = client.buy_ticket(
    &String::from_str(&env, "event_001"),
    &String::from_str(&env, "general"),
    &2,  // quantity
    &buyer_address
);

// Request a refund
client.request_refund(
    &String::from_str(&env, "event_001"),
    &String::from_str(&env, "general"),
    &1,
    &buyer_address
);
```

## Error Handling

The contract returns `TicketPaymentError` enum variants:

- `NotInitialized` - Contract not initialized
- `ContractPaused` - Operations are paused
- `EventNotFound` - Event doesn't exist
- `EventInactive` - Event is not active
- `EventCancelled` - Event has been cancelled
- `TierNotFound` - Ticket tier doesn't exist
- `InsufficientSupply` - Not enough tickets available
- `InvalidQuantity` - Invalid quantity specified
- `RefundDeadlinePassed` - Too late for refund
- `InvalidResalePrice` - Price exceeds cap
- `AuctionNotEnded` - Auction still active
- `Outbid` - Another bidder outbid you
- `UnauthorizedCaller` - Missing authentication
- And many more...

## Fee Calculation

Fees are calculated in basis points (1/10000):

```rust
// Platform fee (e.g., 5% = 500 bps)
let platform_fee = (amount * platform_fee_bps) / 10000;

// Custom event fee (overrides platform fee if set)
let fee_bps = custom_fee_bps.unwrap_or(platform_fee_bps);
let fee = (amount * fee_bps) / 10000;

// Organizer receives remainder
let organizer_amount = amount - fee;
```

## Security Considerations

- All functions requiring authorization use `require_auth()`
- Reentrancy guards on payment functions
- Integer overflow/underflow checks
- Address validation before transfers
- Pause mechanism for emergency stops
- Multi-sig support for admin functions

## License

See the main project LICENSE.md file.
