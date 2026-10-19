-- Outgoing webhook endpoints and delivery logs (Issue #1339)
--
-- Organisers register HTTPS endpoints they want Agora to POST to whenever
-- key events occur (ticket sold, event created, attendee checked in).
-- Payloads are HMAC-SHA256 signed with the endpoint secret; recipients can
-- verify the signature using that shared secret.

CREATE TABLE IF NOT EXISTS webhook_endpoints (
    id UUID PRIMARY KEY DEFAULT uuid_generate_v4(),
    organizer_id UUID NOT NULL REFERENCES organizers(id) ON DELETE CASCADE,
    url TEXT NOT NULL,
    secret TEXT NOT NULL,
    events TEXT[] NOT NULL DEFAULT '{}',
    is_active BOOLEAN NOT NULL DEFAULT TRUE,
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    updated_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

-- A maximum of 5 registered endpoints per organiser. Enforced atomically in
-- the create handler via a guarded INSERT (`WHERE count(*) < 5`), so even
-- concurrent requests cannot exceed the cap.
CREATE INDEX IF NOT EXISTS idx_webhook_endpoints_organizer_active
    ON webhook_endpoints(organizer_id, is_active) WHERE is_active = TRUE;

CREATE INDEX IF NOT EXISTS idx_webhook_endpoints_organizer_active
    ON webhook_endpoints(organizer_id, is_active) WHERE is_active = TRUE;

CREATE TRIGGER update_webhook_endpoints_updated_at
    BEFORE UPDATE ON webhook_endpoints
    FOR EACH ROW EXECUTE PROCEDURE update_updated_at_column();

-- Per-attempt delivery record for retries and debugging.
CREATE TABLE IF NOT EXISTS webhook_delivery_logs (
    id UUID PRIMARY KEY DEFAULT uuid_generate_v4(),
    endpoint_id UUID NOT NULL REFERENCES webhook_endpoints(id) ON DELETE CASCADE,
    event TEXT NOT NULL,
    payload JSONB NOT NULL,
    attempt INT NOT NULL DEFAULT 1,
    status_code INT,
    error TEXT,
    delivered_at TIMESTAMPTZ,
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

CREATE INDEX IF NOT EXISTS idx_webhook_delivery_logs_endpoint
    ON webhook_delivery_logs(endpoint_id, created_at DESC);
CREATE INDEX IF NOT EXISTS idx_webhook_delivery_logs_event
    ON webhook_delivery_logs(event);