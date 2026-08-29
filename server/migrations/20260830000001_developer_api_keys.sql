-- Developer API keys for organiser programmatic access (Issue #1340)
CREATE TABLE IF NOT EXISTS developer_api_keys (
    id UUID PRIMARY KEY DEFAULT uuid_generate_v4(),
    organizer_id TEXT NOT NULL REFERENCES organizer_profiles(address) ON DELETE CASCADE,
    key_hash TEXT NOT NULL UNIQUE,
    prefix TEXT NOT NULL,
    name TEXT NOT NULL,
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    last_used_at TIMESTAMPTZ,
    is_active BOOLEAN NOT NULL DEFAULT TRUE
);

CREATE INDEX IF NOT EXISTS idx_developer_api_keys_organizer ON developer_api_keys(organizer_id);
CREATE INDEX IF NOT EXISTS idx_developer_api_keys_hash ON developer_api_keys(key_hash);
CREATE INDEX IF NOT EXISTS idx_developer_api_keys_active ON developer_api_keys(is_active) WHERE is_active = TRUE;
