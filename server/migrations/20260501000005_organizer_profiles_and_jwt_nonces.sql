-- Migration: Organizer profiles and JWT nonces for wallet-based auth
-- Issue #486: Organizer Profile Management
-- Issue #484: JWT Authentication

-- Organizer profiles table (address is the Stellar wallet address, used as PK)
CREATE TABLE organizer_profiles (
    address       TEXT PRIMARY KEY,
    display_name  TEXT NOT NULL CHECK (char_length(display_name) <= 50),
    bio           TEXT CHECK (bio IS NULL OR char_length(bio) <= 500),
    avatar_url    TEXT,
    socials       JSONB NOT NULL DEFAULT '{}',
    created_at    TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    updated_at    TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

CREATE TRIGGER update_organizer_profiles_updated_at
    BEFORE UPDATE ON organizer_profiles
    FOR EACH ROW EXECUTE PROCEDURE update_updated_at_column();

-- JWT nonces table for challenge-response authentication (Issue #484)
-- Nonces are short-lived (5 minutes) and single-use
CREATE TABLE jwt_nonces (
    id         UUID PRIMARY KEY DEFAULT uuid_generate_v4(),
    nonce      TEXT NOT NULL UNIQUE,
    address    TEXT NOT NULL,
    used       BOOLEAN NOT NULL DEFAULT FALSE,
    expires_at TIMESTAMPTZ NOT NULL DEFAULT (NOW() + INTERVAL '5 minutes'),
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

-- Index for fast nonce lookup and cleanup
CREATE INDEX idx_jwt_nonces_nonce ON jwt_nonces (nonce);
CREATE INDEX idx_jwt_nonces_expires_at ON jwt_nonces (expires_at);
