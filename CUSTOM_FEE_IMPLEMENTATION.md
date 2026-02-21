# Custom Fee Override Implementation

## Summary
Implemented custom fee override system for strategic partnerships and charitable events.

## Changes Made

### 1. Event Registry Contract (`event_registry`)

#### `types.rs`
- ✅ Added `custom_fee_bps: Option<u32>` to `EventInfo` struct
- ✅ Added `CustomEventFee(String)` storage key to `DataKey` enum

#### `events.rs`
- ✅ Added `CustomFeeSet` variant to `AgoraEvent` enum
- ✅ Added `CustomFeeSetEvent` struct with event_id, custom_fee_bps, and timestamp

#### `lib.rs`
- ✅ Updated `register_event` to initialize `custom_fee_bps: None`
- ✅ Updated `get_event_payment_info` to return custom fee when set (takes precedence over global fee)
- ✅ Implemented `set_custom_event_fee` function:
  - Restricted to admin only (multi-sig support via `require_auth()`)
  - Validates fee is <= 10000 basis points
  - Updates event with custom fee
  - Emits `CustomFeeSetEvent`

### 2. Payment Contract (`ticket_payment`)

#### `contract.rs`
- ✅ Added `custom_fee_bps: Option<u32>` to `EventInfo` struct in event_registry module
- ✅ Updated `process_payment` to check custom fee first:
  ```rust
  let fee_bps = event_info.custom_fee_bps.unwrap_or(event_info.platform_fee_percent);
  let platform_fee = (amount * fee_bps as i128) / 10000;
  ```

#### Integration
The payment contract now properly uses the custom fee override during payment processing. When calculating fees in `process_payment`, it checks if `custom_fee_bps` is set and uses it; otherwise, falls back to the global `platform_fee_percent`.

## Usage

### Setting Custom Fee (Admin Only)
```rust
// Set 2% fee (200 basis points) for event
event_registry.set_custom_event_fee(
    env,
    event_id,
    200  // 2% in basis points
);
```

### Fee Priority
1. If `custom_fee_bps` is set → use custom fee
2. Otherwise → use global `platform_fee_percent`

## Security
- ✅ Only platform administrator can call `set_custom_event_fee`
- ✅ Multi-sig support via Stellar's `require_auth()`
- ✅ Fee validation (max 10000 bps = 100%)
- ✅ Event existence validation

## Testing Recommendations
1. Test admin-only access control
2. Test custom fee takes precedence in payment calculations
3. Test fee validation (reject > 10000 bps)
4. Test event not found error handling
5. Test custom fee event emission
