-- Harden event location coordinates for map-based discovery (Issue #1136).
-- Complements 20260729000001_add_event_coordinates.sql with range checks and
-- a partial index that only covers events that have coordinates set.

ALTER TABLE events
    DROP CONSTRAINT IF EXISTS events_latitude_range,
    DROP CONSTRAINT IF EXISTS events_longitude_range;

ALTER TABLE events
    ADD CONSTRAINT events_latitude_range
        CHECK (latitude IS NULL OR (latitude >= -90 AND latitude <= 90)),
    ADD CONSTRAINT events_longitude_range
        CHECK (longitude IS NULL OR (longitude >= -180 AND longitude <= 180));

-- Replace the full composite index with a partial one so location queries skip
-- events that have not been geocoded yet (existing rows remain NULL/unaffected).
DROP INDEX IF EXISTS idx_events_coordinates;

CREATE INDEX idx_events_coordinates
    ON events (latitude, longitude)
    WHERE latitude IS NOT NULL AND longitude IS NOT NULL;
