-- Composite index to speed up event browwsing filtered by category
-- and ordered by start time.

CREATE INDEX IF NOT EXISTS idx_events_category_starts_at
ON events(category, starts_at);
