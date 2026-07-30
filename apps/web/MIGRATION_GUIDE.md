# Database Migration Guide - Gift Tickets Feature

## Overview
This guide explains how to migrate the database to support the gift tickets feature, which adds an `ownerWallet` field to the `Ticket` model.

## Changes Required

### Schema Changes
The `Ticket` model now includes:
- `buyerWallet` - The person who paid for the ticket
- `ownerWallet` - The person who owns/receives the ticket (NEW)

## Migration Steps

### Step 1: Generate Migration
```bash
cd apps/web
npx prisma migrate dev --name add_owner_wallet_to_tickets
```

This will:
1. Create a new migration file in `prisma/migrations/`
2. Add the `ownerWallet` column to the `Ticket` table
3. Apply the migration to your development database

### Step 2: Backfill Existing Data
For existing tickets where `ownerWallet` is not set, we need to backfill with `buyerWallet` values (since before this feature, buyer and owner were the same person).

**Option A: Using Prisma Client (Recommended)**
```typescript
// scripts/backfill-owner-wallet.ts
import { PrismaClient } from '@prisma/client';

const prisma = new PrismaClient();

async function backfillOwnerWallet() {
  const tickets = await prisma.ticket.findMany({
    where: {
      ownerWallet: null, // or use a different condition
    },
  });

  console.log(`Found ${tickets.length} tickets to backfill`);

  for (const ticket of tickets) {
    await prisma.ticket.update({
      where: { id: ticket.id },
      data: { ownerWallet: ticket.buyerWallet },
    });
  }

  console.log('Backfill complete!');
}

backfillOwnerWallet()
  .catch(console.error)
  .finally(() => prisma.$disconnect());
```

Run the script:
```bash
npx ts-node scripts/backfill-owner-wallet.ts
```

**Option B: Using Raw SQL**
```sql
-- Update all tickets where ownerWallet is NULL
UPDATE "Ticket"
SET "ownerWallet" = "buyerWallet"
WHERE "ownerWallet" IS NULL;
```

### Step 3: Make ownerWallet Required (Optional)
If you want to make `ownerWallet` a required field after backfilling:

1. Update the schema:
```prisma
model Ticket {
  id          String   @id @default(uuid())
  stellarId   String?  @unique
  eventId     String
  event       Event    @relation(fields: [eventId], references: [id])
  buyerWallet String
  ownerWallet String   // Remove the ? to make it required
  quantity    Int      @default(1)
  createdAt   DateTime @default(now())
}
```

2. Generate and apply the migration:
```bash
npx prisma migrate dev --name make_owner_wallet_required
```

### Step 4: Deploy to Production

#### For Production Database:
```bash
# 1. Generate migration (already done in dev)
# 2. Apply migration to production
npx prisma migrate deploy

# 3. Run backfill script against production
NODE_ENV=production npx ts-node scripts/backfill-owner-wallet.ts
```

#### Using Prisma Cloud/Hosted Database:
```bash
# Set production database URL
export DATABASE_URL="postgresql://user:password@host:5432/production_db"

# Apply migrations
npx prisma migrate deploy

# Run backfill
npx ts-node scripts/backfill-owner-wallet.ts
```

## Rollback Plan

If you need to rollback the migration:

### Step 1: Revert Schema Changes
```prisma
model Ticket {
  id          String   @id @default(uuid())
  stellarId   String?  @unique
  eventId     String
  event       Event    @relation(fields: [eventId], references: [id])
  buyerWallet String
  // Remove ownerWallet field
  quantity    Int      @default(1)
  createdAt   DateTime @default(now())
}
```

### Step 2: Create Rollback Migration
```bash
npx prisma migrate dev --name remove_owner_wallet
```

### Step 3: Update Application Code
Revert all code changes that reference `ownerWallet`:
- `apps/web/app/api/payments/ticket/route.ts`
- `apps/web/components/events/TicketModal.tsx`
- Smart contract changes

## Testing the Migration

### Test in Development
```bash
# 1. Reset database
npx prisma migrate reset

# 2. Apply all migrations
npx prisma migrate dev

# 3. Seed test data
npx prisma db seed

# 4. Test the application
npm run dev
```

### Verify Data Integrity
```sql
-- Check that all tickets have ownerWallet set
SELECT COUNT(*) FROM "Ticket" WHERE "ownerWallet" IS NULL;
-- Should return 0

-- Check gift tickets (buyer != owner)
SELECT COUNT(*) FROM "Ticket" WHERE "buyerWallet" != "ownerWallet";

-- Verify data consistency
SELECT 
  COUNT(*) as total_tickets,
  COUNT(DISTINCT "buyerWallet") as unique_buyers,
  COUNT(DISTINCT "ownerWallet") as unique_owners
FROM "Ticket";
```

## Common Issues and Solutions

### Issue 1: Migration Fails Due to NULL Values
**Error:** `Column "ownerWallet" cannot be null`

**Solution:** Make the field optional first, backfill data, then make it required:
```prisma
ownerWallet String?  // Step 1: Optional
// Backfill data
ownerWallet String   // Step 2: Make required
```

### Issue 2: Existing Tickets Don't Show Up
**Error:** Queries filtering by `ownerWallet` return no results for old tickets

**Solution:** Ensure backfill script ran successfully:
```sql
SELECT * FROM "Ticket" WHERE "ownerWallet" IS NULL;
```

### Issue 3: Performance Issues with Large Tables
**Problem:** Backfill takes too long for millions of tickets

**Solution:** Use batch updates:
```typescript
const batchSize = 1000;
let skip = 0;

while (true) {
  const tickets = await prisma.ticket.findMany({
    where: { ownerWallet: null },
    take: batchSize,
    skip,
  });

  if (tickets.length === 0) break;

  await prisma.$transaction(
    tickets.map(ticket =>
      prisma.ticket.update({
        where: { id: ticket.id },
        data: { ownerWallet: ticket.buyerWallet },
      })
    )
  );

  skip += batchSize;
  console.log(`Processed ${skip} tickets`);
}
```

## Monitoring

After deployment, monitor:
1. **Error rates** - Check for any API errors related to ticket creation
2. **Database queries** - Ensure queries using `ownerWallet` are performant
3. **User reports** - Watch for any issues with ticket ownership display

## Support

If you encounter issues during migration:
1. Check the Prisma migration logs: `prisma/migrations/`
2. Review database logs for any constraint violations
3. Verify the backfill script completed successfully
4. Test with a small dataset first before production deployment
