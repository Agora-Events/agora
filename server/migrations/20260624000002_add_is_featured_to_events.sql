-- Add is_featured flag to events for home page curation
ALTER TABLE events ADD COLUMN IF NOT EXISTS is_featured BOOLEAN NOT NULL DEFAULT FALSE;

CREATE INDEX idx_events_is_featured ON events(is_featured) WHERE is_featured = TRUE;
