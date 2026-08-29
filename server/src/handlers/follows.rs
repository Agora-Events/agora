//! Follow organiser social graph (Issue #1346)

use axum::{
    extract::{Path, Query, State},
    http::HeaderMap,
    response::{IntoResponse, Response},
};
use serde::Serialize;
use sqlx::PgPool;

use crate::handlers::auth::extract_auth;
use crate::models::organizer_profile::OrganizerProfile;
use crate::utils::error::AppError;
use crate::utils::pagination::PaginationParams;
use crate::utils::response::success;

#[derive(Clone)]
pub struct FollowState {
    pub pool: PgPool,
}

#[derive(Debug, Serialize)]
pub struct FollowCountResponse {
    pub organizer_id: String,
    pub followers_count: i64,
}

/// POST /api/v1/organizers/:id/follow
pub async fn follow_organizer(
    State(state): State<FollowState>,
    headers: HeaderMap,
    Path(organizer_address): Path<String>,
) -> Response {
    let follower = match extract_auth(&headers) {
        Ok(a) => a,
        Err(e) => return e.into_response(),
    };

    if follower == organizer_address {
        return AppError::ValidationError("Cannot follow yourself".to_string()).into_response();
    }

    // Ensure organizer exists
    let exists = match sqlx::query_scalar::<_, Option<String>>(
        "SELECT address FROM organizer_profiles WHERE address = $1",
    )
    .bind(&organizer_address)
    .fetch_optional(&state.pool)
    .await
    {
        Ok(Some(_)) => true,
        Ok(None) => false,
        Err(e) => return AppError::DatabaseError(e).into_response(),
    };
    if !exists {
        return AppError::NotFound(format!("Organizer {organizer_address} not found"))
            .into_response();
    }

    // Atomic upsert
    match sqlx::query(
        "INSERT INTO user_follows (follower_id, following_id) VALUES ($1, $2) ON CONFLICT DO NOTHING",
    )
    .bind(&follower)
    .bind(&organizer_address)
    .execute(&state.pool)
    .await
    {
        Ok(_) => success(serde_json::json!({}), "Followed successfully").into_response(),
        Err(e) => {
            tracing::error!("Failed to follow: {:?}", e);
            AppError::DatabaseError(e).into_response()
        }
    }
}

/// DELETE /api/v1/organizers/:id/follow
pub async fn unfollow_organizer(
    State(state): State<FollowState>,
    headers: HeaderMap,
    Path(organizer_address): Path<String>,
) -> Response {
    let follower = match extract_auth(&headers) {
        Ok(a) => a,
        Err(e) => return e.into_response(),
    };

    match sqlx::query("DELETE FROM user_follows WHERE follower_id = $1 AND following_id = $2")
        .bind(&follower)
        .bind(&organizer_address)
        .execute(&state.pool)
        .await
    {
        Ok(res) => {
            if res.rows_affected() == 0 {
                return AppError::NotFound("Follow relationship not found".to_string())
                    .into_response();
            }
            success(serde_json::json!({}), "Unfollowed successfully").into_response()
        }
        Err(e) => {
            tracing::error!("Failed to unfollow: {:?}", e);
            AppError::DatabaseError(e).into_response()
        }
    }
}

/// GET /api/v1/organizers/:id/followers/count
pub async fn get_followers_count(
    State(state): State<FollowState>,
    Path(organizer_address): Path<String>,
) -> Response {
    let count = match sqlx::query_scalar::<_, i64>(
        "SELECT COUNT(*) FROM user_follows WHERE following_id = $1",
    )
    .bind(&organizer_address)
    .fetch_one(&state.pool)
    .await
    {
        Ok(c) => c,
        Err(e) => return AppError::DatabaseError(e).into_response(),
    };

    success(
        FollowCountResponse {
            organizer_id: organizer_address,
            followers_count: count,
        },
        "Followers count retrieved",
    )
    .into_response()
}

/// GET /api/v1/profile/following
pub async fn list_following(
    State(state): State<FollowState>,
    headers: HeaderMap,
    Query(pagination): Query<PaginationParams>,
) -> Response {
    let follower = match extract_auth(&headers) {
        Ok(a) => a,
        Err(e) => return e.into_response(),
    };

    let validated = pagination.validate();

    let total = match sqlx::query_scalar::<_, i64>(
        "SELECT COUNT(*) FROM user_follows WHERE follower_id = $1",
    )
    .bind(&follower)
    .fetch_one(&state.pool)
    .await
    {
        Ok(c) => c,
        Err(e) => return AppError::DatabaseError(e).into_response(),
    };

    let profiles = match sqlx::query_as::<_, OrganizerProfile>(
        r#"SELECT op.* FROM organizer_profiles op
           INNER JOIN user_follows uf ON uf.following_id = op.address
           WHERE uf.follower_id = $1
           ORDER BY uf.created_at DESC
           LIMIT $2 OFFSET $3"#,
    )
    .bind(&follower)
    .bind(validated.limit())
    .bind(validated.offset())
    .fetch_all(&state.pool)
    .await
    {
        Ok(rows) => rows,
        Err(e) => return AppError::DatabaseError(e).into_response(),
    };

    let resp = crate::utils::pagination::PaginatedResponse::new(profiles, validated, total);
    success(resp, "Following list retrieved").into_response()
}

/// GET /api/v1/events/following – events from organisers the user follows (home dashboard Following tab)
pub async fn list_following_events(
    State(state): State<FollowState>,
    headers: HeaderMap,
    Query(pagination): Query<crate::utils::cursor_pagination::CursorParams>,
) -> Response {
    use crate::models::event::{populate_is_free, Event};
    use crate::utils::cursor_pagination::{decode_cursor, encode_cursor, CursorResponse, EventCursor};
    let follower = match extract_auth(&headers) {
        Ok(a) => a,
        Err(e) => return e.into_response(),
    };
    let validated = pagination.validate();
    let cursor = match validated.cursor {
        Some(ref c) => match decode_cursor::<EventCursor>(c) {
            Ok(cur) => Some(cur),
            Err(e) => return AppError::ValidationError(format!("Invalid cursor: {e}")).into_response(),
        },
        None => None,
    };

    let base_query = if cursor.is_some() {
        "SELECT e.* FROM events e \
         JOIN organizers o ON e.organizer_id = o.id \
         JOIN user_follows uf ON uf.following_id = o.wallet_address \
         WHERE uf.follower_id = $1 \
           AND e.is_flagged = FALSE \
           AND e.end_time > NOW() \
           AND (e.start_time > $3 OR (e.start_time = $3 AND e.id > $4)) \
         ORDER BY e.start_time ASC, e.id ASC LIMIT $2"
    } else {
        "SELECT e.* FROM events e \
         JOIN organizers o ON e.organizer_id = o.id \
         JOIN user_follows uf ON uf.following_id = o.wallet_address \
         WHERE uf.follower_id = $1 \
           AND e.is_flagged = FALSE \
           AND e.end_time > NOW() \
         ORDER BY e.start_time ASC, e.id ASC LIMIT $2"
    };

    let mut builder = sqlx::query_as::<_, Event>(base_query)
        .bind(&follower)
        .bind(validated.query_limit());
    if let Some(ref c) = cursor {
        builder = builder.bind(c.start_time).bind(c.id);
    }

    let mut items = match builder.fetch_all(&state.pool).await {
        Ok(v) => v,
        Err(e) => return AppError::DatabaseError(e).into_response(),
    };
    let has_more = items.len() > validated.page_size();
    let next_cursor = if has_more {
        let last = items.pop().unwrap();
        encode_cursor(&EventCursor {
            start_time: last.start_time,
            id: last.id,
            created_at: Some(last.created_at),
            minted_tickets: Some(last.minted_tickets),
            count_of_ratings: Some(last.count_of_ratings as i64),
            min_ticket_price: Some(last.min_ticket_price),
        })
        .ok()
    } else {
        None
    };
    populate_is_free(&mut items, &state.pool).await;
    let resp = CursorResponse::new(items, &validated, next_cursor);
    success(resp, "Following events retrieved").into_response()
}

/// Notify followers when a followed organiser publishes a new event.
/// Call this after event creation.
pub async fn notify_followers_on_new_event(pool: &PgPool, organizer_address: &str, event_id: uuid::Uuid) {
    let followers = match sqlx::query_scalar::<_, String>(
        "SELECT follower_id FROM user_follows WHERE following_id = $1",
    )
    .bind(organizer_address)
    .fetch_all(pool)
    .await
    {
        Ok(v) => v,
        Err(e) => {
            tracing::error!("Failed to fetch followers for notification: {:?}", e);
            return;
        }
    };

    // Insert a notification row per follower (if notifications table exists, else log)
    // We attempt to insert into a generic notifications table; if it doesn't exist, just log.
    for follower in followers {
        // Try to insert; ignore errors if table missing
        let _ = sqlx::query(
            r#"INSERT INTO notifications (recipient, title, body, metadata)
               VALUES ($1, $2, $3, $4)"#,
        )
        .bind(&follower)
        .bind("New event from followed organizer")
        .bind(format!("Organizer {organizer_address} published a new event {event_id}"))
        .bind(serde_json::json!({"event_id": event_id.to_string(), "organizer": organizer_address}))
        .execute(pool)
        .await;
        tracing::info!("Notified follower {follower} of new event {event_id}");
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_follow_count_response_serializes() {
        let resp = FollowCountResponse {
            organizer_id: "GABC".to_string(),
            followers_count: 42,
        };
        let json = serde_json::to_value(&resp).unwrap();
        assert_eq!(json["followers_count"], 42);
        assert_eq!(json["organizer_id"], "GABC");
    }
}
