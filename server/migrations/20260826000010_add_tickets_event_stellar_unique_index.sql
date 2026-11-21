-- no-transaction
-- Build concurrently so the index does not lock tickets on a live database.
CREATE UNIQUE INDEX CONCURRENTLY IF NOT EXISTS tickets_event_id_stellar_id_key
    ON tickets (event_id, stellar_id)
    WHERE stellar_id IS NOT NULL;
