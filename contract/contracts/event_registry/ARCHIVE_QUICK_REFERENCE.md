# Archive Event - Quick Reference

## Function Signature

```rust
pub fn archive_event(env: Env, event_id: String) -> Result<(), EventRegistryError>
```

## Requirements Checklist

Before calling `archive_event`, ensure:

- [ ] You are the event organizer
- [ ] Event `is_active` is `false`
- [ ] Event `end_time` is set (> 0)
- [ ] At least 30 days have passed since `end_time`

## Quick Usage

```rust
// 1. Deactivate event after it ends
client.update_event_status(&event_id, &false);

// 2. Wait 30 days after end_time

// 3. Archive the event
client.archive_event(&event_id)?;

// 4. Access receipt
let receipts = client.get_organizer_receipts(&organizer);
```

## Error Codes

| Error | Reason | Solution |
|-------|--------|----------|
| `EventNotFound` | Event doesn't exist | Check event_id |
| `EventIsActive` | Event still active | Call `update_event_status(false)` first |
| `EventNotEnded` | No end_time set | Cannot archive events without end_time |
| `InvalidDeadline` | < 30 days since end_time | Wait until 30 days have passed |

## What Gets Deleted

- ✅ Ticket tiers
- ✅ Milestone plans
- ✅ Tags and categories
- ✅ Banner and metadata CIDs
- ✅ All event configuration

## What Gets Preserved

- ✅ Event ID
- ✅ Organizer address
- ✅ Total tickets sold
- ✅ Archive timestamp

## Storage Savings

- **Before:** 3-10 KB per event
- **After:** 200-300 bytes per event
- **Reduction:** ~90-95%

## Important Notes

⚠️ **Archival is permanent and cannot be undone**

⚠️ **30-day grace period is mandatory**

⚠️ **Tickets in ticket_payment contract are NOT affected**

✅ **Storage fees are automatically reclaimed**

✅ **Receipts can be queried via `get_organizer_receipts()`**

## Testing

```bash
cargo test test_archive_garbage_collection --lib
```

## Full Documentation

See `GARBAGE_COLLECTION.md` for complete documentation.
