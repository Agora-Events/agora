-- Indexes for frequent event listing queries (is_featured filter, created_at ordering)
CREATE INDEX idx_events_featured ON events(is_featured);
CREATE INDEX idx_events_created_at ON events(created_at DESC);
