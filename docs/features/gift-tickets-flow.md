# Gift Tickets Feature - Flow Diagram

## Purchase Flow Comparison

### Regular Purchase (Buyer = Owner)
```
┌─────────────────────────────────────────────────────────────────┐
│                        REGULAR PURCHASE                          │
└─────────────────────────────────────────────────────────────────┘

User (Alice)
    │
    │ 1. Opens ticket modal
    ├──────────────────────────────────────────────────────────────►
    │                                                                │
    │ 2. Selects quantity: 2                                        │
    │    Gift mode: OFF                                             │
    ├──────────────────────────────────────────────────────────────►
    │                                                                │
    │ 3. Clicks "Confirm Purchase"                                  │
    ├──────────────────────────────────────────────────────────────►
    │                                                                │
    │                                                                ▼
    │                                                    POST /api/payments/ticket
    │                                                    {
    │                                                      eventId: "event-123",
    │                                                      quantity: 2,
    │                                                      buyerWallet: "Alice"
    │                                                      // No recipientWallet
    │                                                    }
    │                                                                │
    │                                                                ▼
    │                                                    ownerWallet = buyerWallet
    │                                                    ownerWallet = "Alice"
    │                                                                │
    │                                                                ▼
    │                                                    mintTicket(eventId, "Alice", 2)
    │                                                                │
    │                                                                ▼
    │                                                    Smart Contract:
    │                                                    process_payment(
    │                                                      buyer: "Alice",
    │                                                      recipient: None
    │                                                    )
    │                                                                │
    │                                                                ▼
    │                                                    owner = recipient ?? buyer
    │                                                    owner = "Alice"
    │                                                                │
    │                                                                ▼
    │                                                    increment_inventory(
    │                                                      event, tier, "Alice", 2
    │                                                    )
    │                                                                │
    │                                                                ▼
    │                                                    Database:
    │                                                    Ticket {
    │                                                      buyerWallet: "Alice",
    │                                                      ownerWallet: "Alice",
    │                                                      quantity: 2
    │                                                    }
    │                                                                │
    │ 4. Success: "Ticket purchased successfully!"                  │
    │◄──────────────────────────────────────────────────────────────┤
    │                                                                │
    ▼                                                                ▼
Alice's Wallet                                              Alice's Wallet
- Balance: -$100                                            + 2 Tickets
```

### Gift Purchase (Buyer ≠ Owner)
```
┌─────────────────────────────────────────────────────────────────┐
│                         GIFT PURCHASE                            │
└─────────────────────────────────────────────────────────────────┘

User (Alice)                                                Friend (Bob)
    │                                                           │
    │ 1. Opens ticket modal                                    │
    ├──────────────────────────────────────────────────────────►
    │                                                           │
    │ 2. Selects quantity: 2                                   │
    │    Toggles "Gift to someone?" ON                         │
    │    Enters Bob's wallet: "GBOB...XYZ"                     │
    ├──────────────────────────────────────────────────────────►
    │                                                           │
    │ 3. Clicks "Confirm Purchase"                             │
    ├──────────────────────────────────────────────────────────►
    │                                                           │
    │                                                           ▼
    │                                               POST /api/payments/ticket
    │                                               {
    │                                                 eventId: "event-123",
    │                                                 quantity: 2,
    │                                                 buyerWallet: "Alice",
    │                                                 recipientWallet: "Bob" ◄─ NEW!
    │                                               }
    │                                                           │
    │                                                           ▼
    │                                               ownerWallet = recipientWallet
    │                                               ownerWallet = "Bob"
    │                                                           │
    │                                                           ▼
    │                                               mintTicket(eventId, "Bob", 2)
    │                                                           │
    │                                                           ▼
    │                                               Smart Contract:
    │                                               process_payment(
    │                                                 buyer: "Alice",
    │                                                 recipient: Some("Bob") ◄─ NEW!
    │                                               )
    │                                                           │
    │                                                           ▼
    │                                               owner = recipient ?? buyer
    │                                               owner = "Bob"
    │                                                           │
    │                                                           ▼
    │                                               increment_inventory(
    │                                                 event, tier, "Bob", 2 ◄─ Uses Bob!
    │                                               )
    │                                                           │
    │                                                           ▼
    │                                               Database:
    │                                               Ticket {
    │                                                 buyerWallet: "Alice", ◄─ Who paid
    │                                                 ownerWallet: "Bob",   ◄─ Who owns
    │                                                 quantity: 2
    │                                               }
    │                                                           │
    │ 4. Success: "Gift sent to GBOB...XYZ!"                   │
    │◄──────────────────────────────────────────────────────────┤
    │                                                           │
    ▼                                                           ▼
Alice's Wallet                                          Bob's Wallet
- Balance: -$100                                        + 2 Tickets ✨
```

## Component Architecture

```
┌─────────────────────────────────────────────────────────────────┐
│                         Frontend Layer                           │
└─────────────────────────────────────────────────────────────────┘

TicketModal Component
├── State
│   ├── quantity: number
│   ├── isGiftMode: boolean ◄─────────────── NEW!
│   └── recipientWallet: string ◄─────────── NEW!
│
├── UI Elements
│   ├── Quantity Selector (+ / -)
│   ├── Gift Mode Toggle ◄───────────────── NEW!
│   │   └── Switch (ON/OFF)
│   ├── Recipient Input ◄───────────────── NEW!
│   │   └── Text Input (Stellar Address)
│   └── Confirm Button
│
└── Purchase Logic
    ├── Build Request Body
    │   ├── eventId
    │   ├── quantity
    │   ├── buyerWallet
    │   └── recipientWallet? ◄─────────── NEW! (conditional)
    │
    └── POST /api/payments/ticket

┌─────────────────────────────────────────────────────────────────┐
│                          API Layer                               │
└─────────────────────────────────────────────────────────────────┘

/api/payments/ticket
├── Request Body
│   ├── eventId: string
│   ├── quantity: number
│   ├── buyerWallet: string
│   └── recipientWallet?: string ◄────────── NEW! (optional)
│
├── Validation
│   ├── Validate eventId
│   ├── Validate quantity
│   ├── Validate buyerWallet
│   └── Validate recipientWallet? ◄────────── NEW!
│
├── Logic
│   ├── ownerWallet = recipientWallet || buyerWallet ◄─ NEW!
│   ├── mintTicket(eventId, ownerWallet, qty)
│   └── Database Transaction
│       ├── Update Event (increment mintedTickets)
│       └── Create Ticket
│           ├── buyerWallet ◄──────────── Who paid
│           └── ownerWallet ◄──────────── Who owns (NEW!)
│
└── Response
    ├── ticketId
    └── transactionXdr

┌─────────────────────────────────────────────────────────────────┐
│                      Smart Contract Layer                        │
└─────────────────────────────────────────────────────────────────┘

process_payment()
├── Parameters
│   ├── buyer_address: Address
│   ├── recipient_address: Option<Address> ◄─ NEW!
│   ├── event_id: String
│   ├── ticket_tier_id: String
│   ├── amount: i128
│   └── quantity: u32
│
├── Logic
│   ├── buyer_address.require_auth() ◄────── Buyer pays
│   ├── owner_address = recipient ?? buyer ◄─ NEW!
│   ├── Transfer tokens from buyer
│   ├── Calculate fees
│   └── increment_inventory(event, tier, owner_address, qty) ◄─ Uses owner!
│
└── Payment Record
    ├── buyer_address ◄──────────────────── Who paid
    ├── owner_address ◄──────────────────── Who owns (NEW!)
    ├── amount
    └── status

┌─────────────────────────────────────────────────────────────────┐
│                        Database Layer                            │
└─────────────────────────────────────────────────────────────────┘

Ticket Model
├── id: String
├── stellarId: String
├── eventId: String
├── buyerWallet: String ◄────────────────── Who paid
├── ownerWallet: String ◄────────────────── Who owns (NEW!)
├── quantity: Int
└── createdAt: DateTime

Queries
├── Find by buyer: WHERE buyerWallet = ?
├── Find by owner: WHERE ownerWallet = ? ◄─ NEW!
└── Find gifts: WHERE buyerWallet != ownerWallet ◄─ NEW!
```

## Data Flow Diagram

```
┌──────────────────────────────────────────────────────────────────┐
│                         DATA FLOW                                 │
└──────────────────────────────────────────────────────────────────┘

User Input
    │
    ├─► isGiftMode = true
    ├─► recipientWallet = "Bob"
    ├─► quantity = 2
    └─► buyerWallet = "Alice"
        │
        ▼
    ┌─────────────────────┐
    │   Frontend Logic    │
    │                     │
    │ if (isGiftMode &&   │
    │     recipientWallet)│
    │   include in body   │
    └─────────────────────┘
        │
        ▼
    ┌─────────────────────┐
    │   API Request       │
    │                     │
    │ {                   │
    │   buyerWallet: A,   │
    │   recipientWallet: B│
    │ }                   │
    └─────────────────────┘
        │
        ▼
    ┌─────────────────────┐
    │   API Logic         │
    │                     │
    │ owner = recipient   │
    │       || buyer      │
    │ owner = B           │
    └─────────────────────┘
        │
        ├──────────────────┬──────────────────┐
        ▼                  ▼                  ▼
    ┌─────────┐      ┌──────────┐      ┌──────────┐
    │ Stellar │      │ Contract │      │ Database │
    │         │      │          │      │          │
    │ Mint to │      │ owner=B  │      │ buyer=A  │
    │ Bob     │      │ buyer=A  │      │ owner=B  │
    └─────────┘      └──────────┘      └──────────┘
        │                  │                  │
        └──────────────────┴──────────────────┘
                           │
                           ▼
                    ┌──────────────┐
                    │   Result     │
                    │              │
                    │ Alice paid   │
                    │ Bob owns     │
                    └──────────────┘
```

## State Transitions

```
┌──────────────────────────────────────────────────────────────────┐
│                      GIFT MODE STATES                             │
└──────────────────────────────────────────────────────────────────┘

Initial State
    │
    ├─► isGiftMode = false
    ├─► recipientWallet = ""
    └─► UI: Toggle OFF, Input HIDDEN
        │
        │ User clicks toggle
        ▼
Gift Mode Enabled
    │
    ├─► isGiftMode = true
    ├─► recipientWallet = ""
    └─► UI: Toggle ON, Input VISIBLE
        │
        │ User enters wallet
        ▼
Recipient Entered
    │
    ├─► isGiftMode = true
    ├─► recipientWallet = "GBOB...XYZ"
    └─► UI: Toggle ON, Input FILLED
        │
        │ User clicks purchase
        ▼
Purchase Initiated
    │
    ├─► Validate recipient wallet
    ├─► Build request with recipientWallet
    └─► POST to API
        │
        │ API processes
        ▼
Purchase Complete
    │
    ├─► Show success message
    ├─► Display recipient address
    └─► Show QR code
        │
        │ User clicks done
        ▼
Modal Closed
    │
    └─► Reset state
```

## Error Handling Flow

```
┌──────────────────────────────────────────────────────────────────┐
│                      ERROR SCENARIOS                              │
└──────────────────────────────────────────────────────────────────┘

Purchase Request
    │
    ├─► Validate recipientWallet
    │   │
    │   ├─► Empty? ──────────────► Use buyerWallet (OK)
    │   │
    │   ├─► Invalid format? ─────► Error: "Invalid wallet address"
    │   │
    │   └─► Valid? ───────────────► Continue
    │
    ├─► Check per-user limit
    │   │
    │   ├─► Owner at limit? ─────► Error: "Recipient limit exceeded"
    │   │
    │   └─► Under limit? ─────────► Continue
    │
    ├─► Process payment
    │   │
    │   ├─► Insufficient funds? ──► Error: "Insufficient balance"
    │   │
    │   ├─► Event sold out? ──────► Error: "No tickets available"
    │   │
    │   └─► Success? ──────────────► Continue
    │
    └─► Create ticket
        │
        ├─► Database error? ──────► Rollback, Error: "Failed to create"
        │
        └─► Success ──────────────► Return ticketId
```

## Inventory Tracking Flow

```
┌──────────────────────────────────────────────────────────────────┐
│                   INVENTORY TRACKING                              │
└──────────────────────────────────────────────────────────────────┘

Regular Purchase (Alice buys for herself)
    │
    ├─► buyer = Alice
    ├─► recipient = None
    ├─► owner = Alice
    │
    └─► increment_inventory(event, tier, Alice, 2)
        │
        └─► Alice's ticket count += 2

Gift Purchase (Alice buys for Bob)
    │
    ├─► buyer = Alice
    ├─► recipient = Some(Bob)
    ├─► owner = Bob ◄──────────────────────── KEY DIFFERENCE!
    │
    └─► increment_inventory(event, tier, Bob, 2)
        │
        ├─► Bob's ticket count += 2 ◄────── Bob's limit checked
        └─► Alice's ticket count unchanged ◄─ Alice can buy more gifts!

Per-User Limit Check
    │
    ├─► Get current count for OWNER (not buyer)
    ├─► Check: current + quantity <= max_per_user
    │
    ├─► If exceeded ──────────────► Error: "PerUserLimitExceeded"
    └─► If OK ────────────────────► Continue
```

## Summary

### Key Points
1. **Buyer pays, recipient owns** - Core concept
2. **Optional parameter** - Backward compatible
3. **Inventory uses owner** - Limits apply to recipient
4. **Database tracks both** - Enables gift queries
5. **UI toggle** - Simple, intuitive interface

### Benefits
- ✅ Gift tickets to friends/family
- ✅ Recipient sees ticket immediately
- ✅ Buyer's purchase history preserved
- ✅ Flexible per-user limits
- ✅ Backward compatible

### Use Cases
- 🎁 Birthday gifts
- 👨‍👩‍👧‍👦 Family events
- 🎉 Group purchases
- 💝 Corporate gifting
- 🎓 Educational events
