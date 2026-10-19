# Event Garbage Collection Feature

## Overview

The Event Garbage Collection feature allows organizers to archive old events and reclaim storage fees on the Stellar/Soroban blockchain. This feature is designed to reduce long-term storage costs while preserving essential historical data.

## Purpose

On blockchain platforms like Soroban, persistent storage incurs ongoing fees. Events that have concluded and are no longer active accumulate storage costs unnecessarily. The garbage collection feature addresses this by:

1. **Reducing Storage Costs**: Deletes large, non-essential data structures
2. **Preserving History**: Maintains minimal event receipts for historical records
3. **Reclaiming Fees**: Returns storage deposits to organizers
4. **Enforcing Safety**: Requires a 30-day grace period after event end

## How It Works

### Archival Process

When an event is archived, the following happens:

1. **Full EventInfo Deletion**: The complete event record is removed, including:
   - Ticket tiers (pricing, limits, sold counts)
   - Milestone plans
   - Tags and category IDs
   - Banner CIDs
   - Metadata references
   - All other event configuration data

2. **Receipt Creation**: A minimal `EventReceipt` is created containing only:
   - `event_id`: The unique identifier
   - `organizer_address`: Who created the event
   - `total_sold`: Final ticket count
   - `archived_at`: Timestamp of archival

3. **Storage Reclamation**: Soroban automatically reclaims the storage fees associated with the deleted data structures

### Requirements

To archive an event, ALL of the following conditions must be met:

| Requirement | Description |
|-------------|-------------|
| **Organizer Authorization** | Only the event organizer can archive their event |
| **Event Must Be Inactive** | `is_active` must be `false` |
| **End Time Must Be Set** | `end_time` must be greater than 0 |
| **30-Day Grace Period** | Current time must be at least 30 days (2,592,000 seconds) after `end_time` |

### Function Signature

```rust
pub fn archive_event(env: Env, event_id: String) -> Result<(), EventRegistryError>
```

### Error Codes

| Error | Condition |
|-------|-----------|
| `EventNotFound` | No event exists with the given ID |
| `EventIsActive` | Event is still active (`is_active == true`) |
| `EventNotEnded` | `end_time` is 0 or not set |
| `InvalidDeadline` | Less than 30 days have passed since `end_time` |

## Usage Example

### Scenario: Music Festival Cleanup

```rust
// 1. Event ends on June 1, 2024
let event_end_time = 1717200000; // June 1, 2024 00:00:00 UTC

// 2. Organizer deactivates event after it concludes
client.update_event_status(&event_id, &false);

// 3. Wait 30 days...
// Current time: July 2, 2024 (31 days later)

// 4. Archive the event to reclaim storage
client.archive_event(&event_id);

// 5. Event data is deleted, receipt is preserved
let receipt = client.get_organizer_receipts(&organizer);
// receipt contains: event_id, organizer_address, total_sold, archived_at
```

## Storage Savings

### Before Archival

A typical event with multiple tiers, tags, and configuration can occupy:
- **EventInfo struct**: ~2-5 KB
- **Tier data**: ~500 bytes per tier
- **Tags and categories**: ~100-500 bytes
- **Total**: ~3-10 KB per event

### After Archival

- **EventReceipt struct**: ~200-300 bytes
- **Storage reduction**: ~90-95%

For an organizer with 100 archived events:
- **Before**: 300-1000 KB
- **After**: 20-30 KB
- **Savings**: ~97%

## Best Practices

### When to Archive

✅ **Good candidates for archival:**
- One-time events that have concluded
- Past conferences, concerts, or festivals
- Events with no ongoing obligations
- Events older than 30 days past their end date

❌ **Do NOT archive:**
- Recurring events that will be reused
- Events with pending refunds or disputes
- Events still within the 30-day grace period
- Active events

### Recommended Workflow

1. **Set `end_time` when creating events**
   - Always specify an `end_time` for events that will conclude
   - This enables automatic archival eligibility

2. **Deactivate events after they conclude**
   - Call `update_event_status(event_id, false)` after the event ends
   - This prevents new ticket sales

3. **Wait for the grace period**
   - Allow 30 days for any post-event activities (refunds, disputes)
   - This protects attendees and organizers

4. **Archive to reclaim storage**
   - Call `archive_event(event_id)` after the grace period
   - Storage fees are automatically reclaimed

## Integration with Other Features

### Receipts

Archived events are accessible via:
```rust
let receipts = client.get_organizer_receipts(&organizer_address);
```

This returns all archived event receipts for an organizer, allowing historical tracking without the full event data.

### Global Counters

Archival does NOT affect:
- Global event count (total events ever created)
- Global tickets sold count (cumulative across all events)
- Organizer event count (total events per organizer)

These counters remain accurate for platform analytics.

### Private Events

Private events follow the same archival rules as public events. The `is_private` flag does not affect archival eligibility.

## Technical Details

### Storage Keys Affected

When an event is archived, the following storage keys are removed:

- `DataKey::Event(event_id)` - Main event record
- `DataKey::OrganizerEvent(organizer, event_id)` - Organizer index entry
- Entries in `DataKey::OrganizerEventShard(organizer, shard_id)` - Sharded event lists

The following storage keys are created:

- `DataKey::EventReceipt(event_id)` - Minimal receipt
- `DataKey::OrganizerReceipt(organizer, event_id)` - Receipt index entry
- Entry in `DataKey::OrganizerReceiptShard(organizer, shard_id)` - Sharded receipt lists

### Gas Costs

Archival is a relatively inexpensive operation:
- **Read**: 1 event record
- **Write**: 1 receipt record
- **Delete**: 1 event record + index updates
- **Estimated cost**: ~0.0001 XLM (varies with network conditions)

The storage fee reclamation typically exceeds the gas cost of archival.

## Security Considerations

### Authorization

Only the event organizer can archive their events. This is enforced via:
```rust
event_info.organizer_address.require_auth();
```

### Irreversibility

**Archival is permanent and cannot be undone.** Once an event is archived:
- The full event data is permanently deleted
- Only the minimal receipt remains
- The event cannot be "unarchived" or restored

### Grace Period Rationale

The 30-day grace period serves multiple purposes:

1. **Refund Window**: Allows time for attendees to request refunds
2. **Dispute Resolution**: Provides time to resolve any issues
3. **Data Access**: Ensures organizers and attendees can access event details post-event
4. **Safety Buffer**: Prevents accidental premature archival

## Testing

Comprehensive tests are provided in `test_archive_garbage_collection.rs`:

```bash
cargo test test_archive_garbage_collection --lib
```

### Test Coverage

- ✅ Successful archival after 30 days
- ✅ Rejection before 30 days
- ✅ Rejection if event is active
- ✅ Rejection if no end_time set
- ✅ Rejection if event not found
- ✅ Boundary condition (exactly 30 days)
- ✅ Archival with sold tickets
- ✅ Multiple event archival

## Future Enhancements

Potential improvements for future versions:

1. **Batch Archival**: Archive multiple events in a single transaction
2. **Automated Archival**: Trigger archival automatically after grace period
3. **Configurable Grace Period**: Allow organizers to set custom grace periods
4. **Partial Archival**: Archive specific data while retaining other parts
5. **Archive Export**: Export full event data before archival for off-chain storage

## FAQ

### Q: Can I retrieve full event data after archival?
**A:** No. Archival permanently deletes the full event data. Only the minimal receipt remains on-chain.

### Q: What happens to tickets after archival?
**A:** Ticket records in the `ticket_payment` contract are independent and are NOT affected by event archival. Attendees retain their ticket records.

### Q: Can I archive an event without an end_time?
**A:** No. Events must have `end_time` set to be eligible for archival. This ensures the 30-day grace period can be calculated.

### Q: What if I need to archive an event immediately?
**A:** The 30-day grace period is mandatory and cannot be bypassed. This protects both organizers and attendees from premature data loss.

### Q: How much storage fee will I reclaim?
**A:** The amount depends on the size of your event data. Typical events reclaim 90-95% of their storage fees, which can be significant for large events with many tiers and configuration.

### Q: Can archived events be queried?
**A:** Yes, via `get_organizer_receipts(organizer_address)`. This returns all archived event receipts for an organizer, providing basic historical data.

## Conclusion

The Event Garbage Collection feature is a critical tool for managing long-term storage costs on the Stellar/Soroban blockchain. By archiving old events, organizers can significantly reduce their storage fees while maintaining essential historical records. The 30-day grace period ensures safety and provides time for post-event activities.

For questions or issues, please refer to the main contract documentation or open an issue on the project repository.
