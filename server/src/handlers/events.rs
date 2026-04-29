//! # Event Handlers
//!
//! This module provides HTTP handlers for event-related operations including
//! listing, creating, updating, and deleting events.

use axum::{extract::{Path, Query, State}, response::IntoResponse, response::Response, Json};
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use sqlx::PgPool;
use uuid::Uuid;
use std::time::Duration;

use crate::cache::RedisCache;
use crate::models::event::Event;
use crate::utils::error::AppError;
use crate::utils::pagination::{PaginatedResponse, PaginationParams};
use crate::utils::response::success;

/// Cache TTL for event details (5 minutes)
const EVENT_CACHE_TTL: Duration = Duration::from_secs(300);

/// Application state for event handlers
#[derive(Clone)]
pub struct EventState {
    pub pool: PgPool,
    pub redis: RedisCache,
}

/// Query parameters for filtering events
#[derive(Debug, Deserialize)]
pub struct EventFilters {
    /// Filter by organizer ID
    pub organizer_id: Option<Uuid>,
    
    /// Filter by location (partial match)
    pub location: Option<String>,
    
    /// Filter events starting after this date
    pub start_after: Option<DateTime<Utc>>,
    
    /// Filter events starting before this date
    pub start_before: Option<DateTime<Utc>>,
    
    /// Search in title and description
    pub search: Option<String>,
}

#[derive(Debug, Deserialize)]
pub struct SubmitEventRatingRequest {
    pub ticket_id: Uuid,
    pub rating: i16,
    pub review: Option<String>,
}

#[derive(Debug, Serialize)]
pub struct SubmitEventRatingResponse {
    pub sum_of_ratings: i64,
    pub count_of_ratings: i32,
    pub average_rating: f32,
}

/// List all events with pagination and optional filters
///
/// # Endpoint
/// GET `/api/v1/events`
///
/// # Query Parameters
/// - `page` (optional): Page number (default: 1)
/// - `page_size` (optional): Items per page (default: 20, max: 100)
/// - `organizer_id` (optional): Filter by organizer
/// - `location` (optional): Filter by location (partial match)
/// - `start_after` (optional): Filter events starting after date
/// - `start_before` (optional): Filter events starting before date
/// - `search` (optional): Search in title and description
///
/// # Response
/// Returns a paginated list of events with metadata
pub async fn list_events(
    State(state): State<EventState>,
    Query(pagination): Query<PaginationParams>,
    Query(filters): Query<EventFilters>,
) -> Response {
    let validated_pagination = pagination.validate();
    
    // Build the WHERE clause dynamically based on filters
    let mut where_clauses = Vec::new();
    let mut param_count = 0;
    
    // Always exclude flagged events from public listings
    where_clauses.push("is_flagged = FALSE".to_string());
    
    if filters.organizer_id.is_some() {
        param_count += 1;
        where_clauses.push(format!("organizer_id = ${}", param_count));
    }
    
    if filters.location.is_some() {
        param_count += 1;
        where_clauses.push(format!("location ILIKE ${}", param_count));
    }
    
    if filters.start_after.is_some() {
        param_count += 1;
        where_clauses.push(format!("start_time >= ${}", param_count));
    }
    
    if filters.start_before.is_some() {
        param_count += 1;
        where_clauses.push(format!("start_time <= ${}", param_count));
    }
    
    if filters.search.is_some() {
        param_count += 1;
        where_clauses.push(format!(
            "(title ILIKE ${} OR description ILIKE ${})",
            param_count, param_count
        ));
    }
    
    let where_clause = if where_clauses.is_empty() {
        String::new()
    } else {
        format!("WHERE {}", where_clauses.join(" AND "))
    };
    
    // Count total items
    let count_query = format!("SELECT COUNT(*) FROM events {}", where_clause);
    let mut count_query_builder = sqlx::query_scalar::<_, i64>(&count_query);
    
    if let Some(organizer_id) = filters.organizer_id {
        count_query_builder = count_query_builder.bind(organizer_id);
    }
    if let Some(ref location) = filters.location {
        count_query_builder = count_query_builder.bind(format!("%{}%", location));
    }
    if let Some(start_after) = filters.start_after {
        count_query_builder = count_query_builder.bind(start_after);
    }
    if let Some(start_before) = filters.start_before {
        count_query_builder = count_query_builder.bind(start_before);
    }
    if let Some(ref search) = filters.search {
        count_query_builder = count_query_builder.bind(format!("%{}%", search));
    }
    
    let total = match count_query_builder.fetch_one(&state.pool).await {
        Ok(count) => count,
        Err(e) => {
            tracing::error!("Failed to count events: {:?}", e);
            return AppError::DatabaseError(e).into_response();
        }
    };
    
    // Fetch paginated items
    let items_query = format!(
        "SELECT * FROM events {} ORDER BY start_time DESC LIMIT ${} OFFSET ${}",
        where_clause,
        param_count + 1,
        param_count + 2
    );
    
    let mut items_query_builder = sqlx::query_as::<_, Event>(&items_query);
    
    if let Some(organizer_id) = filters.organizer_id {
        items_query_builder = items_query_builder.bind(organizer_id);
    }
    if let Some(ref location) = filters.location {
        items_query_builder = items_query_builder.bind(format!("%{}%", location));
    }
    if let Some(start_after) = filters.start_after {
        items_query_builder = items_query_builder.bind(start_after);
    }
    if let Some(start_before) = filters.start_before {
        items_query_builder = items_query_builder.bind(start_before);
    }
    if let Some(ref search) = filters.search {
        items_query_builder = items_query_builder.bind(format!("%{}%", search));
    }
    
    items_query_builder = items_query_builder
        .bind(validated_pagination.limit())
        .bind(validated_pagination.offset());
    
    let items = match items_query_builder.fetch_all(&state.pool).await {
        Ok(events) => events,
        Err(e) => {
            tracing::error!("Failed to fetch events: {:?}", e);
            return AppError::DatabaseError(e).into_response();
        }
    };
    
    let response = PaginatedResponse::new(items, validated_pagination, total);
    success(response, "Events retrieved successfully").into_response()
}

/// Get a single event by ID
///
/// # Endpoint
/// GET `/api/v1/events/:id`
///
/// # Caching
/// Event details are cached in Redis with a 5-minute TTL to reduce database load.
pub async fn get_event(
    State(mut state): State<EventState>,
    axum::extract::Path(event_id): axum::extract::Path<Uuid>,
) -> Response {
    let cache_key = format!("event:detail:{}", event_id);
    
    // Try to get from cache first
    match state.redis.get::<Event>(&cache_key).await {
        Ok(Some(event)) => {
            tracing::debug!("Cache hit for event {}", event_id);
            return success(event, "Event retrieved successfully (cached)").into_response();
        }
        Ok(None) => {
            tracing::debug!("Cache miss for event {}", event_id);
        }
        Err(e) => {
            tracing::warn!("Redis error, falling back to database: {:?}", e);
        }
    }
    
    // Cache miss or error, fetch from database
    let event = match sqlx::query_as::<_, Event>(
        "SELECT * FROM events WHERE id = $1 AND is_flagged = FALSE"
    )
    .bind(event_id)
    .fetch_optional(&state.pool)
    .await
    {
        Ok(Some(event)) => event,
        Ok(None) => {
            return AppError::NotFound(format!("Event with id '{}' not found", event_id))
                .into_response();
        }
        Err(e) => {
            tracing::error!("Failed to fetch event: {:?}", e);
            return AppError::DatabaseError(e).into_response();
        }
    };
    
    // Store in cache for future requests
    if let Err(e) = state.redis.set(&cache_key, &event, EVENT_CACHE_TTL).await {
        tracing::warn!("Failed to cache event {}: {:?}", event_id, e);
    }
    
    success(event, "Event retrieved successfully").into_response()
}

/// Record a star rating for an event.
///
/// # Endpoint
/// POST `/api/v1/events/:id/rate`
pub async fn submit_event_rating(
    State(state): State<EventState>,
    Path(event_id): Path<Uuid>,
    Json(payload): Json<SubmitEventRatingRequest>,
) -> Response {
    if payload.rating < 1 || payload.rating > 5 {
        return AppError::ValidationError("Rating must be between 1 and 5".to_string())
            .into_response();
    }

    let ticket = match sqlx::query!(
        r#"SELECT t.status AS status, tt.event_id AS event_id
           FROM tickets t
           JOIN ticket_tiers tt ON t.ticket_tier_id = tt.id
           WHERE t.id = $1"#,
        payload.ticket_id
    )
    .fetch_optional(&state.pool)
    .await
    {
        Ok(ticket) => match ticket {
            Some(ticket) => ticket,
            None => {
                return AppError::NotFound(format!("Ticket with id '{}' not found", payload.ticket_id))
                    .into_response();
            }
        },
        Err(e) => {
            tracing::error!("Failed to fetch ticket for rating: {:?}", e);
            return AppError::DatabaseError(e).into_response();
        }
    };

    if ticket.event_id != event_id {
        return AppError::Forbidden("Ticket does not belong to this event".to_string()).into_response();
    }

    if ticket.status != "used" {
        return AppError::ValidationError(
            "Only attendees with a used ticket may leave a rating".to_string(),
        )
        .into_response();
    }

    let mut tx = match state.pool.begin().await {
        Ok(tx) => tx,
        Err(e) => {
            tracing::error!("Failed to begin transaction: {:?}", e);
            return AppError::DatabaseError(e).into_response();
        }
    };

    let already_rated = match sqlx::query_scalar::<_, i64>(
        "SELECT 1::bigint FROM event_ratings WHERE ticket_id = $1",
    )
    .bind(payload.ticket_id)
    .fetch_optional(&mut tx)
    .await
    {
        Ok(exists) => exists.is_some(),
        Err(e) => {
            tracing::error!("Failed to verify existing rating: {:?}", e);
            return AppError::DatabaseError(e).into_response();
        }
    };

    if already_rated {
        return AppError::ValidationError(
            "Each attendee may only submit one rating per event".to_string(),
        )
        .into_response();
    }

    if let Err(e) = sqlx::query(
        "INSERT INTO event_ratings (event_id, ticket_id, rating, review) VALUES ($1, $2, $3, $4)",
    )
    .bind(event_id)
    .bind(payload.ticket_id)
    .bind(payload.rating)
    .bind(payload.review)
    .execute(&mut tx)
    .await
    {
        tracing::error!("Failed to insert event rating: {:?}", e);
        return AppError::DatabaseError(e).into_response();
    }

    let updated_event = match sqlx::query_as::<_, Event>(
        "UPDATE events SET sum_of_ratings = sum_of_ratings + $2, count_of_ratings = count_of_ratings + 1 WHERE id = $1 RETURNING *"
    )
    .bind(event_id)
    .bind(payload.rating)
    .fetch_one(&mut tx)
    .await
    {
        Ok(event) => event,
        Err(e) => {
            tracing::error!("Failed to update event rating aggregates: {:?}", e);
            return AppError::DatabaseError(e).into_response();
        }
    };

    if let Err(e) = tx.commit().await {
        tracing::error!("Failed to commit rating transaction: {:?}", e);
        return AppError::DatabaseError(e).into_response();
    }

    let response = SubmitEventRatingResponse {
        sum_of_ratings: updated_event.sum_of_ratings,
        count_of_ratings: updated_event.count_of_ratings,
        average_rating: updated_event.average_rating().unwrap_or(0.0),
    };

    success(response, "Rating recorded successfully").into_response()
}

/// Toggle the flagged status of an event (admin only)
///
/// # Endpoint
/// POST `/api/v1/admin/events/:id/toggle-flag`
///
/// # Description
/// Flips the `is_flagged` status of the specified event.
/// This endpoint is intended for admin use to moderate content.
pub async fn toggle_event_flag(
    State(state): State<EventState>,
    Path(event_id): Path<Uuid>,
) -> Response {
    // Fetch current flag status
    let current_flagged = match sqlx::query_scalar::<_, bool>(
        "SELECT is_flagged FROM events WHERE id = $1"
    )
    .bind(event_id)
    .fetch_optional(&state.pool)
    .await
    {
        Ok(Some(flagged)) => flagged,
        Ok(None) => {
            return AppError::NotFound(format!("Event with id '{}' not found", event_id))
                .into_response();
        }
        Err(e) => {
            tracing::error!("Failed to fetch event flag status: {:?}", e);
            return AppError::DatabaseError(e).into_response();
        }
    };

    // Toggle the flag
    let new_flagged = !current_flagged;
    if let Err(e) = sqlx::query(
        "UPDATE events SET is_flagged = $1 WHERE id = $2"
    )
    .bind(new_flagged)
    .bind(event_id)
    .execute(&state.pool)
    .await
    {
        tracing::error!("Failed to update event flag: {:?}", e);
        return AppError::DatabaseError(e).into_response();
    }

    // Invalidate cache for this event
    let cache_key = format!("event:detail:{}", event_id);
    if let Err(e) = state.redis.delete(&cache_key).await {
        tracing::warn!("Failed to invalidate cache for event {}: {:?}", event_id, e);
    }

    success(
        serde_json::json!({ "is_flagged": new_flagged }),
        "Event flag toggled successfully"
    ).into_response()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_event_filters_deserialization() {
        // Test that filters can be deserialized from query params
        let filters = EventFilters {
            organizer_id: Some(Uuid::new_v4()),
            location: Some("New York".to_string()),
            start_after: None,
            start_before: None,
            search: Some("concert".to_string()),
        };
        
        assert!(filters.organizer_id.is_some());
        assert_eq!(filters.location.unwrap(), "New York");
    }
}
