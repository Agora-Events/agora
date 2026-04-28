# Agora Soroban Contracts Overview

This directory contains the Soroban smart contracts for Agora's on-chain event and ticketing flow.

## Tech Stack and Layout

This project uses a Rust `cargo` workspace, not Scarb. Build, test, and package management all run through the files in this directory:

- `Cargo.toml`: workspace definition for the Soroban contracts
- `contracts/event_registry`: event lifecycle, organizer controls, inventory, loyalty, staking, and governance
- `contracts/ticket_payment`: ticket purchases, escrow, refunds, settlement, transfers, auctions, and payment-side governance
- `scripts/deploy_devnet.sh`: deploys or upgrades both contracts on Stellar testnet/devnet-style environments
- `scripts/generate_coverage.sh`: generates coverage artifacts for the `ticket-payment` crate

## Contract Catalog

### `event_registry`

`event_registry` is the source of truth for event state. It stores event metadata, tier configuration, organizer ownership, inventory counters, promo settings, scanner permissions, loyalty profiles, staking records, and multi-admin governance proposals.

Key storage keys in [`contracts/event_registry/src/types.rs`](./contracts/event_registry/src/types.rs):

- `Event(event_id)`: full `EventInfo` record for an event
- `OrganizerEvent`, `OrganizerEventShard`, `OrganizerEventCount`: organizer-to-event indexes using sharded storage
- `EventReceipt`, `OrganizerReceipt*`: lightweight archived-event history
- `PlatformWallet`, `PlatformFee`, `TicketPaymentContract`, `Initialized`: core contract configuration
- `MultiSigConfig`, `ProposalCounter`, `Proposal`, `ActiveProposals`: admin governance state
- `Series`, `SeriesPass`, `HolderSeriesPass`, `SeriesEvent`: series and season-pass support
- `BlacklistedOrganizer`, `BlacklistLog`: organizer moderation and audit trail
- `AuthorizedScanner`: per-event scanner authorization
- `GuestProfile`: loyalty tracking for attendees
- `OrganizerStake`, `MinStakeAmount`, `StakingToken`, `TotalStaked`, `StakersList`: organizer staking and verification
- `TokenWhitelist`, `GlobalPromoBps`, `PromoExpiry`, `GlobalEventCount`, `GlobalActiveEventCount`, `GlobalTicketsSold`: platform-wide policy and aggregate counters

Main public functions in [`contracts/event_registry/src/lib.rs`](./contracts/event_registry/src/lib.rs):

- `initialize(admin, platform_wallet, platform_fee_percent, usdc_token)`: one-time setup; stores admin config and whitelists the initial payment token
- `get_version()` / `version()`: contract version helpers
- `register_event(args)`: creates a new event with metadata, tier map, supply limits, refund settings, and optional sales-goal configuration
- `get_event(event_id)`: returns the current `EventInfo`
- `get_event_payment_info(event_id)`: returns payment-facing config such as payment wallet, fee settings, and tiers
- `update_event_status(event_id, is_active)`: toggles whether an event is accepting payments
- `cancel_event(event_id)`: permanently cancels an event
- `archive_event(event_id)`: removes full event state and retains a minimal receipt for historical lookup
- `update_metadata(event_id, new_metadata_cid)`: updates event metadata CID
- `store_event(event_info)`: internal-style public entrypoint used to persist event state
- `get_organizer_address(event_id)`: returns the organizer for an event
- `get_total_tickets_sold(event_id)`: returns sold inventory for an event
- `get_managed_events_count()`: total events ever registered
- `get_active_events_count()`: total currently active events
- `get_global_tickets_sold()`: aggregate platform ticket sales
- `event_exists(event_id)`: quick existence check
- `get_organizer_events(organizer)`: returns event IDs owned by an organizer
- `get_organizer_receipts(organizer)`: returns archived event receipts for an organizer
- `set_platform_fee(new_fee_percent)` / `get_platform_fee()`: manage the default platform fee
- `set_custom_event_fee(event_id, custom_fee_bps)`: set or clear a per-event fee override
- `get_admin()` / `set_admin(new_admin)`: legacy single-admin getter/setter retained alongside multisig support
- `get_platform_wallet()`: returns the fee-collection wallet
- `set_ticket_payment_contract(ticket_payment_address)` / `get_ticket_payment_contract()`: links the payment contract allowed to mutate inventory
- `increment_inventory(event_id, tier_id, quantity)`: increases ticket counters after successful purchases
- `decrement_inventory(event_id, tier_id)`: decreases counters after refunds or reversals
- `register_series(series_id, name, event_ids, organizer_address, metadata_cid)`: groups multiple events into a series
- `get_series(series_id)`: fetches series metadata
- `issue_series_pass(pass_id, series_id, holder, usage_limit, expires_at)`: mints a reusable series pass
- `get_series_pass(pass_id)` / `get_holder_series_pass(holder, series_id)`: retrieves series-pass records
- `blacklist_organizer(organizer_address, reason)` / `remove_from_blacklist(organizer_address, reason)`: moderation controls for organizers
- `is_organizer_blacklisted(organizer_address)` / `get_blacklist_audit_log()`: moderation queries
- `set_global_promo(global_promo_bps, promo_expiry)` / `get_global_promo_bps()` / `get_promo_expiry()`: global promo configuration
- `postpone_event(event_id, grace_period_end)`: marks an event as postponed and opens a refund grace period
- `authorize_scanner(event_id, scanner)` / `is_scanner_authorized(event_id, scanner)`: scanner authorization used by ticket check-in
- `set_staking_config(token, min_stake_amount)`: configures organizer staking
- `stake_collateral(organizer, amount)` / `unstake_collateral(organizer)`: manages organizer collateral
- `distribute_staker_rewards(caller, total_reward)` / `claim_staker_rewards(organizer)`: reward distribution and claims for stakers
- `get_organizer_stake(organizer)` / `is_organizer_verified(organizer)`: staking status lookups
- `update_loyalty_score(caller, guest, tickets_purchased, amount_spent)`: updates attendee loyalty after ticket activity
- `get_guest_profile(guest)` / `get_loyalty_discount_bps(guest)`: loyalty reads used by the payment contract
- `get_multisig_config()` / `is_admin(address)` / `set_multisig_config(caller, admins, threshold)`: multisig configuration
- `propose_parameter_change(proposer, change, expiry_ledgers)`: generic governance proposal creation
- `propose_add_admin(proposer, admin, expiry_ledgers)` / `propose_remove_admin(proposer, admin, expiry_ledgers)` / `propose_set_threshold(proposer, threshold, expiry_ledgers)` / `propose_set_platform_wallet(proposer, wallet, expiry_ledgers)`: convenience governance proposal helpers
- `approve_proposal(approver, proposal_id)` / `execute_proposal(executor, proposal_id)`: multisig approval and execution
- `get_proposal(proposal_id)` / `get_active_proposals()`: governance queries
- `upgrade(new_wasm_hash)`: upgrades the contract code

### `ticket_payment`

`ticket_payment` handles the monetary side of the platform. It validates event payment settings against `event_registry`, processes purchases, keeps escrow balances, settles platform fees, supports refunds and transfers, and emits payment-centric events for indexers and off-chain services.

Key storage keys in [`contracts/ticket_payment/src/types.rs`](./contracts/ticket_payment/src/types.rs):

- `Payment(payment_id)`: full payment record
- `EventPayment*`, `BuyerPayment*`, `EventPaymentStatus*`: sharded indexes for event, buyer, and status-based lookups
- `Balances(event_id)`: escrow and organizer/platform amounts for an event
- `Admin`, `UsdcToken`, `PlatformWallet`, `EventRegistry`, `Initialized`: base contract configuration
- `TokenWhitelist`, `OracleAddress`, `SlippageBps`: accepted assets and pricing controls
- `TransferFee(event_id)`: secondary transfer fee per event
- `BulkRefundIndex`, `PartialRefundIndex`, `PartialRefundPercentage`, `DisputeStatus(event_id)`, `IsPaused`: operational safety and refund state
- `TotalVolumeProcessed`, `TotalFeesCollected(token)`, `ActiveEscrowTotal`, `ActiveEscrowByToken(token)`: protocol-wide accounting
- `DiscountCodeHash`, `DiscountCodeUsed`: discount-code registration and redemption tracking
- `WithdrawalCap`, `DailyWithdrawalAmount`: withdrawal throttling
- `HighestBid`, `AuctionClosed`: auction state
- `Governor`, `TotalGovernors`, `Proposal`, `ProposalCount`: payment-side governance

Main responsibilities in [`contracts/ticket_payment/src/contract.rs`](./contracts/ticket_payment/src/contract.rs):

- Initializes with admin, payment token, platform wallet, and linked `event_registry` contract
- Processes ticket purchases and updates event inventory through `event_registry`
- Confirms payments and records transaction hashes
- Supports guest refunds, admin refunds, automatic refunds, bulk refunds, and partial refunds
- Tracks event escrow balances and organizer/platform settlement amounts
- Handles organizer withdrawals, platform fee settlement, revenue claims, and withdrawal caps
- Supports ticket check-in, transfers, resale fee controls, and event disputes
- Integrates optional price-oracle-based asset pricing and token whitelisting
- Supports tier auctions, bid placement, auction closeout, and governance proposals for contract parameters

## Build and Test

Run commands from [`contract/`](./).

### Build all contracts

```bash
cargo build --target wasm32-unknown-unknown --release
```

### Run all tests

```bash
cargo test
```

### Run a single contract's tests

```bash
cargo test -p event-registry
cargo test -p ticket-payment
```

### Generate coverage for `ticket-payment`

`scripts/generate_coverage.sh` expects `cargo-llvm-cov` to be installed and writes outputs under `coverage/`.

```bash
./scripts/generate_coverage.sh
```

## Deployment

The provided deployment flow is in [`scripts/deploy_devnet.sh`](./scripts/deploy_devnet.sh). It is a Bash script that:

1. Loads environment variables from `.env.devnet`
2. Builds the WASM artifacts with Cargo
3. Deploys or upgrades `event_registry`
4. Initializes `event_registry`
5. Deploys or upgrades `ticket_payment`
6. Initializes `ticket_payment`
7. Links `ticket_payment` back into `event_registry` via `set_ticket_payment_contract`

Required environment variables include:

- `SOROBAN_NETWORK_PASSPHRASE`
- `SOROBAN_RPC_URL`
- `SOROBAN_ACCOUNT_SECRET`
- `ADMIN_ADDRESS`
- `PLATFORM_WALLET`

Optional deployment inputs:

- `USDC_TOKEN_ADDRESS`: existing Stellar asset contract to use for payments; otherwise the script expects you to provide or deploy a mock token separately
- `EVENT_REGISTRY_ID`
- `TICKET_PAYMENT_ID`

Typical usage:

```bash
./scripts/deploy_devnet.sh
```

Upgrade existing deployments:

```bash
./scripts/deploy_devnet.sh --upgrade
```

## Storage Model

Both contracts keep their actual `env.storage()` calls in `storage.rs` instead of scattering them across entrypoints.

- `storage.rs` acts as a thin storage access layer around Soroban persistent storage
- key creation is centralized through the `DataKey` enums in each contract's `types.rs`
- large append-only lists are sharded to avoid oversized storage entries
- counters and indexes are updated together so read paths such as "get organizer events" or "get buyer payments" stay efficient
- the payment contract stores accounting state separately from registry state, while the registry remains authoritative for event metadata and inventory

## Events

Events are defined in each contract's `events.rs` and are emitted with Soroban topics so off-chain indexers can react to contract activity.

### `event_registry` events

Defined in [`contracts/event_registry/src/events.rs`](./contracts/event_registry/src/events.rs):

- `ContractInitialized`
- `ContractUpgraded`
- `EventRegistered`
- `EventStatusUpdated`
- `EventCancelled`
- `EventArchived`
- `MetadataUpdated`
- `FeeUpdated`
- `InventoryIncremented`
- `InventoryDecremented`
- `OrganizerBlacklisted`
- `OrganizerRemovedFromBlacklist`
- `EventsSuspended`
- `GlobalPromoUpdated`
- `EventPostponed`
- `ScannerAuthorized`
- `GoalMet`
- `CollateralStaked`
- `CollateralUnstaked`
- `StakerRewardsDistributed`
- `StakerRewardsClaimed`
- `LoyaltyScoreUpdated`
- `CustomFeeSet`
- admin/governance events including proposal creation, approval/execution, and admin updates

### `ticket_payment` events

Defined in [`contracts/ticket_payment/src/events.rs`](./contracts/ticket_payment/src/events.rs):

- `ContractInitialized`
- `PaymentProcessed`
- `PaymentStatusChanged`
- `TicketTransferred`
- `PriceSwitched`
- `BulkRefundProcessed`
- `PartialRefundProcessed`
- `DiscountCodeApplied`
- `GlobalPromoApplied`
- `RevenueClaimed`
- `FeeSettled`
- `ContractPaused`
- `DisputeStatusChanged`
- `TicketCheckedIn`
- `BidPlaced`
- `AuctionClosed`
- governance events for proposal creation, voting, and execution
- `ContractVerificationFailed`

## Contract Interaction Summary

The contracts are intentionally split by responsibility:

- `event_registry` owns event metadata, organizer policy, inventory truth, loyalty, and staking
- `ticket_payment` owns funds movement, escrow accounting, refunds, fee settlement, and purchase lifecycle
- `ticket_payment` calls into `event_registry` to read event payment settings and to increment or decrement inventory after payment state changes

## PR Note

When you open the PR for this documentation task, link the issue and include:

```text
Closes #issue_number
```
