-- Add image_url column to events table for event banner/cover images.
-- The column is nullable; validation (HTTPS URL requirement) is enforced
-- at the application layer, not the database layer.
ALTER TABLE events
    ADD COLUMN IF NOT EXISTS image_url TEXT;
