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
pub(crate) fn cached_success_response<T: Serialize>(
    data: T,
    message: impl Into<String>,
) -> Response {
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

/// SQL query used by [`get_leaderboard`] to rank organizers by total tickets
/// sold with a deterministic `o.id ASC` tiebreaker.
const LEADERBOARD_QUERY: &str = r#"
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

/// List organizers ranked by tickets sold, most first.
///
/// # Endpoint
/// GET `/api/v1/leaderboard`
///
/// # Query parameters
/// - `page` (default 1) and `page_size` (default 20, max 100).
/// - `count` (default `true`): pass `count=false` to skip the
///   `SELECT COUNT(*) FROM organizers` query. `meta.total` is then omitted and
///   `meta.has_more` is inferred from whether the page came back full.
///
/// Ties in `tickets_sold` are broken by `organizer_id` so that the ordering
/// — and therefore pagination — is deterministic across requests, even when
/// multiple organizers share the same score.
#[utoipa::path(
    get,
    path = "/leaderboard",
    params(
        ("page" = Option<u32>, Query, description = "Page number, 1-indexed (default 1)"),
        ("page_size" = Option<u32>, Query, description = "Items per page (1-100, default 20)"),
        ("count" = Option<bool>, Query, description = "Run the total count query (default true). With `count=false` the count is skipped and `meta.total` is omitted"),
    ),
    responses(
        (status = 200, description = "Organizer leaderboard retrieved"),
        (status = 500, description = "Database error")
    ),
    tag = "Events"
)]
pub async fn get_leaderboard(
    State(pool): State<PgPool>,
    Query(pagination): Query<PaginationParams>,
) -> Response {
    let validated_pagination = pagination.validate();

    let total = match count_organizers(&pool, validated_pagination.include_count).await {
        Ok(count) => count,
        Err(e) => {
            tracing::error!("Failed to count organizers: {:?}", e);
            return no_store_error(AppError::DatabaseError(e));
        }
    };

    let items = match sqlx::query_as::<_, LeaderboardEntry>(LEADERBOARD_QUERY)
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

    let response = PaginatedResponse::new_with_optional_total(items, validated_pagination, total);
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
        assert!(LEADERBOARD_QUERY.contains("ORDER BY tickets_sold DESC, o.id ASC"));
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

    fn unreachable_pool() -> PgPool {
        sqlx::postgres::PgPoolOptions::new()
            .acquire_timeout(std::time::Duration::from_millis(250))
            .connect_lazy("postgres://agora:agora@127.0.0.1:1/agora")
            .expect("lazy pool should be constructible")
    }

    /// With `count=false` the count query must not run at all: against a pool
    /// that can never connect, the helper still returns `Ok(None)`.
    #[tokio::test]
    async fn test_count_organizers_skips_query_when_count_false() {
        let total = count_organizers(&unreachable_pool(), false).await;
        assert!(matches!(total, Ok(None)), "got {total:?}");
    }

    /// With the default (`count=true`) the count query still runs, so an
    /// unreachable database surfaces as an error.
    #[tokio::test]
    async fn test_count_organizers_runs_query_by_default() {
        assert!(count_organizers(&unreachable_pool(), true).await.is_err());
    }

    /// `count=false` deserializes into `include_count = false`; omitting it keeps `true`.
    #[test]
    fn test_count_query_param_maps_to_include_count() {
        let off: PaginationParams = serde_json::from_str(r#"{"count":false}"#).unwrap();
        assert!(!off.validate().include_count);
        let default: PaginationParams = serde_json::from_str("{}").unwrap();
        assert!(default.validate().include_count);
    }

    /// End to end against a real database when `DATABASE_URL` is set (skipped otherwise):
    /// `count=false` omits `meta.total`; the default still returns it.
    #[tokio::test]
    async fn test_leaderboard_count_param_against_database() {
        use axum::{body::Body, http::Request, routing::get, Router};
        use tower::ServiceExt;
        let Ok(url) = std::env::var("DATABASE_URL") else {
            eprintln!(
                "skipping test_leaderboard_count_param_against_database: DATABASE_URL not set"
            );
            return;
        };
        let pool = PgPool::connect(&url)
            .await
            .expect("connect to DATABASE_URL");
        let expected: i64 = sqlx::query_scalar("SELECT COUNT(*) FROM organizers")
            .fetch_one(&pool)
            .await
            .unwrap();
        let app = Router::new()
            .route("/leaderboard", get(get_leaderboard))
            .with_state(pool);
        let fetch = |uri: &'static str| {
            let app = app.clone();
            async move {
                let resp = app
                    .oneshot(Request::builder().uri(uri).body(Body::empty()).unwrap())
                    .await
                    .unwrap();
                assert_eq!(resp.status(), StatusCode::OK);
                let bytes = axum::body::to_bytes(resp.into_body(), usize::MAX)
                    .await
                    .unwrap();
                serde_json::from_slice::<serde_json::Value>(&bytes).unwrap()
            }
        };
        let skipped = fetch("/leaderboard?count=false&page_size=1").await;
        assert!(skipped["data"]["meta"].get("total").is_none(), "{skipped}");
        assert_eq!(skipped["data"]["meta"]["page_size"], 1);
        // No count was taken, so the page-number metadata has no total to report.
        assert_eq!(skipped["data"]["pagination"]["total_items"], 0, "{skipped}");
        // Without a total, `has_more` comes from whether the page was full.
        assert_eq!(
            skipped["data"]["meta"]["has_more"],
            expected >= 1,
            "{skipped}"
        );
        let counted = fetch("/leaderboard?page_size=1").await;
        assert_eq!(counted["data"]["meta"]["total"], expected);
        assert_eq!(counted["data"]["pagination"]["total_items"], expected);
    }
}
