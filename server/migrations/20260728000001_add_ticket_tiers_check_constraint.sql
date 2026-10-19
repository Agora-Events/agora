-- Add CHECK constraint to prevent negative available_quantity (Issue #834)
-- This prevents race conditions from causing negative inventory
ALTER TABLE ticket_tiers 
    ADD CONSTRAINT check_available_quantity_non_negative 
    CHECK (available_quantity >= 0);
