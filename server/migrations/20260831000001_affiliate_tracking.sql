-- Affiliate tracking: referral codes and purchase attribution
-- (Issue #1151, consumed by the registration endpoint in #1150)

-- One affiliate registration per wallet per event.
--
-- The referral code is globally unique rather than unique-per-event so that a
-- code can be resolved from a checkout link on its own, without the event id
-- having to be carried alongside it and kept in sync.
CREATE TABLE IF NOT EXISTS event_affiliates (
    id UUID PRIMARY KEY DEFAULT uuid_generate_v4(),
    event_id UUID NOT NULL REFERENCES events(id) ON DELETE CASCADE,
    wallet_address TEXT NOT NULL,
    referral_code TEXT NOT NULL UNIQUE,
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    updated_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),

    -- Prevents duplicate registrations at the database level, so two
    -- concurrent requests cannot both create a registration for the same
    -- wallet on the same event.
    CONSTRAINT event_affiliates_event_wallet_key UNIQUE (event_id, wallet_address)
);

CREATE INDEX IF NOT EXISTS idx_event_affiliates_event ON event_affiliates(event_id);
CREATE INDEX IF NOT EXISTS idx_event_affiliates_wallet ON event_affiliates(wallet_address);

CREATE TRIGGER update_event_affiliates_updated_at
    BEFORE UPDATE ON event_affiliates
    FOR EACH ROW EXECUTE PROCEDURE update_updated_at_column();

-- Purchase attribution.
--
-- Nullable, with no default: existing transactions are untouched and simply
-- carry no attribution, which is the correct reading of "this sale predates
-- the affiliate programme".
--
-- ON DELETE SET NULL rather than CASCADE: removing an affiliate registration
-- must never delete the financial record of a completed purchase.
ALTER TABLE transactions
    ADD COLUMN IF NOT EXISTS referred_by TEXT
        REFERENCES event_affiliates(referral_code) ON DELETE SET NULL;

CREATE INDEX IF NOT EXISTS idx_transactions_referred_by
    ON transactions(referred_by)
    WHERE referred_by IS NOT NULL;
