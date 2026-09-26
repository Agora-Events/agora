-- Add a lifecycle status to Event (Issue #1284)
CREATE TYPE "EventStatus" AS ENUM ('DRAFT', 'PUBLISHED', 'CANCELLED');

-- Existing rows are all live, so they backfill to PUBLISHED via the default.
ALTER TABLE "Event"
ADD COLUMN "status" "EventStatus" NOT NULL DEFAULT 'PUBLISHED';

CREATE INDEX "Event_status_startsAt_idx" ON "Event"("status", "startsAt");
