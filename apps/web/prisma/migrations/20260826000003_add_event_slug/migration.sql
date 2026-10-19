-- Add a unique, human-readable slug to Event (Issue #1286)
ALTER TABLE "Event" ADD COLUMN "slug" TEXT;

-- Backfill from the title using the same rules as lib/slugify.ts.
UPDATE "Event"
SET "slug" = NULLIF(
    trim(both '-' from left(regexp_replace(lower("title"), '[^a-z0-9]+', '-', 'g'), 80)),
    ''
);

UPDATE "Event" SET "slug" = 'event' WHERE "slug" IS NULL;

-- Disambiguate titles that collapse to the same slug.
UPDATE "Event" AS e
SET "slug" = trim(both '-' from left(e."slug", 73)) || '-' || substr(md5(e."id"), 1, 6)
FROM (
    SELECT "id", row_number() OVER (PARTITION BY "slug" ORDER BY "createdAt", "id") AS rn
    FROM "Event"
) AS dup
WHERE dup."id" = e."id" AND dup.rn > 1;

ALTER TABLE "Event" ALTER COLUMN "slug" SET NOT NULL;

CREATE UNIQUE INDEX "Event_slug_key" ON "Event"("slug");
