-- Add Soroban on-chain fields to tickets table for webhook listener sync (Issue #490)
-- These columns allow the listener to upsert tickets from on-chain events
-- without requiring a user_id or ticket_tier_id (which may not exist yet).

ALTER TABLE tickets
    ADD COLUMN IF NOT EXISTS stellar_id    TEXT UNIQUE,
    ADD COLUMN IF NOT EXISTS event_id      UUID REFERENCES events(id) ON DELETE CASCADE,
    ADD COLUMN IF NOT EXISTS buyer_wallet  TEXT,
    ADD COLUMN IF NOT EXISTS owner_wallet  TEXT;

-- Make user_id and ticket_tier_id nullable so on-chain synced tickets
-- can exist before they are matched to a platform user/tier.
ALTER TABLE tickets
    ALTER COLUMN user_id DROP NOT NULL,
    ALTER COLUMN ticket_tier_id DROP NOT NULL;

-- Index for fast stellar_id lookups (used by the listener for idempotency)
CREATE INDEX IF NOT EXISTS idx_tickets_stellar_id ON tickets (stellar_id);
CREATE INDEX IF NOT EXISTS idx_tickets_event_id ON tickets (event_id);
CREATE INDEX IF NOT EXISTS idx_tickets_buyer_wallet ON tickets (buyer_wallet);
CREATE INDEX IF NOT EXISTS idx_tickets_owner_wallet ON tickets (owner_wallet);
