-- Issue #1135: Materialized analytics views for ticket sales and revenue.
-- Pre-aggregates expensive joins so large-event dashboards avoid scanning
-- tickets/transactions on every request. Existing analytics endpoints are
-- unchanged; consumers can opt into these views for faster reads.

-- ---------------------------------------------------------------------------
-- Daily ticket sales per event / organizer
-- ---------------------------------------------------------------------------
CREATE MATERIALIZED VIEW IF NOT EXISTS mv_daily_ticket_sales AS
SELECT
    (t.created_at AT TIME ZONE 'UTC')::date AS sale_date,
    COALESCE(t.event_id, tt.event_id) AS event_id,
    e.organizer_id,
    COUNT(*)::bigint AS tickets_sold,
    COUNT(*) FILTER (WHERE t.status IN ('active', 'Unused'))::bigint AS active_tickets,
    COUNT(*) FILTER (WHERE t.status IN ('used', 'Scanned'))::bigint AS used_tickets,
    COUNT(*) FILTER (WHERE t.status IN ('cancelled', 'Revoked'))::bigint AS cancelled_tickets
FROM tickets t
LEFT JOIN ticket_tiers tt ON tt.id = t.ticket_tier_id
JOIN events e ON e.id = COALESCE(t.event_id, tt.event_id)
GROUP BY 1, 2, 3;

-- Unique index required for REFRESH MATERIALIZED VIEW CONCURRENTLY
CREATE UNIQUE INDEX IF NOT EXISTS idx_mv_daily_ticket_sales_pk
    ON mv_daily_ticket_sales (sale_date, event_id);

CREATE INDEX IF NOT EXISTS idx_mv_daily_ticket_sales_organizer
    ON mv_daily_ticket_sales (organizer_id, sale_date DESC);

CREATE INDEX IF NOT EXISTS idx_mv_daily_ticket_sales_event
    ON mv_daily_ticket_sales (event_id, sale_date DESC);

-- ---------------------------------------------------------------------------
-- Daily revenue summary from completed/pending/failed transactions
-- ---------------------------------------------------------------------------
CREATE MATERIALIZED VIEW IF NOT EXISTS mv_daily_revenue_summary AS
SELECT
    (tx.created_at AT TIME ZONE 'UTC')::date AS revenue_date,
    COALESCE(t.event_id, tt.event_id) AS event_id,
    e.organizer_id,
    tx.currency,
    COALESCE(SUM(tx.amount) FILTER (WHERE tx.status = 'completed'), 0)::numeric(14, 2) AS total_revenue,
    COUNT(*) FILTER (WHERE tx.status = 'completed')::bigint AS completed_transactions,
    COUNT(*) FILTER (WHERE tx.status = 'pending')::bigint AS pending_transactions,
    COUNT(*) FILTER (WHERE tx.status = 'failed')::bigint AS failed_transactions
FROM transactions tx
JOIN tickets t ON t.id = tx.ticket_id
LEFT JOIN ticket_tiers tt ON tt.id = t.ticket_tier_id
JOIN events e ON e.id = COALESCE(t.event_id, tt.event_id)
GROUP BY 1, 2, 3, 4;

CREATE UNIQUE INDEX IF NOT EXISTS idx_mv_daily_revenue_summary_pk
    ON mv_daily_revenue_summary (revenue_date, event_id, currency);

CREATE INDEX IF NOT EXISTS idx_mv_daily_revenue_summary_organizer
    ON mv_daily_revenue_summary (organizer_id, revenue_date DESC);

CREATE INDEX IF NOT EXISTS idx_mv_daily_revenue_summary_event
    ON mv_daily_revenue_summary (event_id, revenue_date DESC);

-- ---------------------------------------------------------------------------
-- Event-level revenue rollup (lifetime totals)
-- ---------------------------------------------------------------------------
CREATE MATERIALIZED VIEW IF NOT EXISTS mv_event_revenue_summary AS
SELECT
    COALESCE(t.event_id, tt.event_id) AS event_id,
    e.organizer_id,
    tx.currency,
    COALESCE(SUM(tx.amount) FILTER (WHERE tx.status = 'completed'), 0)::numeric(14, 2) AS total_revenue,
    COUNT(*) FILTER (WHERE tx.status = 'completed')::bigint AS tickets_paid,
    COALESCE(AVG(tx.amount) FILTER (WHERE tx.status = 'completed'), 0)::numeric(14, 2) AS avg_ticket_price
FROM transactions tx
JOIN tickets t ON t.id = tx.ticket_id
LEFT JOIN ticket_tiers tt ON tt.id = t.ticket_tier_id
JOIN events e ON e.id = COALESCE(t.event_id, tt.event_id)
GROUP BY 1, 2, 3;

CREATE UNIQUE INDEX IF NOT EXISTS idx_mv_event_revenue_summary_pk
    ON mv_event_revenue_summary (event_id, currency);

CREATE INDEX IF NOT EXISTS idx_mv_event_revenue_summary_organizer
    ON mv_event_revenue_summary (organizer_id);

-- ---------------------------------------------------------------------------
-- Refresh strategy
-- ---------------------------------------------------------------------------
-- Call `SELECT refresh_analytics_materialized_views();` from a scheduled job
-- (e.g. cron every 5–15 minutes, or after bulk ticket imports).
-- CONCURRENTLY keeps the views readable during refresh and requires the
-- unique indexes defined above.
CREATE OR REPLACE FUNCTION refresh_analytics_materialized_views()
RETURNS void
LANGUAGE plpgsql
AS $$
BEGIN
    REFRESH MATERIALIZED VIEW CONCURRENTLY mv_daily_ticket_sales;
    REFRESH MATERIALIZED VIEW CONCURRENTLY mv_daily_revenue_summary;
    REFRESH MATERIALIZED VIEW CONCURRENTLY mv_event_revenue_summary;
END;
$$;

COMMENT ON FUNCTION refresh_analytics_materialized_views() IS
    'Refresh analytics materialized views concurrently. Schedule via cron/job queue (Issue #1135).';

COMMENT ON MATERIALIZED VIEW mv_daily_ticket_sales IS
    'Daily ticket sales aggregates per event for analytics dashboards (Issue #1135).';

COMMENT ON MATERIALIZED VIEW mv_daily_revenue_summary IS
    'Daily revenue aggregates per event/currency from transactions (Issue #1135).';

COMMENT ON MATERIALIZED VIEW mv_event_revenue_summary IS
    'Lifetime revenue rollup per event/currency (Issue #1135).';
