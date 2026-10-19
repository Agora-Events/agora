-- Add Stellar wallet address to organizers table
-- This links organizers to their on-chain identity and organizer_profiles records.
ALTER TABLE organizers
    ADD COLUMN IF NOT EXISTS wallet_address TEXT;

-- Index for fast profile lookups by wallet
CREATE INDEX IF NOT EXISTS idx_organizers_wallet_address ON organizers (wallet_address);
