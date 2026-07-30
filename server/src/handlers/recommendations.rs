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
use serde::{Deserialize, Serialize};
use sqlx::PgPool;
use time::OffsetDateTime;
use uuid::Uuid;

use crate::{
    errors::AppError,
    middleware::auth::AuthUser,
};

// ── Request & Response types ──────────────────────────────────────────────────

#[derive(Debug, Deserialize)]
pub struct RecommendQuery {
    /// Maximum results to return (default: 12, max: 24)
    #[serde(default = "default_limit")]
    pub limit: i64,
}

fn default_limit() -> i64 {
    12
}

#[derive(Debug, Serialize, sqlx::FromRow)]
pub struct RecommendedEvent {
    pub id: Uuid,
    pub title: String,
    pub slug: String,
    pub description: Option<String>,
    pub start_time: OffsetDateTime,
    pub end_time: OffsetDateTime,
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

// ── Handler ───────────────────────────────────────────────────────────────────

pub async fn get_recommended_events(
    State(pool): State<PgPool>,
    Extension(auth_user): Extension<AuthUser>,
    Query(params): Query<RecommendQuery>,
) -> Result<impl IntoResponse, AppError> {
    let limit = params.limit.clamp(1, 24);
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
    let based_on_categories: Vec<String> = user_categories.iter().map(|(_, n)| n.clone()).collect();

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

    Ok((
        StatusCode::OK,
        Json(RecommendationsResponse {
            events,
            personalised,
            based_on_categories,
        }),
    ))
}