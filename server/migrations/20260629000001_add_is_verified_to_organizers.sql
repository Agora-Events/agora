-- Add is_verified to organizers
ALTER TABLE organizers ADD COLUMN is_verified BOOLEAN NOT NULL DEFAULT FALSE;
