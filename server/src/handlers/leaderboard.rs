//! # Leaderboard Handler
//!
//! Provides the leaderboard endpoint that ranks organizers by tickets sold.
//! Supports `all_time`, `monthly`, and `weekly` timeframes.
//! Results are cached in Redis for 5 minutes.

use axum::{
    extract::{Query, State},
    response::{IntoResponse, Response},
};
use serde::{Deserialize, Serialize};
use sqlx::PgPool;
use std::time::Duration;

use crate::cache::RedisCache;
use crate::utils::db_timer::log_if_slow;
use crate::utils::error::AppError;
use crate::utils::response::success;

const LEADERBOARD_CACHE_TTL: Duration = Duration::from_secs(300);

/// Application state for leaderboard handlers.
#[derive(Clone)]
pub struct LeaderboardState {
    pub pool: PgPool,
    pub redis: RedisCache,
}

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
#[derive(Debug, Serialize, Deserialize, sqlx::FromRow)]
pub struct LeaderboardEntry {
    /// Rank position (1-based).
    pub rank: i64,
    /// Organizer name.
    pub organizer_name: String,
    /// Total tickets sold within the requested timeframe.
    pub tickets_sold: i64,
}

fn cache_key(timeframe: &Timeframe) -> &'static str {
    match timeframe {
        Timeframe::AllTime => "leaderboard:top:all_time",
        Timeframe::Monthly => "leaderboard:top:monthly",
        Timeframe::Weekly => "leaderboard:top:weekly",
    }
}

/// GET `/api/v1/leaderboard` – Rank organizers by tickets sold.
///
/// # Query Parameters
/// - `timeframe` (optional): `all_time` (default), `monthly`, `weekly`
///
/// # Response
/// Returns a JSON array of organizers ordered by tickets sold descending,
/// each entry including their rank and score. Results are cached for 5 minutes.
pub async fn get_leaderboard(
    State(mut state): State<LeaderboardState>,
    Query(params): Query<LeaderboardParams>,
) -> Response {
    let key = cache_key(&params.timeframe);

    if let Ok(Some(cached)) = state.redis.get::<Vec<LeaderboardEntry>>(key).await {
        return success(cached, "Leaderboard retrieved successfully").into_response();
    }

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

    let start = std::time::Instant::now();
    let result = sqlx::query_as::<_, LeaderboardEntry>(&query)
        .fetch_all(&state.pool)
        .await;
    log_if_slow("get_leaderboard", start.elapsed());

    match result {
        Ok(entries) => {
            if let Err(e) = state.redis.set(key, &entries, LEADERBOARD_CACHE_TTL).await {
                tracing::warn!("Failed to cache leaderboard: {:?}", e);
            }
            success(entries, "Leaderboard retrieved successfully").into_response()
        }
        Err(e) => {
            tracing::error!("Failed to fetch leaderboard: {:?}", e);
            AppError::DatabaseError(e).into_response()
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_leaderboard_cache_keys_are_distinct() {
        let all_time = cache_key(&Timeframe::AllTime);
        let monthly = cache_key(&Timeframe::Monthly);
        let weekly = cache_key(&Timeframe::Weekly);
        assert_ne!(all_time, monthly);
        assert_ne!(monthly, weekly);
        assert_ne!(all_time, weekly);
    }

    #[test]
    fn test_leaderboard_cache_ttl_is_5_minutes() {
        assert_eq!(LEADERBOARD_CACHE_TTL.as_secs(), 300);
    }

    #[test]
    fn test_leaderboard_entry_serialization() {
        let entry = LeaderboardEntry {
            rank: 1,
            organizer_name: "Agora".to_string(),
            tickets_sold: 500,
        };
        let json = serde_json::to_value(&entry).unwrap();
        assert_eq!(json["rank"], 1);
        assert_eq!(json["organizer_name"], "Agora");
        assert_eq!(json["tickets_sold"], 500);
    }
}
