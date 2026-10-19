-- Create qr_payloads table for storing cryptographically signed QR code data
CREATE TABLE IF NOT EXISTS qr_payloads (
    id TEXT PRIMARY KEY,
    qr_type TEXT NOT NULL,
    payload_data JSONB NOT NULL,
    signature TEXT NOT NULL,
    public_key TEXT NOT NULL,
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    expires_at TIMESTAMPTZ NOT NULL,
    is_used BOOLEAN NOT NULL DEFAULT FALSE,
    used_at TIMESTAMPTZ,
    CONSTRAINT valid_expiration CHECK (expires_at > created_at)
);

-- Index for querying by type
CREATE INDEX idx_qr_payloads_type ON qr_payloads(qr_type);

-- Index for querying by expiration
CREATE INDEX idx_qr_payloads_expires_at ON qr_payloads(expires_at);

-- Index for querying unused QR codes
CREATE INDEX idx_qr_payloads_is_used ON qr_payloads(is_used);

-- Index for querying by creation time
CREATE INDEX idx_qr_payloads_created_at ON qr_payloads(created_at);

-- Composite index for common queries
CREATE INDEX idx_qr_payloads_type_used ON qr_payloads(qr_type, is_used);

COMMENT ON TABLE qr_payloads IS 'Stores cryptographically signed QR code payloads with Ed25519 signatures';
COMMENT ON COLUMN qr_payloads.id IS 'Unique identifier for the QR payload (UUID)';
COMMENT ON COLUMN qr_payloads.qr_type IS 'Type of QR code (e.g., ticket, payment, access)';
COMMENT ON COLUMN qr_payloads.payload_data IS 'JSON data associated with the QR code';
COMMENT ON COLUMN qr_payloads.signature IS 'Base64-encoded Ed25519 signature of the payload';
COMMENT ON COLUMN qr_payloads.public_key IS 'Hex-encoded Ed25519 public key for verification';
COMMENT ON COLUMN qr_payloads.created_at IS 'Timestamp when the QR code was generated';
COMMENT ON COLUMN qr_payloads.expires_at IS 'Timestamp when the QR code expires';
COMMENT ON COLUMN qr_payloads.is_used IS 'Whether the QR code has been used/redeemed';
COMMENT ON COLUMN qr_payloads.used_at IS 'Timestamp when the QR code was used';
