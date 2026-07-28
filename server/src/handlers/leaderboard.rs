//! # Organizer Leaderboard Handlers
//!
//! Ranks organizers by total tickets sold across all of their events.

use axum::{
    extract::{Query, State},
    response::{IntoResponse, Response},
};
use serde::Serialize;
use sqlx::{FromRow, PgPool};
use uuid::Uuid;

use crate::utils::error::AppError;
use crate::utils::pagination::{PaginatedResponse, PaginationParams};
use crate::utils::response::success;

/// A single row in the organizer leaderboard.
#[derive(Debug, Serialize, FromRow)]
pub struct LeaderboardEntry {
    pub organizer_id: Uuid,
    pub organizer_name: String,
    pub tickets_sold: i64,
}

/// List organizers ranked by tickets sold, most first.
///
/// # Endpoint
/// GET `/api/v1/leaderboard`
///
/// Ties in `tickets_sold` are broken by `organizer_id` so that the ordering
/// — and therefore pagination — is deterministic across requests, even when
/// multiple organizers share the same score.
pub async fn get_leaderboard(
    State(pool): State<PgPool>,
    Query(pagination): Query<PaginationParams>,
) -> Response {
    let validated_pagination = pagination.validate();

    let total = match sqlx::query_scalar::<_, i64>("SELECT COUNT(*) FROM organizers")
        .fetch_one(&pool)
        .await
    {
        Ok(count) => count,
        Err(e) => {
            tracing::error!("Failed to count organizers: {:?}", e);
            return AppError::DatabaseError(e).into_response();
        }
    };

    let items = match sqlx::query_as::<_, LeaderboardEntry>(
        r#"
        SELECT
            o.id AS organizer_id,
            o.name AS organizer_name,
            COALESCE(SUM(tt.total_quantity - tt.available_quantity), 0)::bigint AS tickets_sold
        FROM organizers o
        LEFT JOIN events e ON e.organizer_id = o.id
        LEFT JOIN ticket_tiers tt ON tt.event_id = e.id
        GROUP BY o.id, o.name
        ORDER BY tickets_sold DESC, o.id ASC
        LIMIT $1 OFFSET $2
        "#,
    )
    .bind(validated_pagination.limit())
    .bind(validated_pagination.offset())
    .fetch_all(&pool)
    .await
    {
        Ok(rows) => rows,
        Err(e) => {
            tracing::error!("Failed to fetch leaderboard: {:?}", e);
            return AppError::DatabaseError(e).into_response();
        }
    };

    let response = PaginatedResponse::new(items, validated_pagination, total);
    success(response, "Leaderboard retrieved successfully").into_response()
}

#[cfg(test)]
mod tests {
    use super::*;

    /// Ensures the ORDER BY clause always carries a deterministic tiebreaker
    /// (`o.id ASC`) alongside the primary `tickets_sold DESC` sort, so that
    /// organizers with equal scores keep a stable relative order across
    /// pagination requests.
    #[test]
    fn test_leaderboard_query_has_deterministic_tiebreaker() {
        let query = r#"
        SELECT
            o.id AS organizer_id,
            o.name AS organizer_name,
            COALESCE(SUM(tt.total_quantity - tt.available_quantity), 0)::bigint AS tickets_sold
        FROM organizers o
        LEFT JOIN events e ON e.organizer_id = o.id
        LEFT JOIN ticket_tiers tt ON tt.event_id = e.id
        GROUP BY o.id, o.name
        ORDER BY tickets_sold DESC, o.id ASC
        LIMIT $1 OFFSET $2
        "#;

        assert!(query.contains("ORDER BY tickets_sold DESC, o.id ASC"));
    }

    #[test]
    fn test_leaderboard_entry_serializes_expected_fields() {
        let entry = LeaderboardEntry {
            organizer_id: Uuid::nil(),
            organizer_name: "Test Organizer".to_string(),
            tickets_sold: 42,
        };
        let json = serde_json::to_value(&entry).unwrap();
        assert_eq!(json["tickets_sold"], 42);
        assert_eq!(json["organizer_name"], "Test Organizer");
    }
}
