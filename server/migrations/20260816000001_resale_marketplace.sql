-- Secondary ticket market with end-to-end encrypted key handover (Issue #1184)
--
-- Settlement (payment, royalty, ownership) happens on-chain in the
-- `ticket_payment` contract. These tables are the off-chain half:
--   * a queryable mirror of on-chain listings, so buyers can browse without
--     scanning ledger state,
--   * buyer offers carrying the X25519 public key the seller will seal to,
--   * the sealed ticket secret itself.
--
-- The server is a BLIND RELAY for the last of those. Everything stored in
-- `resale_key_envelopes` is ciphertext produced on the seller's device and
-- opened on the buyer's; the server holds no key that can decrypt it and must
-- never be given one. Ticket ids here are on-chain `payment_id` strings, not
-- platform UUIDs — the contract is the source of truth for ownership.

CREATE TABLE IF NOT EXISTS resale_listings (
    -- On-chain `payment_id` of the ticket. Primary key because the contract
    -- also allows only one listing per ticket at a time.
    payment_id            TEXT PRIMARY KEY,
    event_id              TEXT NOT NULL,
    seller_wallet         TEXT NOT NULL,
    -- Prices in token base units (stroops, 7dp for USDC) to avoid float drift.
    price_stroops         BIGINT NOT NULL CHECK (price_stroops > 0),
    -- Ceiling the contract validated this listing against, mirrored so the
    -- browse endpoint can show remaining headroom without an RPC round-trip.
    max_price_stroops     BIGINT NOT NULL CHECK (max_price_stroops > 0),
    royalty_bps           INTEGER NOT NULL CHECK (royalty_bps BETWEEN 0 AND 10000),
    status                TEXT NOT NULL DEFAULT 'active'
                              CHECK (status IN ('active', 'cancelled', 'sold')),
    -- Populated on settlement.
    buyer_wallet          TEXT,
    listing_tx_hash       TEXT,
    sale_tx_hash          TEXT,
    sold_at               TIMESTAMPTZ,
    created_at            TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    updated_at            TIMESTAMPTZ NOT NULL DEFAULT NOW(),

    CONSTRAINT resale_listings_price_within_cap CHECK (price_stroops <= max_price_stroops)
);

-- Browse feed: active listings for an event, newest first.
CREATE INDEX IF NOT EXISTS idx_resale_listings_event_status
    ON resale_listings (event_id, status, created_at DESC);
-- "My listings" for a seller.
CREATE INDEX IF NOT EXISTS idx_resale_listings_seller
    ON resale_listings (seller_wallet, status);

-- Buyer offers. The buyer's X25519 public key is registered here so the seller
-- has something to seal the ticket secret to once they accept.
CREATE TABLE IF NOT EXISTS resale_offers (
    id                    UUID PRIMARY KEY DEFAULT uuid_generate_v4(),
    payment_id            TEXT NOT NULL REFERENCES resale_listings(payment_id) ON DELETE CASCADE,
    buyer_wallet          TEXT NOT NULL,
    -- Base64 X25519 public key (32 raw bytes). Distinct from the buyer's
    -- Stellar Ed25519 signing key: signing keys must not be reused for key
    -- agreement, so the app derives a separate encryption keypair.
    buyer_public_key      TEXT NOT NULL,
    offer_price_stroops   BIGINT NOT NULL CHECK (offer_price_stroops > 0),
    status                TEXT NOT NULL DEFAULT 'pending'
                              CHECK (status IN ('pending', 'accepted', 'withdrawn', 'declined')),
    created_at            TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    updated_at            TIMESTAMPTZ NOT NULL DEFAULT NOW(),

    -- One standing offer per buyer per listing; re-offering updates in place.
    CONSTRAINT resale_offers_unique_buyer UNIQUE (payment_id, buyer_wallet)
);

CREATE INDEX IF NOT EXISTS idx_resale_offers_payment
    ON resale_offers (payment_id, status);
CREATE INDEX IF NOT EXISTS idx_resale_offers_buyer
    ON resale_offers (buyer_wallet, status);

-- The sealed ticket secret. Every column below the wallet is opaque to the
-- server: it stores and returns bytes it cannot read.
CREATE TABLE IF NOT EXISTS resale_key_envelopes (
    id                    UUID PRIMARY KEY DEFAULT uuid_generate_v4(),
    payment_id            TEXT NOT NULL REFERENCES resale_listings(payment_id) ON DELETE CASCADE,
    -- Only this wallet may fetch the envelope.
    buyer_wallet          TEXT NOT NULL,
    -- Base64 X25519 public key of the seller's ephemeral sending keypair.
    ephemeral_public_key  TEXT NOT NULL,
    -- Base64 24-byte XSalsa20 nonce.
    nonce                 TEXT NOT NULL,
    -- Base64 NaCl box ciphertext wrapping the ticket's check-in secret.
    ciphertext            TEXT NOT NULL,
    -- Set the first time the buyer successfully fetches it, so a seller can
    -- see the handover completed.
    claimed_at            TIMESTAMPTZ,
    created_at            TIMESTAMPTZ NOT NULL DEFAULT NOW(),

    CONSTRAINT resale_key_envelopes_unique UNIQUE (payment_id, buyer_wallet)
);

CREATE INDEX IF NOT EXISTS idx_resale_key_envelopes_buyer
    ON resale_key_envelopes (buyer_wallet);

-- Device push tokens, so a seller can be told their ticket sold while the app
-- is backgrounded.
CREATE TABLE IF NOT EXISTS push_tokens (
    id                    UUID PRIMARY KEY DEFAULT uuid_generate_v4(),
    wallet_address        TEXT NOT NULL,
    -- Expo push token, e.g. `ExponentPushToken[xxxxxxxx]`.
    token                 TEXT NOT NULL UNIQUE,
    platform              TEXT NOT NULL CHECK (platform IN ('ios', 'android', 'web')),
    created_at            TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    updated_at            TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

CREATE INDEX IF NOT EXISTS idx_push_tokens_wallet ON push_tokens (wallet_address);

CREATE TRIGGER update_resale_listings_updated_at
    BEFORE UPDATE ON resale_listings
    FOR EACH ROW EXECUTE FUNCTION update_updated_at_column();

CREATE TRIGGER update_resale_offers_updated_at
    BEFORE UPDATE ON resale_offers
    FOR EACH ROW EXECUTE FUNCTION update_updated_at_column();

CREATE TRIGGER update_push_tokens_updated_at
    BEFORE UPDATE ON push_tokens
    FOR EACH ROW EXECUTE FUNCTION update_updated_at_column();
