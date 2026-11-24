ALTER TABLE developer_api_keys
    ADD COLUMN expires_at TIMESTAMPTZ;

CREATE INDEX idx_developer_api_keys_expires_at
    ON developer_api_keys (expires_at)
    WHERE expires_at IS NOT NULL;