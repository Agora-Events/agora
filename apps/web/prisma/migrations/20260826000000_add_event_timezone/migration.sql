ALTER TABLE "Event"
ADD COLUMN "timezone" TEXT NOT NULL DEFAULT 'UTC';

ALTER TABLE "Event"
ADD CONSTRAINT "Event_timezone_not_empty" CHECK (length(trim("timezone")) > 0);
