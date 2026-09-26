-- Add ticket scan tracking fields and enforce scan status values
ALTER TABLE tickets
    ADD COLUMN IF NOT EXISTS scanned_at TIMESTAMPTZ;

COMMENT ON COLUMN tickets.scanned_at IS 'Timestamp when the ticket was scanned at event entry';

-- Existing ticket status values remain unaffected. New scan endpoint may
-- use the new `Scanned` value for tickets that have been checked in.
