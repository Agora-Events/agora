// server/src/handlers/recommendations.rs
//
// GET /api/v1/recommendations/events
//
// Returns up to 12 personalised events based on the authenticated
// user's 3 most recent confirmed ticket purchases (category overlap).
// Falls back to "upcoming popular events" when the user has no history.

use axum::{
    extract::{Extension, Query, State},
    http::StatusCode,
    response::IntoResponse,
    Json,
};
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use sqlx::PgPool;
use uuid::Uuid;

use crate::utils::error::AppError;

/// Authenticated user extracted from session / token.
#[derive(Debug, Clone)]
pub struct AuthUser {
    pub user_id: Uuid,
}

// ── Request & Response types ──────────────────────────────────────────────────

pub const DEFAULT_LIMIT: i64 = 12;
pub const MIN_LIMIT: i64 = 1;
pub const MAX_LIMIT: i64 = 24;

#[derive(Debug, Deserialize)]
pub struct RecommendQuery {
    /// Maximum results to return (default: 12, max: 24)
    #[serde(default = "default_limit")]
    pub limit: i64,
}

fn default_limit() -> i64 {
    DEFAULT_LIMIT
}

/// Clamp the limit parameter between `MIN_LIMIT` and `MAX_LIMIT`.
pub fn clamp_limit(limit: i64) -> i64 {
    limit.clamp(MIN_LIMIT, MAX_LIMIT)
}

#[derive(Debug, Clone, Serialize, sqlx::FromRow)]
pub struct RecommendedEvent {
    pub id: Uuid,
    pub title: String,
    pub slug: String,
    pub description: Option<String>,
    pub start_time: DateTime<Utc>,
    pub end_time: DateTime<Utc>,
    pub location: Option<String>,
    pub banner_url: Option<String>,
    pub category_id: Uuid,
    pub category_name: String,
    pub organizer_id: Uuid,
    pub organizer_name: String,
    pub organizer_avatar: Option<String>,
    pub min_price: Option<f64>,
    pub tickets_remaining: i64,
    /// How many of the user's recent categories matched (personalisation signal)
    pub relevance_score: i64,
}

#[derive(Debug, Serialize)]
pub struct RecommendationsResponse {
    pub events: Vec<RecommendedEvent>,
    /// True when results are personalised; false = popularity fallback
    pub personalised: bool,
    /// Category names that drove the recommendations
    pub based_on_categories: Vec<String>,
}

/// Pure helper to assemble the recommendations response DTO.
pub fn build_recommendations_response(
    events: Vec<RecommendedEvent>,
    user_categories: &[(Uuid, String)],
) -> RecommendationsResponse {
    let personalised = !user_categories.is_empty();
    let based_on_categories: Vec<String> = user_categories.iter().map(|(_, n)| n.clone()).collect();
    RecommendationsResponse {
        events,
        personalised,
        based_on_categories,
    }
}

// ── Handler ───────────────────────────────────────────────────────────────────

pub async fn get_recommended_events(
    State(pool): State<PgPool>,
    Extension(auth_user): Extension<AuthUser>,
    Query(params): Query<RecommendQuery>,
) -> Result<impl IntoResponse, AppError> {
    let limit = clamp_limit(params.limit);
    let user_id = auth_user.user_id;

    // 1. Discover the categories from the user's last 3 purchases
    let user_categories: Vec<(Uuid, String)> = sqlx::query_as(
        r#"
        SELECT DISTINCT e.category_id, c.name
        FROM   tickets t
        JOIN   events  e ON e.id = t.event_id
        JOIN   categories c ON c.id = e.category_id
        WHERE  t.user_id = $1
          AND  t.status  = 'confirmed'
        ORDER  BY MAX(t.created_at) DESC
        LIMIT  3
        "#,
    )
    .bind(user_id)
    .fetch_all(&pool)
    .await?;

    let personalised = !user_categories.is_empty();

    let events: Vec<RecommendedEvent> = if personalised {
        // ── Personalised path ─────────────────────────────────────────────────
        sqlx::query_as(
            r#"
            WITH recent_purchases AS (
                SELECT DISTINCT e.category_id
                FROM   tickets t
                JOIN   events  e ON e.id = t.event_id
                WHERE  t.user_id = $1
                  AND  t.status  = 'confirmed'
                ORDER  BY MAX(t.created_at) DESC
                LIMIT  3
            ),
            scored_events AS (
                SELECT
                    e.id,
                    e.title,
                    e.slug,
                    e.description,
                    e.start_time,
                    e.end_time,
                    e.location,
                    e.banner_url,
                    e.category_id,
                    c.name            AS category_name,
                    e.organizer_id,
                    u.display_name    AS organizer_name,
                    u.avatar_url      AS organizer_avatar,
                    (
                        SELECT MIN(tp.price)
                        FROM   ticket_types tp
                        WHERE  tp.event_id = e.id
                          AND  tp.is_active = TRUE
                    )                 AS min_price,
                    (
                        SELECT COALESCE(SUM(tp.quantity - tp.sold), 0)
                        FROM   ticket_types tp
                        WHERE  tp.event_id = e.id
                          AND  tp.is_active = TRUE
                    )                 AS tickets_remaining,
                    COUNT(rp.category_id) AS relevance_score
                FROM   events  e
                JOIN   categories c ON c.id = e.category_id
                JOIN   users      u ON u.id = e.organizer_id
                JOIN   recent_purchases rp ON rp.category_id = e.category_id
                WHERE  e.status     = 'published'
                  AND  e.start_time > NOW()
                  AND  e.id NOT IN (
                           SELECT t2.event_id
                           FROM   tickets t2
                           WHERE  t2.user_id = $1
                             AND  t2.status  = 'confirmed'
                       )
                GROUP  BY
                    e.id, e.title, e.slug, e.description,
                    e.start_time, e.end_time, e.location,
                    e.banner_url, e.category_id, c.name,
                    e.organizer_id, u.display_name, u.avatar_url
            )
            SELECT * FROM scored_events
            ORDER  BY relevance_score DESC, start_time ASC
            LIMIT  $2
            "#,
        )
        .bind(user_id)
        .bind(limit)
        .fetch_all(&pool)
        .await?
    } else {
        // ── Cold-start fallback: popular upcoming events ───────────────────────
        sqlx::query_as(
            r#"
            SELECT
                e.id,
                e.title,
                e.slug,
                e.description,
                e.start_time,
                e.end_time,
                e.location,
                e.banner_url,
                e.category_id,
                c.name          AS category_name,
                e.organizer_id,
                u.display_name  AS organizer_name,
                u.avatar_url    AS organizer_avatar,
                (
                    SELECT MIN(tp.price)
                    FROM   ticket_types tp
                    WHERE  tp.event_id = e.id
                      AND  tp.is_active = TRUE
                )               AS min_price,
                (
                    SELECT COALESCE(SUM(tp.quantity - tp.sold), 0)
                    FROM   ticket_types tp
                    WHERE  tp.event_id = e.id
                      AND  tp.is_active = TRUE
                )               AS tickets_remaining,
                -- Fallback score: confirmed ticket count as popularity proxy
                COUNT(t.id)     AS relevance_score
            FROM   events  e
            JOIN   categories c ON c.id = e.category_id
            JOIN   users      u ON u.id = e.organizer_id
            LEFT   JOIN tickets t ON t.event_id = e.id
                                  AND t.status = 'confirmed'
            WHERE  e.status     = 'published'
              AND  e.start_time > NOW()
            GROUP  BY
                e.id, e.title, e.slug, e.description,
                e.start_time, e.end_time, e.location,
                e.banner_url, e.category_id, c.name,
                e.organizer_id, u.display_name, u.avatar_url
            ORDER  BY relevance_score DESC, start_time ASC
            LIMIT  $1
            "#,
        )
        .bind(limit)
        .fetch_all(&pool)
        .await?
    };

    let response = build_recommendations_response(events, &user_categories);
    Ok((StatusCode::OK, Json(response)))
}

#[cfg(test)]
mod tests {
    use super::*;
    use axum::extract::Query;
    use axum::http::Uri;

    #[test]
    fn test_query_parameter_defaults_when_missing() {
        let uri = Uri::from_static("/api/v1/recommendations/events");
        let Query(params) = Query::<RecommendQuery>::try_from_uri(&uri).unwrap();
        assert_eq!(params.limit, DEFAULT_LIMIT);
    }

    #[test]
    fn test_query_parameter_custom_limit_parsing() {
        let uri = Uri::from_static("/api/v1/recommendations/events?limit=8");
        let Query(params) = Query::<RecommendQuery>::try_from_uri(&uri).unwrap();
        assert_eq!(params.limit, 8);
    }

    #[test]
    fn test_limit_capped_by_global_max_page_size() {
        assert_eq!(clamp_limit(100), MAX_LIMIT);
        assert_eq!(clamp_limit(24), MAX_LIMIT);
        assert_eq!(clamp_limit(25), MAX_LIMIT);
        assert_eq!(clamp_limit(0), MIN_LIMIT);
        assert_eq!(clamp_limit(-10), MIN_LIMIT);
        assert_eq!(clamp_limit(15), 15);
        assert_eq!(clamp_limit(DEFAULT_LIMIT), 12);
    }

    #[test]
    fn test_new_or_unknown_user_with_no_history_returns_valid_response() {
        // Unknown or new user with no ticket purchase history gets a valid (empty) list, not an error
        let empty_events: Vec<RecommendedEvent> = vec![];
        let no_categories: Vec<(Uuid, String)> = vec![];

        let response = build_recommendations_response(empty_events, &no_categories);
        assert!(!response.personalised);
        assert!(response.based_on_categories.is_empty());
        assert!(response.events.is_empty());
    }

    #[test]
    fn test_user_with_categories_produces_personalised_response() {
        let cat_id = Uuid::new_v4();
        let user_categories = vec![(cat_id, "Tech".to_string())];
        let event = RecommendedEvent {
            id: Uuid::new_v4(),
            title: "Tech Summit".to_string(),
            slug: "tech-summit".to_string(),
            description: Some("Tech conference".to_string()),
            start_time: Utc::now(),
            end_time: Utc::now(),
            location: Some("Lagos".to_string()),
            banner_url: None,
            category_id: cat_id,
            category_name: "Tech".to_string(),
            organizer_id: Uuid::new_v4(),
            organizer_name: "Agora".to_string(),
            organizer_avatar: None,
            min_price: Some(10.0),
            tickets_remaining: 100,
            relevance_score: 1,
        };

        let response = build_recommendations_response(vec![event], &user_categories);
        assert!(response.personalised);
        assert_eq!(response.based_on_categories, vec!["Tech"]);
        assert_eq!(response.events.len(), 1);
        assert_eq!(response.events[0].title, "Tech Summit");
    }
}