//! # Leaderboard Handler
//!
//! Provides the leaderboard endpoint that ranks organizers by tickets sold.
//! Supports `all_time`, `monthly`, and `weekly` timeframes.

use axum::{extract::{Query, State}, response::{IntoResponse, Response}};
use serde::{Deserialize, Serialize};
use sqlx::PgPool;

use crate::utils::error::AppError;
use crate::utils::response::success;

/// Supported timeframe values for the leaderboard query.
#[derive(Debug, Deserialize, Default)]
#[serde(rename_all = "snake_case")]
pub enum Timeframe {
    #[default]
    AllTime,
    Monthly,
    Weekly,
}

/// Query parameters accepted by the leaderboard endpoint.
#[derive(Debug, Deserialize)]
pub struct LeaderboardParams {
    /// Ranking timeframe: `all_time` (default), `monthly`, or `weekly`.
    #[serde(default)]
    pub timeframe: Timeframe,
}

/// A single entry in the leaderboard response.
#[derive(Debug, Serialize, sqlx::FromRow)]
pub struct LeaderboardEntry {
    /// Rank position (1-based).
    pub rank: i64,
    /// Organizer name.
    pub organizer_name: String,
    /// Total tickets sold within the requested timeframe.
    pub tickets_sold: i64,
}

/// GET `/api/v1/leaderboard` – Rank organizers by tickets sold.
///
/// # Query Parameters
/// - `timeframe` (optional): `all_time` (default), `monthly`, `weekly`
///
/// # Response
/// Returns a JSON array of organizers ordered by tickets sold descending,
/// each entry including their rank and score.
pub async fn get_leaderboard(
    State(pool): State<PgPool>,
    Query(params): Query<LeaderboardParams>,
) -> Response {
    let time_filter = match params.timeframe {
        Timeframe::AllTime => "TRUE",
        Timeframe::Monthly => "t.created_at >= NOW() - INTERVAL '30 days'",
        Timeframe::Weekly => "t.created_at >= NOW() - INTERVAL '7 days'",
    };

    let query = format!(
        r#"
        SELECT
            ROW_NUMBER() OVER (ORDER BY COUNT(t.id) DESC) AS rank,
            o.name AS organizer_name,
            COUNT(t.id) AS tickets_sold
        FROM organizers o
        JOIN events e ON e.organizer_id = o.id
        JOIN ticket_tiers tt ON tt.event_id = e.id
        JOIN tickets t ON t.ticket_tier_id = tt.id
        WHERE {time_filter}
        GROUP BY o.id, o.name
        ORDER BY tickets_sold DESC
        "#
    );

    match sqlx::query_as::<_, LeaderboardEntry>(&query)
        .fetch_all(&pool)
        .await
    {
        Ok(entries) => success(entries, "Leaderboard retrieved successfully").into_response(),
        Err(e) => {
            tracing::error!("Failed to fetch leaderboard: {:?}", e);
            AppError::DatabaseError(e).into_response()
        }
    }
}
