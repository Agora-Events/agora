-- Add CHECK constraint to limit event description length to 10,000 characters
-- This prevents extremely large payloads that could cause DoS issues

ALTER TABLE events
ADD CONSTRAINT events_description_length_check
CHECK (description IS NULL OR LENGTH(description) <= 10000);

-- Add index on is_flagged for better query performance (if not already exists)
CREATE INDEX IF NOT EXISTS idx_events_is_flagged ON events(is_flagged);

COMMENT ON CONSTRAINT events_description_length_check ON events IS
'Ensures event descriptions do not exceed 10,000 characters to prevent DoS attacks';