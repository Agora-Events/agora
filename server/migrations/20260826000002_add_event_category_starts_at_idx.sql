-- Issue #1282: Add composite index on "Event"("category", "startsAt") to support
-- category-browsing queries ordered by start date without a sequential scan.
--
-- Query accelerated:
--   SELECT * FROM "Event" WHERE category = $1 ORDER BY "startsAt" ASC;
--
-- Non-duplicate: existing indexes only cover organizer_id, start_time (server schema),
-- is_featured, and created_at. No category+startsAt index exists.

CREATE INDEX IF NOT EXISTS "Event_category_startsAt_idx" ON "Event"("category", "startsAt");
