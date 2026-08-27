-- Prevent duplicate on-chain tickets for the same event (Issue #1287)
-- Rows with a null "stellarId" are excluded and stay unaffected.
DO $$
DECLARE
    duplicates TEXT;
BEGIN
    SELECT string_agg(format('(%s, %s) x%s', "eventId", "stellarId", n), ', ')
    INTO duplicates
    FROM (
        SELECT "eventId", "stellarId", count(*) AS n
        FROM "Ticket"
        WHERE "stellarId" IS NOT NULL
        GROUP BY "eventId", "stellarId"
        HAVING count(*) > 1
    ) AS d;

    IF duplicates IS NOT NULL THEN
        RAISE EXCEPTION 'Cannot create Ticket_eventId_stellarId_key: duplicate (eventId, stellarId) pairs exist: %', duplicates;
    END IF;
END
$$;

CREATE UNIQUE INDEX "Ticket_eventId_stellarId_key"
ON "Ticket"("eventId", "stellarId")
WHERE "stellarId" IS NOT NULL;
