# Event Archive Flow Diagram

## Overview Flow

```
┌─────────────────────────────────────────────────────────────────┐
│                     Event Lifecycle                              │
└─────────────────────────────────────────────────────────────────┘

1. Event Creation
   ↓
   [Event Registered]
   - Full EventInfo stored (~3-10 KB)
   - end_time set
   - is_active = true
   
2. Event Active Period
   ↓
   [Ticket Sales]
   - Tickets sold
   - Inventory tracked
   - Revenue collected
   
3. Event Concludes
   ↓
   [Event Ends]
   - end_time reached
   - Organizer deactivates: is_active = false
   
4. Grace Period (30 days)
   ↓
   [Waiting Period]
   - Refunds processed
   - Disputes resolved
   - Data accessible
   
5. Archive Eligible
   ↓
   [Archive Event]
   - Full data deleted
   - Receipt created (~200-300 bytes)
   - Storage fees reclaimed
```

## Archive Function Flow

```
┌─────────────────────────────────────────────────────────────────┐
│                  archive_event(event_id)                         │
└─────────────────────────────────────────────────────────────────┘

START
  │
  ├─► Check: Event exists?
  │   ├─ NO  → Return EventNotFound ❌
  │   └─ YES → Continue
  │
  ├─► Check: Organizer authorized?
  │   ├─ NO  → Return Unauthorized ❌
  │   └─ YES → Continue
  │
  ├─► Check: Event is inactive?
  │   ├─ NO  → Return EventIsActive ❌
  │   └─ YES → Continue
  │
  ├─► Check: end_time is set?
  │   ├─ NO  → Return EventNotEnded ❌
  │   └─ YES → Continue
  │
  ├─► Check: 30 days passed since end_time?
  │   ├─ NO  → Return InvalidDeadline ❌
  │   └─ YES → Continue
  │
  ├─► Delete full EventInfo
  │   └─ Storage reclaimed ✅
  │
  ├─► Create EventReceipt
  │   └─ Minimal data preserved ✅
  │
  ├─► Emit EventArchived event
  │   └─ Blockchain audit trail ✅
  │
  └─► Return Success ✅

END
```

## Storage Transformation

```
┌─────────────────────────────────────────────────────────────────┐
│                    Before Archive                                │
└─────────────────────────────────────────────────────────────────┘

EventInfo {
  event_id: "summer-fest-2024"
  name: "Summer Music Festival"
  organizer_address: GABC...
  payment_address: GXYZ...
  platform_fee_percent: 500
  is_active: false
  status: Active
  created_at: 1704067200
  metadata_cid: "QmABC..."
  max_supply: 1000
  current_supply: 850
  milestone_plan: Some([...])
  tiers: Map {
    "general": TicketTier { ... }
    "vip": TicketTier { ... }
    "early-bird": TicketTier { ... }
  }
  refund_deadline: 1717200000
  restocking_fee: 50_0000000
  resale_cap_bps: Some(1000)
  is_postponed: false
  grace_period_end: 0
  min_sales_target: 500
  target_deadline: 1716000000
  goal_met: true
  custom_fee_bps: None
  banner_cid: Some("QmBanner...")
  tags: Some(["music", "festival", "outdoor"])
  category_ids: Some([1, 2, 9])
  start_time: 1716595200
  is_private: false
  end_time: 1717200000
  transfer_lock_duration: 86400
  accepted_tokens: [...]
  use_global_whitelist: true
  feedback_cid: None
  cancellation_reason: None
  referral_rate_bps: 500
}

Size: ~3-10 KB
Storage Fee: HIGH 💰💰💰

                    ↓ ARCHIVE ↓

┌─────────────────────────────────────────────────────────────────┐
│                     After Archive                                │
└─────────────────────────────────────────────────────────────────┘

EventReceipt {
  event_id: "summer-fest-2024"
  organizer_address: GABC...
  total_sold: 850
  archived_at: 1719792000
}

Size: ~200-300 bytes
Storage Fee: MINIMAL 💰

Savings: 90-95% ✅
```

## Timeline Example

```
┌─────────────────────────────────────────────────────────────────┐
│                    Timeline Example                              │
└─────────────────────────────────────────────────────────────────┘

June 1, 2024 00:00:00 UTC
│ end_time = 1717200000
│ Event concludes
│
├─► Organizer deactivates event
│   is_active = false
│
│ ◄─── 30 Day Grace Period ───►
│
│ Day 1-29: Cannot archive ❌
│ - Refunds processed
│ - Disputes resolved
│ - Data accessible
│
July 1, 2024 00:00:00 UTC
│ end_time + 30 days = 1719792000
│
├─► Archive eligible ✅
│   Can call archive_event()
│
July 2, 2024 10:30:00 UTC
│ Organizer archives event
│
├─► Storage reclaimed ✅
│   - Full data deleted
│   - Receipt created
│   - Fees returned
│
└─► Complete ✅
```

## Error Scenarios

```
┌─────────────────────────────────────────────────────────────────┐
│                    Error Scenarios                               │
└─────────────────────────────────────────────────────────────────┘

Scenario 1: Too Early
─────────────────────
Event ends: June 1, 2024
Current:    June 15, 2024 (15 days later)
Result:     InvalidDeadline ❌
Reason:     Only 15 days passed, need 30

Scenario 2: Event Still Active
───────────────────────────────
Event ends: June 1, 2024
Current:    July 5, 2024 (34 days later)
is_active:  true
Result:     EventIsActive ❌
Reason:     Must deactivate first

Scenario 3: No End Time
───────────────────────
Event ends: Not set (end_time = 0)
Current:    Any time
Result:     EventNotEnded ❌
Reason:     Cannot calculate grace period

Scenario 4: Success
───────────────────
Event ends: June 1, 2024
Current:    July 2, 2024 (31 days later)
is_active:  false
Result:     Success ✅
Action:     Event archived
```

## Storage Reclamation Process

```
┌─────────────────────────────────────────────────────────────────┐
│              Storage Reclamation Process                         │
└─────────────────────────────────────────────────────────────────┘

1. Before Archive
   ┌──────────────────────────────────┐
   │  Soroban Persistent Storage      │
   ├──────────────────────────────────┤
   │  EventInfo (3-10 KB)             │
   │  - Tiers                         │
   │  - Milestones                    │
   │  - Tags                          │
   │  - Categories                    │
   │  - Configuration                 │
   └──────────────────────────────────┘
   Storage Rent: HIGH 💰💰💰

2. Archive Called
   ┌──────────────────────────────────┐
   │  storage::remove_event()         │
   └──────────────────────────────────┘
   ↓
   Soroban deletes persistent data
   ↓
   Storage deposit returned to organizer ✅

3. After Archive
   ┌──────────────────────────────────┐
   │  Soroban Persistent Storage      │
   ├──────────────────────────────────┤
   │  EventReceipt (200-300 bytes)    │
   │  - event_id                      │
   │  - organizer_address             │
   │  - total_sold                    │
   │  - archived_at                   │
   └──────────────────────────────────┘
   Storage Rent: MINIMAL 💰

4. Result
   ✅ 90-95% storage reduction
   ✅ Storage fees reclaimed
   ✅ Ongoing rent reduced
   ✅ Historical data preserved
```

## Multi-Event Archival

```
┌─────────────────────────────────────────────────────────────────┐
│              Organizer with Multiple Events                      │
└─────────────────────────────────────────────────────────────────┘

Organizer: GABC...

Events:
├─ Event 1: "Spring Concert"
│  Status: Archived ✅
│  Storage: 250 bytes (was 5 KB)
│
├─ Event 2: "Summer Festival"
│  Status: Archived ✅
│  Storage: 280 bytes (was 8 KB)
│
├─ Event 3: "Fall Gala"
│  Status: Archived ✅
│  Storage: 230 bytes (was 4 KB)
│
└─ Event 4: "Winter Ball"
   Status: Active 🔴
   Storage: 6 KB (not eligible for archive)

Total Storage:
- Before: 23 KB (5+8+4+6)
- After:  6.76 KB (0.25+0.28+0.23+6)
- Savings: 70.6% overall
- Savings (archived only): 96.5%

Receipts Available:
- get_organizer_receipts(GABC...) returns 3 receipts
- Historical data preserved
- Minimal storage footprint
```

## Integration with Platform

```
┌─────────────────────────────────────────────────────────────────┐
│              Platform Integration                                │
└─────────────────────────────────────────────────────────────────┘

Frontend (Next.js)
  │
  ├─► Display "Archive" button
  │   - Only if event eligible
  │   - Show days remaining
  │
  ├─► Call archive_event()
  │   - Handle errors gracefully
  │   - Show success message
  │
  └─► Display receipts
      - List archived events
      - Show basic stats

Backend (Rust/Axum)
  │
  ├─► Monitor events
  │   - Track eligible events
  │   - Send notifications
  │
  └─► Analytics
      - Storage savings
      - Archive trends

Smart Contract (Soroban)
  │
  ├─► Validate archival
  │   - Check all requirements
  │   - Enforce grace period
  │
  ├─► Delete data
  │   - Remove EventInfo
  │   - Reclaim storage
  │
  └─► Create receipt
      - Preserve history
      - Emit event
```

## Legend

```
✅ Success / Completed
❌ Error / Rejected
🔴 Active / In Progress
💰 Cost / Fee
→  Flow direction
├─ Branch point
└─ End point
```
