# Event Registry Contract

A Soroban smart contract for managing event registration, ticketing, and organizer operations on the Stellar blockchain.

## Overview

The Event Registry contract provides a comprehensive decentralized event management system with the following features:

- **Event Registration**: Create and manage events with tiered pricing
- **Series Management**: Group multiple events into series with season passes
- **Multi-Sig Admin**: Configurable multi-signature administration
- **Fee Management**: Platform fees and custom event-specific fees
- **Inventory Tracking**: Real-time ticket supply management per tier
- **Event Lifecycle**: Support for active, cancelled, postponed, and archived states
- **Organizer Staking**: Collateral requirements for event organizers
- **Scanner Authorization**: Decentralized ticket verification system
- **Loyalty Program**: Track attendee loyalty scores

## Key Features

### Event Management
- Register events with customizable metadata (IPFS CID)
- Configure tiered pricing with early bird discounts
- Set maximum supply limits per event and per tier
- Update event status (active, cancelled, postponed)
- Archive events after completion

### Series & Season Passes
- Group related events into series
- Issue reusable season passes with usage limits
- Track pass holder attendance across multiple events

### Fee Structure
- Platform-wide fee percentage (basis points)
- Custom per-event fee overrides
- Fee distribution to platform wallet

### Security Features
- Multi-signature admin configuration
- Organizer blacklist management
- Address validation and authentication
- Event emission for off-chain tracking

## Functions

### Initialization
- `initialize(admin, platform_wallet, platform_fee_percent)` - Initialize contract with admin and fee settings

### Event Operations
- `register_event(args)` - Register a new event with tiered pricing
- `get_event(event_id)` - Retrieve event information
- `get_event_payment_info(event_id)` - Get payment details and fee structure
- `update_event_status(event_id, is_active)` - Activate or deactivate an event
- `cancel_event(event_id)` - Cancel an event (irreversible)
- `archive_event(event_id)` - Archive a completed/cancelled event
- `update_metadata(event_id, new_metadata_cid)` - Update event metadata

### Series Operations
- `register_series(series_id, name, event_ids, organizer, metadata)` - Create an event series
- `get_series(series_id)` - Retrieve series information
- `issue_series_pass(pass_id, series_id, holder, usage_limit, expires_at)` - Issue a season pass
- `get_series_pass(pass_id)` - Retrieve pass information
- `get_holder_series_pass(holder, series_id)` - Get pass for a specific holder

### Administration
- `set_platform_fee(new_fee_percent)` - Update platform fee
- `set_custom_event_fee(event_id, custom_fee_bps)` - Set custom fee for specific event
- `get_admin()` - Get administrator address
- `get_platform_wallet()` - Get platform wallet address
- `set_ticket_payment_contract(address)` - Set authorized payment contract

### Inventory Management
- `increment_inventory(event_id, tier_id, quantity)` - Increment ticket sales (payment contract only)
- `decrement_inventory(event_id, tier_id)` - Decrement on refund (payment contract only)

## Data Structures

### EventInfo
```rust
pub struct EventInfo {
    pub event_id: String,
    pub organizer_address: Address,
    pub payment_address: Address,
    pub platform_fee_percent: u32,
    pub is_active: bool,
    pub status: EventStatus,
    pub created_at: u64,
    pub metadata_cid: String,
    pub max_supply: i128,
    pub current_supply: i128,
    pub tiers: Map<String, TicketTier>,
    // ... additional fields
}
```

### TicketTier
```rust
pub struct TicketTier {
    pub name: String,
    pub price: i128,
    pub early_bird_price: i128,
    pub early_bird_deadline: u64,
    pub tier_limit: i128,
    pub current_sold: i128,
    pub is_refundable: bool,
    // ... additional fields
}
```

### SeriesRegistry
```rust
pub struct SeriesRegistry {
    pub series_id: String,
    pub name: String,
    pub event_ids: Vec<String>,
    pub organizer_address: Address,
    pub metadata_cid: Option<String>,
}
```

## Events Emitted

The contract emits events for all significant state changes:
- `ContractInitialized` - Contract initialization
- `EventRegistered` - New event registration
- `EventStatusUpdated` - Event status change
- `EventCancelled` - Event cancellation
- `EventArchived` - Event archival
- `MetadataUpdated` - Metadata update
- `FeeUpdated` - Platform fee change
- `CustomFeeSet` - Custom fee configuration
- `InventoryIncremented` - Ticket sale
- `InventoryDecremented` - Ticket refund
- `GoalMet` - Sales target achieved
- `SeriesPassIssued` - Season pass creation
- And many more for staking, scanner auth, loyalty, etc.

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
cargo test -p event-registry
```

Run with output:

```bash
cargo test -p event-registry -- --nocapture
```

### Linting

```bash
cargo clippy --all-targets
```

## Integration

The Event Registry contract is designed to work with the companion [Ticket Payment](../ticket_payment/) contract. The payment contract is authorized to call inventory management functions.

### Example Usage

```rust
// Initialize the contract
let admin = Address::generate(&env);
let platform_wallet = Address::generate(&env);
client.initialize(&admin, &platform_wallet, &500); // 5% fee

// Register an event
let tiers = Map::new(&env);
tiers.set(
    String::from_str(&env, "general"),
    TicketTier {
        name: String::from_str(&env, "General Admission"),
        price: 1000_000_000, // 1000 XLM in stroops
        tier_limit: 100,
        // ... other fields
    }
);

client.register_event(&EventRegistrationArgs {
    event_id: String::from_str(&env, "event_001"),
    organizer_address: organizer.clone(),
    payment_address: organizer.clone(),
    metadata_cid: String::from_str(&env, "ipfs_cid"),
    max_supply: 100,
    tiers,
    // ... other args
});
```

## Error Handling

The contract returns `EventRegistryError` enum variants for various failure conditions:
- `NotInitialized` - Contract not initialized
- `AlreadyInitialized` - Double initialization attempt
- `EventNotFound` - Event ID doesn't exist
- `EventAlreadyExists` - Duplicate event registration
- `UnauthorizedCaller` - Missing or failed authentication
- `OrganizerBlacklisted` - Organizer is blacklisted
- `InvalidFeePercent` - Fee exceeds 100%
- `MaxSupplyExceeded` - Ticket limit reached
- `TierSoldOut` - Tier limit reached
- `EventInactive` - Event is not active
- And many more...

## License

See the main project LICENSE.md file.
