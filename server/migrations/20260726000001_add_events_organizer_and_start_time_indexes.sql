-- Indexes for organizer-filtered listings and default start_time ordering
CREATE INDEX IF NOT EXISTS events_organizer_id_idx ON events (organizer_id);
CREATE INDEX IF NOT EXISTS events_start_time_idx ON events (start_time ASC);
