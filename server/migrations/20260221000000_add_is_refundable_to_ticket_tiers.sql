-- Add is_refundable column to ticket_tiers table
-- This column indicates whether tickets of this tier can be refunded by the buyer
-- Default value is true to maintain backward compatibility with existing tiers

ALTER TABLE ticket_tiers
ADD COLUMN is_refundable BOOLEAN NOT NULL DEFAULT true;

-- Add comment for clarity
COMMENT ON COLUMN ticket_tiers.is_refundable IS 'Indicates whether tickets of this tier can be refunded by the buyer. Organizer-initiated cancellations always result in refunds regardless of this flag.';
