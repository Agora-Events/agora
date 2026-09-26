-- Add explicit unique indexes for tickets table (Issue #835)
-- These indexes ensure efficient upserts and joins for on-chain sync

-- Unique index on stellar_id for efficient conflict detection in upserts
CREATE UNIQUE INDEX IF NOT EXISTS tickets_stellar_id_idx ON tickets (stellar_id);

-- Index on event_id for efficient joins and filters
CREATE INDEX IF NOT EXISTS tickets_event_id_idx ON tickets (event_id);
