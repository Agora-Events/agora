-- Add is_flagged column to events table for content moderation

ALTER TABLE events
    ADD COLUMN is_flagged BOOLEAN NOT NULL DEFAULT FALSE;