//! Read models backed by analytics materialized views (Issue #1135).
//!
//! These structs map to PostgreSQL materialized views that pre-aggregate
//! ticket sales and revenue. Prefer querying these views for dashboards
//! instead of scanning `tickets` / `transactions` directly. Refresh via
//! `refresh_analytics_materialized_views()`.

use chrono::NaiveDate;
use rust_decimal::Decimal;
use serde::{Deserialize, Serialize};
use sqlx::FromRow;
use uuid::Uuid;

/// Daily ticket sales row from `mv_daily_ticket_sales`.
#[derive(Debug, Clone, Serialize, Deserialize, FromRow)]
pub struct DailyTicketSales {
    pub sale_date: NaiveDate,
    pub event_id: Uuid,
    pub organizer_id: Uuid,
    pub tickets_sold: i64,
    pub active_tickets: i64,
    pub used_tickets: i64,
    pub cancelled_tickets: i64,
}

/// Daily revenue row from `mv_daily_revenue_summary`.
#[derive(Debug, Clone, Serialize, Deserialize, FromRow)]
pub struct DailyRevenueSummary {
    pub revenue_date: NaiveDate,
    pub event_id: Uuid,
    pub organizer_id: Uuid,
    pub currency: String,
    pub total_revenue: Decimal,
    pub completed_transactions: i64,
    pub pending_transactions: i64,
    pub failed_transactions: i64,
}

/// Lifetime event revenue row from `mv_event_revenue_summary`.
#[derive(Debug, Clone, Serialize, Deserialize, FromRow)]
pub struct EventRevenueSummary {
    pub event_id: Uuid,
    pub organizer_id: Uuid,
    pub currency: String,
    pub total_revenue: Decimal,
    pub tickets_paid: i64,
    pub avg_ticket_price: Decimal,
}

/// Refresh all analytics materialized views concurrently.
///
/// Intended to be called from a scheduled job (cron / background worker).
/// Uses `REFRESH MATERIALIZED VIEW CONCURRENTLY` so readers are not blocked.
pub async fn refresh_analytics_materialized_views(
    pool: &sqlx::PgPool,
) -> Result<(), sqlx::Error> {
    sqlx::query("SELECT refresh_analytics_materialized_views()")
        .execute(pool)
        .await?;
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::str::FromStr;

    #[test]
    fn daily_ticket_sales_serializes() {
        let row = DailyTicketSales {
            sale_date: NaiveDate::from_ymd_opt(2026, 7, 29).unwrap(),
            event_id: Uuid::nil(),
            organizer_id: Uuid::nil(),
            tickets_sold: 10,
            active_tickets: 8,
            used_tickets: 1,
            cancelled_tickets: 1,
        };
        let json = serde_json::to_value(&row).unwrap();
        assert_eq!(json["tickets_sold"], 10);
        assert_eq!(json["sale_date"], "2026-07-29");
    }

    #[test]
    fn daily_revenue_summary_serializes() {
        let row = DailyRevenueSummary {
            revenue_date: NaiveDate::from_ymd_opt(2026, 7, 29).unwrap(),
            event_id: Uuid::nil(),
            organizer_id: Uuid::nil(),
            currency: "USD".into(),
            total_revenue: Decimal::from_str("125.50").unwrap(),
            completed_transactions: 5,
            pending_transactions: 1,
            failed_transactions: 0,
        };
        let json = serde_json::to_value(&row).unwrap();
        assert_eq!(json["currency"], "USD");
        assert_eq!(json["completed_transactions"], 5);
    }

    #[test]
    fn event_revenue_summary_serializes() {
        let row = EventRevenueSummary {
            event_id: Uuid::nil(),
            organizer_id: Uuid::nil(),
            currency: "USDC".into(),
            total_revenue: Decimal::from_str("1000.00").unwrap(),
            tickets_paid: 40,
            avg_ticket_price: Decimal::from_str("25.00").unwrap(),
        };
        let json = serde_json::to_value(&row).unwrap();
        assert_eq!(json["tickets_paid"], 40);
        assert_eq!(json["currency"], "USDC");
    }
}
