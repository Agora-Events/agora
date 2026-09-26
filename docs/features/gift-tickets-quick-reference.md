# Gift Tickets Feature - Quick Reference

## TL;DR
Users can now buy tickets for friends/family. Buyer pays, recipient owns the ticket.

## API Usage

### Regular Purchase (Buyer = Owner)
```bash
POST /api/payments/ticket
Content-Type: application/json

{
  "eventId": "event-123",
  "quantity": 2,
  "buyerWallet": "GABC...XYZ"
}
```

### Gift Purchase (Buyer ≠ Owner)
```bash
POST /api/payments/ticket
Content-Type: application/json

{
  "eventId": "event-123",
  "quantity": 2,
  "buyerWallet": "GABC...XYZ",
  "recipientWallet": "GDEF...UVW"  # NEW: Optional recipient
}
```

## Smart Contract Usage

### Function Signature
```rust
pub fn process_payment(
    env: Env,
    payment_id: String,
    event_id: String,
    ticket_tier_id: String,
    buyer_address: Address,
    recipient_address: Option<Address>,  // NEW: Optional recipient
    token_address: Address,
    amount: i128,
    quantity: u32,
    options: PurchaseOptions,
    validation_hash: BytesN<32>,
) -> Result<String, TicketPaymentError>
```

### Example Call
```rust
// Regular purchase
process_payment(env, payment_id, event_id, tier_id, buyer, None, token, amount, qty, options, hash)

// Gift purchase
process_payment(env, payment_id, event_id, tier_id, buyer, Some(recipient), token, amount, qty, options, hash)
```

## Database Queries

### Find tickets bought by user
```sql
SELECT * FROM "Ticket" WHERE "buyerWallet" = 'GABC...XYZ';
```

### Find tickets owned by user
```sql
SELECT * FROM "Ticket" WHERE "ownerWallet" = 'GABC...XYZ';
```

### Find all gift tickets
```sql
SELECT * FROM "Ticket" WHERE "buyerWallet" != "ownerWallet";
```

### Find gifts sent by user
```sql
SELECT * FROM "Ticket" 
WHERE "buyerWallet" = 'GABC...XYZ' 
AND "ownerWallet" != "buyerWallet";
```

### Find gifts received by user
```sql
SELECT * FROM "Ticket" 
WHERE "ownerWallet" = 'GABC...XYZ' 
AND "buyerWallet" != "ownerWallet";
```

## Prisma Queries

### Find tickets bought by user
```typescript
const tickets = await prisma.ticket.findMany({
  where: { buyerWallet: 'GABC...XYZ' }
});
```

### Find tickets owned by user
```typescript
const tickets = await prisma.ticket.findMany({
  where: { ownerWallet: 'GABC...XYZ' }
});
```

### Find gift tickets
```typescript
const giftTickets = await prisma.ticket.findMany({
  where: {
    NOT: {
      buyerWallet: { equals: prisma.ticket.fields.ownerWallet }
    }
  }
});
```

### Create ticket with gift
```typescript
await prisma.ticket.create({
  data: {
    stellarId: 'ticket-123',
    eventId: 'event-456',
    buyerWallet: 'GABC...XYZ',
    ownerWallet: 'GDEF...UVW',  // Recipient
    quantity: 2,
  }
});
```

## Frontend Usage

### Component State
```typescript
const [recipientWallet, setRecipientWallet] = useState<string>("");
const [isGiftMode, setIsGiftMode] = useState(false);
```

### Toggle Gift Mode
```typescript
<button onClick={() => setIsGiftMode(!isGiftMode)}>
  Gift Mode
</button>
```

### Recipient Input
```typescript
{isGiftMode && (
  <input
    type="text"
    value={recipientWallet}
    onChange={(e) => setRecipientWallet(e.target.value)}
    placeholder="G... (Stellar address)"
  />
)}
```

### Purchase Request
```typescript
const requestBody = {
  eventId: event.id,
  quantity: quantity,
  buyerWallet: currentUserWallet,
};

if (isGiftMode && recipientWallet.trim()) {
  requestBody.recipientWallet = recipientWallet.trim();
}

await fetch('/api/payments/ticket', {
  method: 'POST',
  body: JSON.stringify(requestBody),
});
```

## Key Concepts

### Buyer vs. Owner
- **Buyer**: Person who pays for the ticket
- **Owner**: Person who receives/owns the ticket
- For regular purchases: Buyer = Owner
- For gift purchases: Buyer ≠ Owner

### Inventory Tracking
- Uses `owner_address` (recipient), not `buyer_address`
- Per-user limits apply to owner
- Loyalty points awarded to buyer

### Payment Flow
1. Buyer authenticates and pays
2. Funds deducted from buyer's wallet
3. Ticket minted to owner's wallet
4. Database records both buyer and owner
5. Inventory incremented for owner

### Refund Flow
- Refunds go back to buyer (who paid)
- Inventory decremented for owner
- Ticket removed from owner's wallet

## Common Patterns

### Check if ticket is a gift
```typescript
const isGift = ticket.buyerWallet !== ticket.ownerWallet;
```

### Get gift sender
```typescript
const sender = ticket.buyerWallet;
```

### Get gift recipient
```typescript
const recipient = ticket.ownerWallet;
```

### Display gift info
```typescript
if (isGift) {
  return `Gift from ${ticket.buyerWallet.slice(0, 8)}...`;
} else {
  return `Purchased by you`;
}
```

## Validation Rules

### Recipient Wallet
- Must be a valid Stellar address (starts with 'G')
- Cannot be empty if gift mode is enabled
- Defaults to buyer wallet if not provided
- No self-gifting restrictions (buyer can gift to self)

### Purchase Limits
- Per-user limits apply to **owner** (recipient)
- Buyer can purchase unlimited gifts (subject to event limits)
- Inventory tracking uses owner address

## Error Handling

### Invalid Recipient Wallet
```typescript
if (recipientWallet && !isValidStellarAddress(recipientWallet)) {
  throw new Error('Invalid recipientWallet');
}
```

### Missing Buyer Wallet
```typescript
if (!buyerWallet) {
  throw new Error('Invalid buyerWallet');
}
```

### Per-User Limit Exceeded
```rust
if new_user_count > tier.max_per_user {
    return Err(EventRegistryError::PerUserLimitExceeded);
}
```

## Testing Commands

### Run tests
```bash
npm test apps/web/__tests__/gift-tickets.test.ts
```

### Test API endpoint
```bash
curl -X POST http://localhost:3000/api/payments/ticket \
  -H "Content-Type: application/json" \
  -d '{
    "eventId": "event-123",
    "quantity": 1,
    "buyerWallet": "GABC...XYZ",
    "recipientWallet": "GDEF...UVW"
  }'
```

### Test database
```bash
npx prisma studio
```

## Migration Commands

### Generate migration
```bash
cd apps/web
npx prisma migrate dev --name add_owner_wallet_to_tickets
```

### Apply migration
```bash
npx prisma migrate deploy
```

### Backfill existing data
```bash
npx ts-node scripts/backfill-owner-wallet.ts
```

## Deployment Commands

### Build contract
```bash
cd contract
cargo build --release --target wasm32-unknown-unknown
```

### Deploy contract
```bash
stellar contract deploy \
  --wasm target/wasm32-unknown-unknown/release/ticket_payment.wasm \
  --network testnet
```

### Build frontend
```bash
cd apps/web
npm run build
```

### Start production server
```bash
npm run start
```

## Monitoring Queries

### Count gift tickets
```sql
SELECT COUNT(*) FROM "Ticket" WHERE "buyerWallet" != "ownerWallet";
```

### Top gift senders
```sql
SELECT "buyerWallet", COUNT(*) as gifts_sent
FROM "Ticket"
WHERE "buyerWallet" != "ownerWallet"
GROUP BY "buyerWallet"
ORDER BY gifts_sent DESC
LIMIT 10;
```

### Top gift recipients
```sql
SELECT "ownerWallet", COUNT(*) as gifts_received
FROM "Ticket"
WHERE "buyerWallet" != "ownerWallet"
GROUP BY "ownerWallet"
ORDER BY gifts_received DESC
LIMIT 10;
```

## Troubleshooting

### Ticket not showing in recipient's wallet
- Check `ownerWallet` field in database
- Verify `increment_inventory` used owner address
- Check Stellar transaction status

### Payment charged to wrong wallet
- Verify `buyer_address.require_auth()` is called
- Check payment deduction logic
- Review transaction logs

### Per-user limit not working
- Verify inventory tracking uses owner address
- Check `get_user_ticket_count` implementation
- Review event tier configuration

## Resources

- Full Documentation: `GIFT_TICKETS_FEATURE.md`
- Migration Guide: `apps/web/MIGRATION_GUIDE.md`
- Test Suite: `apps/web/__tests__/gift-tickets.test.ts`
- Implementation Summary: `IMPLEMENTATION_SUMMARY.md`
