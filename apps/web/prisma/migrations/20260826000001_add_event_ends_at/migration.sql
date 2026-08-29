-- Add an optional end time to Event (Issue #1283)
ALTER TABLE "Event" ADD COLUMN "endsAt" TIMESTAMP(3);

-- An end time, when present, must come after the start time.
ALTER TABLE "Event"
ADD CONSTRAINT "Event_endsAt_after_startsAt_check"
CHECK ("endsAt" IS NULL OR "endsAt" > "startsAt");
