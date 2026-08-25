-- Add host_email column to events table for organizer contact email
ALTER TABLE events ADD COLUMN IF NOT EXISTS host_email TEXT;
