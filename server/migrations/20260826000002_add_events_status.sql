-- Add a lifecycle status using the lowercase values used by the API.
ALTER TABLE events
    ADD COLUMN IF NOT EXISTS status TEXT NOT NULL DEFAULT 'published';

ALTER TABLE events
    ADD CONSTRAINT events_status_check
    CHECK (status IN ('draft', 'published', 'cancelled', 'archived'));

CREATE INDEX IF NOT EXISTS events_status_start_time_idx ON events (status, start_time);
