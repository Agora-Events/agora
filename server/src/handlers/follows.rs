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
use crate::notifications::{Notification, NotificationService};
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

/// Maximum followers loaded per keyset-paginated batch (Issue #1798).
const FOLLOWER_NOTIFICATION_BATCH_SIZE: i64 = 500;

/// Notify followers when a followed organiser publishes a new event.
///
/// Followers are streamed in `follower_id` order in bounded batches so popular
/// organizers do not require loading every follower into memory at once.
pub async fn notify_followers_on_new_event(
    pool: &PgPool,
    notifications: &NotificationService,
    organizer_address: &str,
    event_id: uuid::Uuid,
) {
    let subject = "New event from followed organizer".to_string();
    let body = format!("Organizer {organizer_address} published a new event {event_id}");
    let mut last_follower_id: Option<String> = None;

    loop {
        let followers = match sqlx::query_scalar::<_, String>(
            r#"SELECT follower_id FROM user_follows
               WHERE following_id = $1
                 AND ($2::TEXT IS NULL OR follower_id > $2)
               ORDER BY follower_id ASC
               LIMIT $3"#,
        )
        .bind(organizer_address)
        .bind(last_follower_id.as_deref())
        .bind(FOLLOWER_NOTIFICATION_BATCH_SIZE)
        .fetch_all(pool)
        .await
        {
            Ok(v) => v,
            Err(e) => {
                tracing::error!("Failed to fetch followers for notification: {:?}", e);
                return;
            }
        };

        let batch_len = followers.len();
        if batch_len == 0 {
            break;
        }

        for follower in &followers {
            let notification = Notification {
                recipient: follower.clone(),
                subject: subject.clone(),
                body: body.clone(),
            };
            if let Err(e) = notifications.send(&notification).await {
                tracing::warn!(
                    "Failed to notify follower {follower} of new event {event_id}: {:?}",
                    e
                );
            } else {
                tracing::info!("Notified follower {follower} of new event {event_id}");
            }
        }

        if batch_len < FOLLOWER_NOTIFICATION_BATCH_SIZE as usize {
            break;
        }
        last_follower_id = followers.last().cloned();
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::notifications::{NotificationError, NotificationProvider};
    use std::sync::{Arc, Mutex};

    struct RecordingProvider(Arc<Mutex<Vec<String>>>);

    #[async_trait::async_trait]
    impl NotificationProvider for RecordingProvider {
        fn name(&self) -> &'static str {
            "recording"
        }

        async fn send(&self, notification: &Notification) -> Result<(), NotificationError> {
            self.0
                .lock()
                .expect("notification record lock")
                .push(notification.recipient.clone());
            Ok(())
        }
    }

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

    /// One notification per follower, including followers on a second batch.
    #[tokio::test]
    async fn notify_followers_sends_once_per_follower() {
        let Ok(database_url) = std::env::var("TEST_DATABASE_URL") else {
            eprintln!("Skipping PostgreSQL-backed follower notification test: TEST_DATABASE_URL is unset");
            return;
        };
        let pool = sqlx::postgres::PgPoolOptions::new()
            .max_connections(1)
            .connect(&database_url)
            .await
            .expect("connect test database");

        sqlx::query(
            "CREATE TEMP TABLE user_follows (
                follower_id TEXT NOT NULL,
                following_id TEXT NOT NULL,
                created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
                PRIMARY KEY (follower_id, following_id)
             )",
        )
        .execute(&pool)
        .await
        .expect("create temporary user_follows table");

        let organizer = "GORGANIZER";
        let follower_count = FOLLOWER_NOTIFICATION_BATCH_SIZE as usize + 3;
        for i in 0..follower_count {
            sqlx::query("INSERT INTO user_follows (follower_id, following_id) VALUES ($1, $2)")
                .bind(format!("GFOLLOWER-{i:04}"))
                .bind(organizer)
                .execute(&pool)
                .await
                .expect("insert follower");
        }
        sqlx::query("INSERT INTO user_follows (follower_id, following_id) VALUES ($1, $2)")
            .bind("GUNRELATED")
            .bind("GOTHER-ORGANIZER")
            .execute(&pool)
            .await
            .expect("insert unrelated follower");

        let sent = Arc::new(Mutex::new(Vec::new()));
        let mut service = NotificationService::new();
        service.register(RecordingProvider(sent.clone()));

        notify_followers_on_new_event(&pool, &service, organizer, uuid::Uuid::new_v4()).await;

        let recipients = sent.lock().expect("notification record lock");
        assert_eq!(recipients.len(), follower_count);
        let unique: std::collections::HashSet<_> = recipients.iter().collect();
        assert_eq!(unique.len(), follower_count);
        assert!(recipients.iter().all(|r| r.starts_with("GFOLLOWER-")));
    }
}
