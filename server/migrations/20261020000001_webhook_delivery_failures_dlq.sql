-- Webhook Dead-Letter Queue (DLQ) for permanently failed deliveries (Issue #1626)

CREATE TABLE IF NOT EXISTS webhook_delivery_failures (
    id UUID PRIMARY KEY DEFAULT uuid_generate_v4(),
    endpoint_id UUID NOT NULL REFERENCES webhook_endpoints(id) ON DELETE CASCADE,
    event TEXT NOT NULL,
    payload JSONB NOT NULL,
    attempts INT NOT NULL DEFAULT 3,
    last_status_code INT,
    last_error TEXT,
    failed_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    retried_at TIMESTAMPTZ,
    retry_status TEXT
);

CREATE INDEX IF NOT EXISTS idx_webhook_delivery_failures_endpoint
    ON webhook_delivery_failures(endpoint_id, failed_at DESC);
CREATE INDEX IF NOT EXISTS idx_webhook_delivery_failures_retry
    ON webhook_delivery_failures(retry_status, failed_at DESC);
