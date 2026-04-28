-- Add event rating aggregates and a dedicated rating history table.

ALTER TABLE events
    ADD COLUMN sum_of_ratings BIGINT NOT NULL DEFAULT 0,
    ADD COLUMN count_of_ratings INTEGER NOT NULL DEFAULT 0;

CREATE TABLE event_ratings (
    id UUID PRIMARY KEY DEFAULT uuid_generate_v4(),
    event_id UUID NOT NULL REFERENCES events(id) ON DELETE CASCADE,
    ticket_id UUID NOT NULL REFERENCES tickets(id) ON DELETE CASCADE,
    rating SMALLINT NOT NULL CHECK (rating >= 1 AND rating <= 5),
    review TEXT,
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    updated_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    UNIQUE(event_id, ticket_id)
);

CREATE TRIGGER update_event_ratings_updated_at
    BEFORE UPDATE ON event_ratings
    FOR EACH ROW
    EXECUTE PROCEDURE update_updated_at_column();
