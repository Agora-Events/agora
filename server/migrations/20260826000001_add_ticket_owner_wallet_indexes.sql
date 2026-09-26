-- Issue #1281: Add indexes on "Ticket"."ownerWallet" for efficient wallet-based lookups.
--
-- Queries accelerated:
--   SELECT * FROM "Ticket" WHERE "ownerWallet" = $1;
--   SELECT * FROM "Ticket" WHERE "ownerWallet" = $1 ORDER BY "createdAt" DESC;
--
-- The single-column index handles equality filters; the composite index additionally
-- covers ORDER BY "createdAt" without a separate sort step.

CREATE INDEX IF NOT EXISTS "Ticket_ownerWallet_idx" ON "Ticket"("ownerWallet");
CREATE INDEX IF NOT EXISTS "Ticket_ownerWallet_createdAt_idx" ON "Ticket"("ownerWallet", "createdAt");
