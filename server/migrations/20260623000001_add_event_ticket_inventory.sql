-- Add aggregate ticket inventory fields used by event list filters and count endpoints.

ALTER TABLE events
    ADD COLUMN IF NOT EXISTS total_tickets BIGINT NOT NULL DEFAULT 0,
    ADD COLUMN IF NOT EXISTS minted_tickets BIGINT NOT NULL DEFAULT 0;

ALTER TABLE events
    ADD CONSTRAINT events_ticket_inventory_non_negative
        CHECK (total_tickets >= 0 AND minted_tickets >= 0);
