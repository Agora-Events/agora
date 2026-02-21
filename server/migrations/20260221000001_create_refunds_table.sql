-- Create refunds table
-- Tracks refund requests and their status

CREATE TABLE refunds (
    id UUID PRIMARY KEY DEFAULT uuid_generate_v4(),
    ticket_id UUID NOT NULL REFERENCES tickets(id) ON DELETE CASCADE,
    amount DECIMAL(10, 2) NOT NULL,
    reason TEXT NOT NULL,
    status TEXT NOT NULL, -- pending, approved, rejected, completed
    initiated_by TEXT NOT NULL, -- 'guest' or 'organizer'
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    updated_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

-- Add index for ticket lookups
CREATE INDEX idx_refunds_ticket_id ON refunds(ticket_id);

-- Add index for status lookups
CREATE INDEX idx_refunds_status ON refunds(status);

-- Add trigger for updated_at
CREATE TRIGGER update_refunds_updated_at BEFORE UPDATE ON refunds FOR EACH ROW EXECUTE PROCEDURE update_updated_at_column();
