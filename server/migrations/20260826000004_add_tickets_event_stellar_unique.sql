-- no-transaction
-- Prevent duplicate on-chain tickets for the same event (Issue #1287).
-- `tickets_stellar_id_idx` (20260728000002) only covers stellar_id on its own,
-- so this partial index on (event_id, stellar_id) is not a duplicate of it.
-- Rows with a null stellar_id are excluded and stay unaffected.
DO $$
DECLARE
    duplicates TEXT;
BEGIN
    SELECT string_agg(format('(%s, %s) x%s', event_id, stellar_id, n), ', ')
    INTO duplicates
    FROM (
        SELECT event_id, stellar_id, count(*) AS n
        FROM tickets
        WHERE stellar_id IS NOT NULL
        GROUP BY event_id, stellar_id
        HAVING count(*) > 1
    ) AS d;

    IF duplicates IS NOT NULL THEN
        RAISE EXCEPTION 'Cannot create tickets_event_id_stellar_id_key: duplicate (event_id, stellar_id) pairs exist: %', duplicates;
    END IF;
END
$$;

-- CONCURRENTLY so the index build does not lock tickets on a live database.
CREATE UNIQUE INDEX CONCURRENTLY IF NOT EXISTS tickets_event_id_stellar_id_key
    ON tickets (event_id, stellar_id)
    WHERE stellar_id IS NOT NULL;
