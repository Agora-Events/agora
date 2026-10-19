-- Add a metadata JSONB column to audit_logs for action-specific context,
-- e.g. the organizer wallet affected by an admin toggle-flag action (Issue #586).
ALTER TABLE audit_logs ADD COLUMN IF NOT EXISTS metadata JSONB;
