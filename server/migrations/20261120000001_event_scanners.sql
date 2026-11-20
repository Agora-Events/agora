CREATE TABLE event_scanners (
    event_id UUID NOT NULL REFERENCES events(id) ON DELETE CASCADE,
    wallet_address TEXT NOT NULL,
    created_by UUID NOT NULL REFERENCES organizers(id) ON DELETE CASCADE,
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    PRIMARY KEY (event_id, wallet_address)
);

CREATE INDEX idx_event_scanners_wallet_address
    ON event_scanners (wallet_address);
