-- The category lookup is indexed on event_categories; this index supports the
-- start-time ordering after the join.
CREATE INDEX IF NOT EXISTS idx_events_start_time_id ON events(start_time, id);
