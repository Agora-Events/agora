-- Enforce that an event's end time, when set, comes after its start time (Issue #1283).
-- `events.end_time` already exists and stays nullable, so existing rows are unaffected.
ALTER TABLE events
    ADD CONSTRAINT events_end_time_after_start_time_check
    CHECK (end_time IS NULL OR end_time > start_time);
