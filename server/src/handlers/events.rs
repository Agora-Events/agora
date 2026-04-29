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
use crate::utils::cursor_pagination::{CursorParams, CursorResponse, EventCursor, encode_cursor, decode_cursor};
use crate::utils::error::AppError;
use crate::utils::response::success;

/// Query parameters for searching events with filters
#[derive(Debug, Deserialize)]
pub struct SearchParams {
    /// Keyword search in title/description
    pub q: Option<String>,
    /// Filter by category ID
    pub category_id: Option<Uuid>,
    /// Minimum ticket price in cents (e.g., 1000 = $10.00)
    pub min_price: Option<i64>,
    /// Maximum ticket price in cents (e.g., 5000 = $50.00)
    pub max_price: Option<i64>,
    /// Events starting after this date
    pub date_from: Option<DateTime<Utc>>,
    /// Events starting before this date
    pub date_to: Option<DateTime<Utc>>,
    /// Page number (default: 1)
    #[serde(default = "default_page")]
    pub page: u32,
    /// Items per page (default: 20, max: 100)
    #[serde(default = "default_page_size")]
    pub page_size: u32,
}

fn default_page() -> u32 {
    1
}

fn default_page_size() -> u32 {
    20
}

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

/// List upcoming events with cursor-based pagination and optional filters.
///
/// # Endpoint
/// GET `/api/v1/events`
///
/// # Query Parameters
/// - `limit` (optional): Items per page (default: 20, max: 100)
/// - `cursor` (optional): Opaque cursor for the next page
/// - `organizer_id` (optional): Filter by organizer
/// - `location` (optional): Filter by location (partial match)
/// - `start_after` (optional): Filter events starting after date
/// - `start_before` (optional): Filter events starting before date
/// - `search` (optional): Search in title and description
///
/// # Response
/// Returns a cursor-paginated list of upcoming events with metadata
pub async fn list_events(
    State(state): State<EventState>,
    Query(pagination): Query<CursorParams>,
    Query(filters): Query<EventFilters>,
) -> Response {
    let validated = pagination.validate();

    // Decode cursor if provided
    let cursor = match validated.cursor {
        Some(ref c) => match decode_cursor::<EventCursor>(c) {
            Ok(c) => Some(c),
            Err(e) => {
                tracing::warn!("Invalid cursor provided: {}", e);
                return AppError::ValidationError(format!("Invalid cursor: {}", e)).into_response();
            }
        },
        None => None,
    };

    // Build the WHERE clause dynamically based on filters
    let mut where_clauses = Vec::new();
    let mut param_count = 0;

    // Only show upcoming (not ended) events
    where_clauses.push("end_time > NOW()".to_string());

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
            "(title ILIKE ${0} OR description ILIKE ${0})",
            param_count
        ));
    }

    // Cursor condition: (start_time, id) > (cursor.start_time, cursor.id)
    if cursor.is_some() {
        param_count += 1;
        let st = param_count;
        param_count += 1;
        let id = param_count;
        where_clauses.push(format!(
            "(start_time > ${st} OR (start_time = ${st} AND id > ${id}))",
            st = st,
            id = id
        ));
    }

    let where_clause = format!("WHERE {}", where_clauses.join(" AND "));

    // Fetch items (limit + 1 to detect has_more)
    let items_query = format!(
        "SELECT * FROM events {} ORDER BY start_time ASC, id ASC LIMIT ${}",
        where_clause,
        param_count + 1
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
    if let Some(ref c) = cursor {
        items_query_builder = items_query_builder.bind(c.start_time);
        items_query_builder = items_query_builder.bind(c.id);
    }

    items_query_builder = items_query_builder.bind(validated.query_limit());

    let mut items = match items_query_builder.fetch_all(&state.pool).await {
        Ok(events) => events,
        Err(e) => {
            tracing::error!("Failed to fetch events: {:?}", e);
            return AppError::DatabaseError(e).into_response();
        }
    };

    // Determine if there are more pages
    let has_more = items.len() > validated.page_size();
    let next_cursor = if has_more {
        // Remove the extra item used for detection
        let last = items.pop().unwrap();
        match encode_cursor(&EventCursor {
            start_time: last.start_time,
            id: last.id,
        }) {
            Ok(c) => Some(c),
            Err(e) => {
                tracing::error!("Failed to encode cursor: {:?}", e);
                return AppError::InternalServerError("Failed to encode cursor".to_string()).into_response();
            }
        }
    } else {
        None
    };

    let response = CursorResponse::new(items, &validated, next_cursor);
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

/// Search events with advanced filters
///
/// # Endpoint
/// GET `/api/v1/events/search`
///
/// # Query Parameters
/// - `q` (optional): Keyword search in title and description
/// - `category_id` (optional): Filter by category UUID
/// - `min_price` (optional): Minimum ticket price in cents
/// - `max_price` (optional): Maximum ticket price in cents
/// - `date_from` (optional): Events starting after this date
/// - `date_to` (optional): Events starting before this date
/// - `page` (optional): Page number (default: 1)
/// - `page_size` (optional): Items per page (default: 20, max: 100)
///
/// # Response
/// Returns a paginated list of events matching the search criteria
pub async fn search_events(
    State(state): State<EventState>,
    Query(params): Query<SearchParams>,
) -> Response {
    let pagination = PaginationParams {
        page: params.page,
        page_size: params.page_size,
    };
    let validated_pagination = pagination.validate();

    // Build dynamic WHERE clause using WHERE 1=1 pattern
    let mut where_clauses = vec!["1=1".to_string()];
    let mut param_count = 0;

    // Keyword search in title and description
    if params.q.is_some() {
        param_count += 1;
        where_clauses.push(format!(
            "(e.title ILIKE ${} OR e.description ILIKE ${})",
            param_count, param_count
        ));
    }

    // Filter by category (requires join with event_categories)
    let category_join = if params.category_id.is_some() {
        param_count += 1;
        where_clauses.push(format!("ec.category_id = ${}", param_count));
        "INNER JOIN event_categories ec ON e.id = ec.event_id"
    } else {
        ""
    };

    // Filter by price range (requires join with ticket_tiers)
    let price_join = if params.min_price.is_some() || params.max_price.is_some() {
        "INNER JOIN ticket_tiers tt ON e.id = tt.event_id"
    } else {
        ""
    };

    if params.min_price.is_some() {
        param_count += 1;
        where_clauses.push(format!("tt.price >= ${}", param_count));
    }

    if params.max_price.is_some() {
        param_count += 1;
        where_clauses.push(format!("tt.price <= ${}", param_count));
    }

    // Filter by date range
    if params.date_from.is_some() {
        param_count += 1;
        where_clauses.push(format!("e.start_time >= ${}", param_count));
    }

    if params.date_to.is_some() {
        param_count += 1;
        where_clauses.push(format!("e.start_time <= ${}", param_count));
    }

    let where_clause = where_clauses.join(" AND ");

    // Count total items with DISTINCT to handle joins
    let count_query = format!(
        "SELECT COUNT(DISTINCT e.id) FROM events e {} {} WHERE {}",
        category_join, price_join, where_clause
    );

    let mut count_query_builder = sqlx::query_scalar::<_, i64>(&count_query);

    if let Some(ref q) = params.q {
        count_query_builder = count_query_builder.bind(format!("%{}%", q));
    }
    if let Some(category_id) = params.category_id {
        count_query_builder = count_query_builder.bind(category_id);
    }
    if let Some(min_price) = params.min_price {
        let min_price_decimal = min_price as f64 / 100.0;
        count_query_builder = count_query_builder.bind(min_price_decimal);
    }
    if let Some(max_price) = params.max_price {
        let max_price_decimal = max_price as f64 / 100.0;
        count_query_builder = count_query_builder.bind(max_price_decimal);
    }
    if let Some(date_from) = params.date_from {
        count_query_builder = count_query_builder.bind(date_from);
    }
    if let Some(date_to) = params.date_to {
        count_query_builder = count_query_builder.bind(date_to);
    }

    let total = match count_query_builder.fetch_one(&state.pool).await {
        Ok(count) => count,
        Err(e) => {
            tracing::error!("Failed to count search results: {:?}", e);
            return AppError::DatabaseError(e).into_response();
        }
    };

    // Fetch paginated items with DISTINCT to handle joins
    let items_query = format!(
        "SELECT DISTINCT e.* FROM events e {} {} WHERE {} ORDER BY e.start_time DESC LIMIT ${} OFFSET ${}",
        category_join,
        price_join,
        where_clause,
        param_count + 1,
        param_count + 2
    );

    let mut items_query_builder = sqlx::query_as::<_, Event>(&items_query);

    if let Some(ref q) = params.q {
        items_query_builder = items_query_builder.bind(format!("%{}%", q));
    }
    if let Some(category_id) = params.category_id {
        items_query_builder = items_query_builder.bind(category_id);
    }
    if let Some(min_price) = params.min_price {
        let min_price_decimal = min_price as f64 / 100.0;
        items_query_builder = items_query_builder.bind(min_price_decimal);
    }
    if let Some(max_price) = params.max_price {
        let max_price_decimal = max_price as f64 / 100.0;
        items_query_builder = items_query_builder.bind(max_price_decimal);
    }
    if let Some(date_from) = params.date_from {
        items_query_builder = items_query_builder.bind(date_from);
    }
    if let Some(date_to) = params.date_to {
        items_query_builder = items_query_builder.bind(date_to);
    }

    items_query_builder = items_query_builder
        .bind(validated_pagination.limit())
        .bind(validated_pagination.offset());

    let items = match items_query_builder.fetch_all(&state.pool).await {
        Ok(events) => events,
        Err(e) => {
            tracing::error!("Failed to fetch search results: {:?}", e);
            return AppError::DatabaseError(e).into_response();
        }
    };

    let response = PaginatedResponse::new(items, validated_pagination, total);
    success(response, "Search results retrieved successfully").into_response()
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
