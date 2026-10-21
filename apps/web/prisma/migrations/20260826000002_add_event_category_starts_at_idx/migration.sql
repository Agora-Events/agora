-- Issue #1282: Add composite index on Event(category, startsAt) to speed up
-- category-browsing queries ordered by event start date.
-- Covers: WHERE category = $1 ORDER BY "startsAt" ASC/DESC
CREATE INDEX "Event_category_startsAt_idx" ON "Event"("category", "startsAt");
