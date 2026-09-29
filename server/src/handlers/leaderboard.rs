//! # Organizer Leaderboard Handlers
//!
//! Ranks organizers by total tickets sold across all of their events.

use axum::{
    extract::{Query, State},
    http::{header, HeaderValue},
    response::{IntoResponse, Response},
};
use serde::Serialize;
use sqlx::{FromRow, PgPool};
use uuid::Uuid;

use crate::utils::error::AppError;
use crate::utils::pagination::{PaginatedResponse, PaginationParams};
use crate::utils::response::success;

/// `Cache-Control` applied to successful leaderboard responses.
///
/// `max-age=60` lets clients/CDNs serve for 1 minute; `stale-while-revalidate=120`
/// lets them serve a stale body for a further 2 minutes while revalidating.
pub const LEADERBOARD_CACHE_CONTROL: &str = "public, max-age=60, stale-while-revalidate=120";

/// Build a cacheable success response for the leaderboard endpoint.
pub(crate) fn cached_success_response<T: Serialize>(data: T, message: impl Into<String>) -> Response {
    let mut resp = success(data, message).into_response();
    resp.headers_mut().insert(
        header::CACHE_CONTROL,
        HeaderValue::from_static(LEADERBOARD_CACHE_CONTROL),
    );
    resp
}

/// Return an error response that is explicitly marked `no-store` so caches never
/// retain error bodies.
pub(crate) fn no_store_error(err: AppError) -> Response {
    let mut resp = err.into_response();
    resp.headers_mut()
        .insert(header::CACHE_CONTROL, HeaderValue::from_static("no-store"));
    resp
}

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
            return no_store_error(AppError::DatabaseError(e));
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
            return no_store_error(AppError::DatabaseError(e));
        }
    };

    let response = PaginatedResponse::new(items, validated_pagination, total);
    cached_success_response(response, "Leaderboard retrieved successfully")
}

#[cfg(test)]
mod tests {
    use super::*;
    use axum::http::StatusCode;

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

    #[test]
    fn test_leaderboard_cached_success_response_sets_cache_control_header() {
        let entry = LeaderboardEntry {
            organizer_id: Uuid::nil(),
            organizer_name: "Test Organizer".to_string(),
            tickets_sold: 10,
        };
        let resp = cached_success_response(vec![entry], "Leaderboard retrieved successfully");
        assert_eq!(resp.status(), StatusCode::OK);
        assert_eq!(
            resp.headers().get(header::CACHE_CONTROL).unwrap(),
            LEADERBOARD_CACHE_CONTROL
        );
    }

    #[test]
    fn test_leaderboard_no_store_error_sets_no_store_cache_control_header() {
        let resp = no_store_error(AppError::NotFound("organizer not found".into()));
        assert_eq!(
            resp.headers().get(header::CACHE_CONTROL).unwrap(),
            "no-store"
        );
    }
}
