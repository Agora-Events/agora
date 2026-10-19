-- Add event_id and ticket_id to qr_payloads so organizers can list QR codes per event
ALTER TABLE qr_payloads ADD COLUMN IF NOT EXISTS event_id UUID REFERENCES events(id) ON DELETE CASCADE;
ALTER TABLE qr_payloads ADD COLUMN IF NOT EXISTS ticket_id UUID REFERENCES tickets(id) ON DELETE SET NULL;

CREATE INDEX idx_qr_payloads_event_id ON qr_payloads(event_id);
