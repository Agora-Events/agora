-- Add a status column to events so lifecycle transitions
-- (draft -> published -> cancelled) are explicit rather than inferred.

ALTER TABLE events
ADD COLUMN IF NOT EXISTS status TEXT NOT NULL DEFAULT 'published';

ALTER TABLE events
ADD CSONSTRAINT events_status_check
CHECK (status IN ('draft', 'published', 'cancelled', 'archived'));

CREATE INDEX IF NOT EXISTS idx_events_status
ON events(status);
