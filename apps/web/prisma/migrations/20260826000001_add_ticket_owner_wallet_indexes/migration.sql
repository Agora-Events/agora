-- Issue #1281: Add indexes on Ticket.ownerWallet to speed up wallet-based lookups.
-- Single-column index for equality filters on ownerWallet.
CREATE INDEX "Ticket_ownerWallet_idx" ON "Ticket"("ownerWallet");

-- Composite index to support "newest-first" queries per wallet.
-- Covers: WHERE "ownerWallet" = $1 ORDER BY "createdAt" DESC
CREATE INDEX "Ticket_ownerWallet_createdAt_idx" ON "Ticket"("ownerWallet", "createdAt");
