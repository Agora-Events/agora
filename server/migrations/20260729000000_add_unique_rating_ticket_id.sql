-- Ensure a ticket may only have one rating
ALTER TABLE event_ratings
    ADD CONSTRAINT IF NOT EXISTS unique_event_ratings_ticket_id UNIQUE (ticket_id);
