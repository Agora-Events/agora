-- Composite index for the `has_ticket` endpoint
-- (GET /api/v1/events/:event_id/has-ticket?wallet=...)
--
-- The handler runs:
--   SELECT EXISTS(
--       SELECT 1 FROM tickets
--       WHERE event_id    = $1
--         AND owner_wallet = $2
--         AND status       = 'active'
--   )
--
-- This index covers the full predicate so Postgres can satisfy it with a single
-- index scan rather than a sequential scan of the tickets table.
CREATE INDEX IF NOT EXISTS idx_tickets_has_ticket
    ON tickets (event_id, owner_wallet, status);
