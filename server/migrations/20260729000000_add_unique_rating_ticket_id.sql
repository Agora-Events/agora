-- Ensure a ticket may only have one rating
DO $$
BEGIN
    IF NOT EXISTS (
        SELECT 1
        FROM pg_constraint
        WHERE conname = 'unique_event_ratings_ticket_id'
          AND conrelid = 'event_ratings'::regclass
    ) THEN
        ALTER TABLE event_ratings
            ADD CONSTRAINT unique_event_ratings_ticket_id UNIQUE (ticket_id);
    END IF;
END
$$;
