-- Add a unique, human-readable slug to events (Issue #1286).
ALTER TABLE events ADD COLUMN IF NOT EXISTS slug TEXT;

UPDATE events
SET slug = NULLIF(
    trim(both '-' from left(regexp_replace(lower(title), '[^a-z0-9]+', '-', 'g'), 80)),
    ''
)
WHERE slug IS NULL;

UPDATE events SET slug = 'event' WHERE slug IS NULL;

-- Disambiguate titles that collapse to the same slug.
UPDATE events AS e
SET slug = trim(both '-' from left(e.slug, 73)) || '-' || substr(md5(e.id::text), 1, 6)
FROM (
    SELECT id, row_number() OVER (PARTITION BY slug ORDER BY created_at, id) AS rn
    FROM events
) AS dup
WHERE dup.id = e.id AND dup.rn > 1;

ALTER TABLE events ALTER COLUMN slug SET NOT NULL;

CREATE UNIQUE INDEX IF NOT EXISTS events_slug_key ON events (slug);
