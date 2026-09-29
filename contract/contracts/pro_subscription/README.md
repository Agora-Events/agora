# Pro Subscription Contract

A Soroban smart contract that manages the "Pro" subscription tier for event organizers on Agora.

## Overview

Organizers who want reduced platform fees and other Pro-only benefits pay a
recurring monthly fee in a configurable payment token (e.g. USDC). The
contract tracks each organizer's subscription record, the current monthly
price, and the platform wallet that receives payments.

- The **monthly price** (`pro_monthly_price`) is set at initialization and
  can be updated by the admin at any time via `update_pro_price`. It is
  denominated in the smallest unit of the payment token (stroops for
  Stellar Classic assets).
- The **platform wallet** (`platform_wallet`) is the address that receives
  every subscription and renewal payment. It can be rotated by the admin
  via `update_platform_wallet`, but it can never be set to the contract's
  own address (see [Validation](#validation)).
- Organizers pay with a single **payment token** (`payment_token`) at a
  time, configured at initialization and updatable by the admin via
  `update_payment_token`. Subscribing/renewing requires the organizer to
  have approved the contract to transfer that amount beforehand.
- A **Basic** tier (`register_basic`) is also available: it is free, never
  expires, and is only meant to record that an organizer has opted into the
  platform without paying for Pro.

## Validation

`update_platform_wallet`, `update_payment_token`, `update_admin`, and
`initialize` all reject the contract's own address as the target address
(`ProSubscriptionError::InvalidAddress`), via the shared
`validate_address` helper in `validation.rs`. This prevents accidentally
bricking the contract by pointing critical addresses at itself.

## Functions

| Function | Who can call it | Description |
| --- | --- | --- |
| `initialize(admin, platform_wallet, payment_token, pro_monthly_price)` | Anyone (once) | Sets up the contract. Fails if already initialized, if any address equals the contract address, or if `pro_monthly_price <= 0`. |
| `version()` | Anyone | Returns the crate version compiled into the contract. |
| `subscribe_pro(organizer, months)` | Organizer | Charges `pro_monthly_price * months` from `organizer` to the platform wallet and creates/activates a Pro subscription. Fails if `months == 0`, if the organizer already has an active subscription, or on arithmetic overflow. |
| `renew_subscription(organizer, months)` | Organizer | Charges for `months` more and extends the subscription — from the current expiry if still active, or from now if expired. Fails if no subscription exists yet. |
| `cancel_subscription(organizer)` | Admin | Marks the subscription inactive, removes the organizer from the Pro members list, and decrements the Pro subscriber count. |
| `is_pro_member(organizer)` | Anyone | Returns `true` only if a subscription exists, is active, and has not expired. |
| `get_subscription(organizer)` | Anyone | Returns the full `Subscription` record, or `None` if the organizer never subscribed. |
| `get_subscription_expiry(organizer)` | Anyone | Returns the subscription's `expires_at`, or `None` if the organizer never subscribed. |
| `get_pro_members()` | Anyone | Returns the list of addresses currently in the Pro members list. |
| `get_pro_members_count()` | Anyone | Returns the length of the Pro members list. |
| `register_basic(organizer)` | Organizer | Creates a free, non-expiring Basic subscription. Fails if the organizer already has an active Pro subscription. |
| `get_total_pro_subscriptions()` | Anyone | Returns the running count of active Pro subscriptions (incremented on subscribe, decremented on cancel). |
| `update_pro_price(new_price)` | Admin | Updates the monthly price. Fails if `new_price <= 0`. Emits `PriceUpdated`. |
| `get_pro_monthly_price()` | Anyone | Returns the current monthly price. |
| `get_admin()` | Anyone | Returns the current admin address. |
| `is_initialized()` | Anyone | Returns whether `initialize` has been called. |
| `update_admin(new_admin)` | Admin | Rotates the admin address. |
| `get_platform_wallet()` | Anyone | Returns the current platform wallet address. |
| `update_platform_wallet(new_wallet)` | Admin | Rotates the platform wallet address. |
| `get_payment_token()` | Anyone | Returns the current accepted payment token address. |
| `update_payment_token(new_token)` | Admin | Rotates the accepted payment token address. |

"Admin" calls are enforced with `admin.require_auth()` inside `require_admin`.
"Organizer" calls are enforced with `organizer.require_auth()` on the address
passed in, so an organizer can only ever act on their own subscription.

## Errors

`ProSubscriptionError` (see `error.rs`):

| Variant | Code | Meaning |
| --- | --- | --- |
| `AlreadyInitialized` | 1 | `initialize` was called on a contract that is already initialized. |
| `NotInitialized` | 2 | A call was made before `initialize`, or a required piece of contract state (admin/payment token/platform wallet) is missing. |
| `Unauthorized` | 3 | Reserved for caller-is-not-admin failures (auth is primarily enforced via `require_auth`, which panics rather than returning this variant). |
| `SubscriptionNotFound` | 4 | `renew_subscription` or `cancel_subscription` was called for an organizer with no subscription record. |
| `SubscriptionExpired` | 5 | Reserved for future use — expiry is currently checked by comparing `expires_at` to the ledger timestamp on read, not by transitioning state, so this variant is not emitted today. |
| `SubscriptionInactive` | 6 | Reserved for flows that need to distinguish an inactive-but-present subscription. |
| `InvalidTier` | 7 | Reserved for tier-validation failures. |
| `InvalidPrice` | 8 | `pro_monthly_price`/`new_price` was `<= 0`, or `months == 0` was passed to `subscribe_pro`/`renew_subscription`. |
| `InsufficientPayment` | 9 | Reserved for payment-amount validation. |
| `ArithmeticError` | 10 | An overflow occurred multiplying price by months or adding to an expiry/total. |
| `TransferFailed` | 11 | Reserved for token-transfer failure handling. |
| `InvalidAddress` | 12 | `initialize`, `update_admin`, `update_platform_wallet`, or `update_payment_token` was called with the contract's own address. |
| `SubscriptionAlreadyActive` | 13 | `subscribe_pro` was called while an active, unexpired subscription already exists, or `register_basic` was called while an active Pro subscription exists. |
| `AlreadyPro` | 14 | Reserved for flows that need to reject a redundant Pro upgrade. |

## Events

Defined in `events.rs` as `ProSubscriptionEvent` topics, each paired with a
`#[contracttype]` payload struct:

| Event | Payload | Emitted by |
| --- | --- | --- |
| `ContractInitialized` | `InitializationEvent { admin, platform_wallet, payment_token, pro_monthly_price, timestamp }` | `initialize` |
| `SubscriptionCreated` | `SubscriptionCreatedEvent { organizer, tier, amount_paid, expires_at, timestamp }` | `subscribe_pro` |
| `SubscriptionRenewed` | `SubscriptionRenewedEvent { organizer, amount_paid, new_expiry, timestamp }` | `renew_subscription` |
| `SubscriptionCancelled` | `SubscriptionCancelledEvent { organizer, cancelled_by, timestamp }` | `cancel_subscription` |
| `PriceUpdated` | `PriceUpdatedEvent { old_price, new_price, updated_by, timestamp }` | `update_pro_price` |
| `ProMemberAdded` | `ProMemberAddedEvent { organizer, timestamp }` | `subscribe_pro`, `renew_subscription` |
| `ProMemberRemoved` | `ProMemberRemovedEvent { organizer, timestamp }` | `cancel_subscription` |
| `SubscriptionExpired` | — (topic only, no payload struct) | Reserved; not currently emitted anywhere in `contract.rs`. |

## Development

### Prerequisites
- Rust toolchain with the `wasm32-unknown-unknown` target
- Soroban CLI

### Building

```bash
cd contract
cargo build --target wasm32-unknown-unknown --release
```

### Testing

```bash
cargo test -p pro-subscription
```

Run with output:

```bash
cargo test -p pro-subscription -- --nocapture
```

### Linting

```bash
cargo clippy --all-targets
```

## License

See the main project LICENSE.md file.
