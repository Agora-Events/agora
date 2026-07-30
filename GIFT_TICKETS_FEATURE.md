# Gift Tickets Feature - Buy Tickets for Friends or Family

## Overview
This feature allows users to purchase event tickets for friends or family members by specifying a recipient wallet address at the time of purchase. The buyer pays for the ticket, but the ticket ownership is transferred to the recipient's wallet immediately after the purchase is confirmed.

## Implementation Details

### Smart Contract Changes (Soroban/Rust)

#### 1. Updated Payment Struct (`contract/contracts/ticket_payment/src/types.rs`)
Added `owner_address` field to track the actual ticket owner (recipient):

```rust
pub struct Payment {
    pub payment_id: String,
    pub event_id: String,
    pub buyer_address: Address,      // Who paid for the ticket
    pub owner_address: Address,       // Who owns the ticket (recipient)
    pub ticket_tier_id: String,
    // ... other fields
}
```

#### 2. Updated `process_payment` Function (`contract/contracts/ticket_payment/src/contract.rs`)
- Added `recipient_address: Option<Address>` parameter
- If `recipient_address` is `None`, the buyer is also the owner
- If `recipient_address` is provided, the ticket is owned by the recipient
- The `owner_address` is used for inventory tracking instead of `buyer_address`

**Function Signature:**
```rust
pub fn process_payment(
    env: Env,
    payment_id: String,
    event_id: String,
    ticket_tier_id: String,
    buyer_address: Address,
    recipient_address: Option<Address>, // NEW: Optional recipient
    token_address: Address,
    amount: i128,
    quantity: u32,
    options: PurchaseOptions,
    validation_hash: BytesN<32>,
) -> Result<String, TicketPaymentError>
```

**Key Logic:**
```rust
// Determine the actual owner of the ticket (recipient or buyer)
let owner_address = recipient_address.unwrap_or_else(|| buyer_address.clone());

// Use owner_address for inventory tracking
registry_client.increment_inventory(&event_id, &ticket_tier_id, &owner_address, &quantity);
```

### Database Schema Changes (`apps/web/prisma/schema.prisma`)

Updated the `Ticket` model to track both buyer and owner:

```prisma
model Ticket {
  id          String   @id @default(uuid())
  stellarId   String?  @unique
  eventId     String
  event       Event    @relation(fields: [eventId], references: [id])
  buyerWallet String   // The person who paid for the ticket
  ownerWallet String   // The person who owns/receives the ticket
  quantity    Int      @default(1)
  createdAt   DateTime @default(now())
}
```

**Migration Required:**
```bash
cd apps/web
npx prisma migrate dev --name add_owner_wallet_to_tickets
```

### Backend API Changes (`apps/web/app/api/payments/ticket/route.ts`)

#### Updated Request Body Type:
```typescript
type TicketRequestBody = {
  eventId?: string;
  quantity?: number;
  buyerWallet?: string;
  recipientWallet?: string; // NEW: Optional recipient wallet
};
```

#### Updated Logic:
```typescript
// Validate recipientWallet if provided
if (recipientWallet !== undefined && recipientWallet !== null && typeof recipientWallet !== "string") {
  throwApiError("Invalid recipientWallet", 400);
}

// Determine the actual owner of the ticket
const ownerWallet = recipientWallet || buyerWallet;

// Use ownerWallet for minting
const mintResult = await mintTicket(eventId, ownerWallet, qty);

// Store both buyer and owner in database
await prisma.ticket.create({
  data: {
    stellarId: mintResult.ticketId,
    eventId,
    buyerWallet,
    ownerWallet,
    quantity: qty,
  },
});
```

### Frontend Changes (`apps/web/components/events/TicketModal.tsx`)

#### New State Variables:
```typescript
const [recipientWallet, setRecipientWallet] = useState<string>("");
const [isGiftMode, setIsGiftMode] = useState(false);
```

#### New UI Components:
1. **Gift Mode Toggle** - A toggle switch to enable/disable gift mode
2. **Recipient Wallet Input** - Text input for entering the recipient's Stellar wallet address
3. **Enhanced Success Message** - Shows different messages for gift vs. regular purchases

#### Gift Mode UI:
```tsx
{/* Gift Mode Toggle */}
<div className="flex justify-between items-center">
  <div className="flex items-center gap-2">
    <Gift size={20} className="text-black/70" />
    <span className="text-lg font-bold text-black">Gift to someone?</span>
  </div>
  <button onClick={() => setIsGiftMode(!isGiftMode)}>
    {/* Toggle switch UI */}
  </button>
</div>

{/* Recipient Wallet Input (shown when gift mode is enabled) */}
{isGiftMode && (
  <div className="flex flex-col gap-2">
    <label>Recipient Wallet Address</label>
    <input
      type="text"
      value={recipientWallet}
      onChange={(e) => setRecipientWallet(e.target.value)}
      placeholder="G... (Stellar address)"
    />
    <p>The ticket will be sent to this wallet address</p>
  </div>
)}
```

#### Updated Purchase Logic:
```typescript
const requestBody = {
  eventId: event.id.toString(),
  quantity: quantity,
  buyerWallet: "G...MOCK_WALLET_ADDRESS",
};

// Only include recipientWallet if gift mode is enabled
if (isGiftMode && recipientWallet.trim()) {
  requestBody.recipientWallet = recipientWallet.trim();
}
```

## Acceptance Criteria

✅ **The buyer can specify a recipient address at purchase time**
- Gift mode toggle in the ticket purchase modal
- Recipient wallet address input field
- Validation of recipient wallet address

✅ **The buyer pays, but the ticket is owned by the recipient**
- `buyer_address` field tracks who paid
- `owner_address` field tracks who owns the ticket
- Payment is deducted from buyer's wallet
- Ticket appears in recipient's wallet

✅ **The friend's wallet shows the ticket immediately after purchase**
- `increment_inventory` uses `owner_address` (recipient)
- Database stores `ownerWallet` as the recipient
- Smart contract creates ticket with `owner = recipient`

## User Flow

### Regular Purchase (No Gift)
1. User clicks "Register" on event page
2. Opens ticket modal, selects quantity
3. Clicks "Confirm Purchase"
4. Ticket is minted to buyer's wallet
5. Success message: "Ticket purchased successfully!"

### Gift Purchase
1. User clicks "Register" on event page
2. Opens ticket modal, selects quantity
3. **Toggles "Gift to someone?" switch**
4. **Enters recipient's Stellar wallet address**
5. Clicks "Confirm Purchase"
6. Buyer pays, but ticket is minted to recipient's wallet
7. Success message: "Ticket purchased as a gift! The recipient will see it in their wallet."
8. Success screen shows: "Your gift ticket has been sent to G...XXXX"

## Security Considerations

1. **Buyer Authentication**: Only the buyer needs to authenticate (`buyer_address.require_auth()`)
2. **Recipient Validation**: Recipient address is validated as a proper Stellar address
3. **Payment Security**: Payment is always deducted from the buyer's wallet
4. **Ownership Transfer**: Ticket ownership is atomically transferred to recipient during minting
5. **Refund Handling**: Refunds should go back to the buyer (who paid), not the recipient

## Testing Checklist

- [ ] Purchase ticket without gift mode (buyer = owner)
- [ ] Purchase ticket with gift mode (buyer ≠ owner)
- [ ] Verify ticket appears in recipient's wallet
- [ ] Verify buyer's wallet is charged
- [ ] Verify database stores both buyer and owner correctly
- [ ] Test with invalid recipient wallet address
- [ ] Test with empty recipient wallet address (should default to buyer)
- [ ] Test multiple ticket purchase as gift
- [ ] Verify inventory tracking uses owner address
- [ ] Verify per-user limits apply to owner, not buyer

## Future Enhancements

1. **Gift Message**: Allow buyer to include a personal message with the gift
2. **Gift Notification**: Send notification to recipient when they receive a gift ticket
3. **Gift History**: Track gift purchases in buyer's transaction history
4. **Gift Wrapping**: Special UI treatment for gifted tickets in recipient's wallet
5. **Gift Redemption**: Allow recipient to accept or decline gift tickets
6. **Bulk Gifting**: Purchase multiple tickets for different recipients in one transaction

## API Examples

### Regular Purchase
```bash
POST /api/payments/ticket
{
  "eventId": "event-123",
  "quantity": 2,
  "buyerWallet": "GABC...XYZ"
}
```

### Gift Purchase
```bash
POST /api/payments/ticket
{
  "eventId": "event-123",
  "quantity": 2,
  "buyerWallet": "GABC...XYZ",
  "recipientWallet": "GDEF...UVW"
}
```

## Database Queries

### Find all tickets bought by a user
```sql
SELECT * FROM "Ticket" WHERE "buyerWallet" = 'GABC...XYZ';
```

### Find all tickets owned by a user
```sql
SELECT * FROM "Ticket" WHERE "ownerWallet" = 'GABC...XYZ';
```

### Find all gift tickets (buyer ≠ owner)
```sql
SELECT * FROM "Ticket" WHERE "buyerWallet" != "ownerWallet";
```

## Notes

- The feature is backward compatible: if `recipientWallet` is not provided, it defaults to `buyerWallet`
- The smart contract uses `Option<Address>` for the recipient parameter, making it optional
- The UI gracefully handles both gift and non-gift purchases
- The database migration adds the `ownerWallet` field to existing tickets (will need to backfill with `buyerWallet` values)
