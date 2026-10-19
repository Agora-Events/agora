-- Localised event descriptions (Issue #1344)
--
-- Organisers operating in non-English markets can provide their event title
-- and description in multiple languages. Translations live in their own table
-- so the canonical `events` row stays the source of truth for the default
-- locale, and each GET can overlay the requested `Accept-Language`.
--
-- The (event_id, locale) unique constraint is enforced at the database level
-- so concurrent translations for the same locale cannot be duplicated.

CREATE TABLE IF NOT EXISTS event_translations (
    id UUID PRIMARY KEY DEFAULT uuid_generate_v4(),
    event_id UUID NOT NULL REFERENCES events(id) ON DELETE CASCADE,
    locale TEXT NOT NULL,
    title TEXT NOT NULL,
    description TEXT,
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    updated_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),

    CONSTRAINT event_translations_event_locale_key UNIQUE (event_id, locale)
);

CREATE INDEX IF NOT EXISTS idx_event_translations_event_id
    ON event_translations(event_id);

CREATE TRIGGER update_event_translations_updated_at
    BEFORE UPDATE ON event_translations
    FOR EACH ROW EXECUTE PROCEDURE update_updated_at_column();

-- Optional IPFS CID for the localised metadata JSON (`translations` payload)
-- that organisers can pin and reference on-chain via the event_registry
-- contract's `update_metadata` function. Null until pinned.
ALTER TABLE events
    ADD COLUMN IF NOT EXISTS localised_metadata_cid TEXT;