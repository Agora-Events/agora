-- Add indexes on ticket owner wallets for efficient wallet-based lookups.
--
-- Queries accelerated:
--   SELECT * FROM tickets WHERE owner_wallet = $1;
--   SELECT * FROM tickets WHERE owner_wallet = $1 ORDER BY created_at DESC;
--
-- The composite index also supports ordering by creation time.

CREATE INDEX IF NOT EXISTS idx_tickets_owner_wallet ON tickets(owner_wallet);
CREATE INDEX IF NOT EXISTS idx_tickets_owner_wallet_created_at ON tickets(owner_wallet, created_at);
