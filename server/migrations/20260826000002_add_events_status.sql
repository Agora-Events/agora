-- Add a lifecycle status to events (Issue #1284).
-- Existing rows backfill to 'PUBLISHED' via the column default.
DO $$
BEGIN
    IF NOT EXISTS (SELECT 1 FROM pg_type WHERE typname = 'event_status') THEN
        CREATE TYPE event_status AS ENUM ('DRAFT', 'PUBLISHED', 'CANCELLED');
    END IF;
END
$$;

ALTER TABLE events
    ADD COLUMN IF NOT EXISTS status event_status NOT NULL DEFAULT 'PUBLISHED';

CREATE INDEX IF NOT EXISTS events_status_start_time_idx ON events (status, start_time);
