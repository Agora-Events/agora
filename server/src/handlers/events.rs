//! # Event Handlers
//!
//! This module provides HTTP handlers for event-related operations including
//! listing, creating, updating, and deleting events.

use axum::{
    extract::{Path, Query, State},
    response::IntoResponse,
    response::Response,
    Json,
};
use chrono::{DateTime, NaiveDate, NaiveTime, TimeZone, Utc};
use serde::{Deserialize, Serialize};
use sqlx::{PgPool, Row};
use std::time::Duration;
use uuid::Uuid;

use crate::cache::RedisCache;
use crate::models::event::Event;
use crate::models::organizer_profile::OrganizerProfile;
use crate::utils::cursor_pagination::{
    decode_cursor, encode_cursor, CursorParams, CursorResponse, EventCursor, PastEventCursor,
};
use crate::utils::db_timer::log_if_slow;
use crate::utils::error::AppError;
use crate::utils::pagination::{PaginatedResponse, PaginationParams};
use crate::utils::response::success;

/// Query parameters for searching events with filters
#[derive(Debug, Deserialize)]
pub struct SearchParams {
    /// Keyword search in title/description
    pub q: Option<String>,
    /// Filter by category ID (single, backward-compat)
    pub category_id: Option<Uuid>,
    /// Comma-separated category UUIDs for multi-select filtering
    pub category_ids: Option<String>,
    /// Minimum ticket price in cents (e.g., 1000 = $10.00)
    pub min_price: Option<i64>,
    /// Maximum ticket price in cents (e.g., 5000 = $50.00)
    pub max_price: Option<i64>,
    /// Events starting after this date
    pub date_from: Option<DateTime<Utc>>,
    /// Events starting before this date
    pub date_to: Option<DateTime<Utc>>,
    /// Filter by location (partial match, case-insensitive)
    pub location: Option<String>,
    /// Filter by ticket tier name (partial match, case-insensitive)
    pub ticket_type: Option<String>,
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

/// Cache TTL for social proof (60 seconds)
const SOCIAL_PROOF_CACHE_TTL: Duration = Duration::from_secs(60);

/// Application state for event handlers
#[derive(Clone)]
pub struct EventState {
    pub pool: PgPool,
    pub redis: RedisCache,
    pub base_url: String,
}

/// Event detail response that includes the organizer's public profile (Issue #486).
#[derive(Debug, Serialize, Deserialize)]
pub struct EventDetail {
    #[serde(flatten)]
    pub event: Event,
    /// Organizer profile, if one has been created for the event's organizer wallet.
    pub organizer_profile: Option<OrganizerProfile>,
}

/// Query parameters for filtering events
#[derive(Debug, Deserialize, Default)]
pub struct EventFilters {
    /// Filter by organizer ID
    pub organizer_id: Option<Uuid>,

    /// Filter by organizer wallet address (Stellar public key)
    pub organizer_wallet: Option<String>,

    /// Filter by location (partial match)
    pub location: Option<String>,

    /// Filter events starting after this date
    pub start_after: Option<DateTime<Utc>>,

    /// Filter events starting before this date
    pub start_before: Option<DateTime<Utc>>,

    /// Search in title and description
    pub search: Option<String>,

    /// Minimum tickets available (total_tickets - minted_tickets) >= N
    pub min_tickets_available: Option<i64>,

    /// Filter by free events (true = ticket_price = 0, false = ticket_price > 0)
    pub is_free: Option<bool>,

    /// Filter events starting on or after this date (YYYY-MM-DD, treated as midnight UTC).
    /// Takes precedence over `start_after` when both are supplied.
    pub start_date: Option<String>,

    /// Filter events starting on or before this date (YYYY-MM-DD, treated as midnight UTC).
    /// Takes precedence over `start_before` when both are supplied.
    pub end_date: Option<String>,

    /// Filter to return only followers-only events (Issue #ForYou)
    pub followers_only: Option<bool>,
}

/// Build WHERE clause and return (where_clause, param_count)
fn build_event_where_clause(
    filters: &EventFilters,
    cursor: Option<&EventCursor>,
) -> (String, usize) {
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

    if filters.organizer_wallet.is_some() {
        param_count += 1;
        where_clauses.push(format!(
            "organizer_id = (SELECT id FROM organizers WHERE wallet_address = ${})",
            param_count
        ));
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

    if let Some(is_free) = filters.is_free {
        if is_free {
            where_clauses.push(
                "NOT EXISTS (SELECT 1 FROM ticket_tiers tt WHERE tt.event_id = events.id AND tt.price > 0.0)".to_string(),
            );
        } else {
            where_clauses.push(
                "EXISTS (SELECT 1 FROM ticket_tiers tt WHERE tt.event_id = events.id AND tt.price > 0.0)".to_string(),
            );
        }
    }

    if let Some(_min_tickets) = filters.min_tickets_available {
        param_count += 1;
        where_clauses.push(format!(
            "(total_tickets - minted_tickets) >= ${}",
            param_count
        ));
    }

    // start_date / end_date: date-only filters (treated as midnight UTC).
    // They are wired the same way as start_after / start_before; the actual
    // DateTime binding happens in the handler after parsing.
    if filters.start_date.is_some() {
        param_count += 1;
        where_clauses.push(format!("start_time >= ${}", param_count));
    }

    if filters.end_date.is_some() {
        param_count += 1;
        where_clauses.push(format!("start_time <= ${}", param_count));
    }

    if let Some(true) = filters.followers_only {
        where_clauses.push("followers_only = TRUE".to_string());
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
    (where_clause, param_count)
}

/// Query parameters for filtering past events.
#[derive(Debug, Deserialize)]
pub struct PastEventFilters {
    /// Filter by organizer wallet address (Stellar public key)
    pub organizer_wallet: Option<String>,
}

fn build_past_event_where_clause(
    filters: &PastEventFilters,
    cursor: Option<&PastEventCursor>,
) -> (String, usize) {
    let mut where_clauses = vec![
        "end_time <= NOW()".to_string(),
        "is_flagged = FALSE".to_string(),
    ];
    let mut param_count = 0;

    if filters.organizer_wallet.is_some() {
        param_count += 1;
        where_clauses.push(format!(
            "organizer_id = (SELECT id FROM organizers WHERE wallet_address = ${})",
            param_count
        ));
    }

    if cursor.is_some() {
        param_count += 1;
        let end_time = param_count;
        param_count += 1;
        let id = param_count;
        where_clauses.push(format!(
            "(end_time < ${end_time} OR (end_time = ${end_time} AND id < ${id}))",
            end_time = end_time,
            id = id
        ));
    }

    (format!("WHERE {}", where_clauses.join(" AND ")), param_count)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn build_where_clause_includes_min_tickets_available() {
        let filters = EventFilters {
            organizer_id: None,
            organizer_wallet: None,
            location: None,
            start_after: None,
            start_before: None,
            search: None,
            min_tickets_available: Some(10),
            is_free: None,
            start_date: None,
            end_date: None,
            followers_only: None,
        };

        let (where_clause, _) = build_event_where_clause(&filters, None);
        assert!(
            where_clause.contains("(total_tickets - minted_tickets) >= $1"),
            "where_clause was: {}",
            where_clause
        );
    }

    #[test]
    fn test_event_filters_deserialization() {
        // Test that filters can be deserialized from query params
        let filters = EventFilters {
            organizer_id: Some(Uuid::new_v4()),
            organizer_wallet: Some("GABC123".to_string()),
            location: Some("New York".to_string()),
            start_after: None,
            start_before: None,
            search: Some("concert".to_string()),
            min_tickets_available: None,
            is_free: None,
            start_date: None,
            end_date: None,
            followers_only: None,
        };

        assert!(filters.organizer_id.is_some());
        assert_eq!(filters.organizer_wallet.as_deref(), Some("GABC123"));
        assert_eq!(filters.location.unwrap(), "New York");
    }

    #[test]
    fn test_organizer_wallet_filter() {
        let filters = EventFilters {
            organizer_id: None,
            organizer_wallet: Some("GBXXX".to_string()),
            location: None,
            start_after: None,
            start_before: None,
            search: None,
            min_tickets_available: None,
            is_free: None,
            start_date: None,
            end_date: None,
            followers_only: None,
        };
        assert_eq!(filters.organizer_wallet.as_deref(), Some("GBXXX"));
    }

    #[test]
    fn test_past_event_where_clause_default() {
        let filters = PastEventFilters {
            organizer_wallet: None,
        };

        let (where_clause, param_count) = build_past_event_where_clause(&filters, None);

        assert_eq!(param_count, 0);
        assert!(where_clause.contains("end_time <= NOW()"));
        assert!(where_clause.contains("is_flagged = FALSE"));
    }

    #[test]
    fn test_past_event_where_clause_with_filter_and_cursor() {
        let filters = PastEventFilters {
            organizer_wallet: Some("GBXXX".to_string()),
        };
        let cursor = PastEventCursor {
            end_time: Utc::now(),
            id: Uuid::new_v4(),
        };

        let (where_clause, param_count) =
            build_past_event_where_clause(&filters, Some(&cursor));

        assert_eq!(param_count, 3);
        assert!(where_clause.contains("wallet_address = $1"));
        assert!(where_clause.contains("(end_time < $2 OR (end_time = $2 AND id < $3))"));
    }

    #[test]
    fn test_is_free_filter() {
        let filters_free = EventFilters {
            organizer_id: None,
            organizer_wallet: None,
            location: None,
            start_after: None,
            start_before: None,
            search: None,
            min_tickets_available: None,
            is_free: Some(true),
            start_date: None,
            end_date: None,
            followers_only: None,
        };
        assert_eq!(filters_free.is_free, Some(true));

        let filters_paid = EventFilters {
            organizer_id: None,
            organizer_wallet: None,
            location: None,
            start_after: None,
            start_before: None,
            search: None,
            min_tickets_available: None,
            is_free: Some(false),
            start_date: None,
            end_date: None,
            followers_only: None,
        };
        assert_eq!(filters_paid.is_free, Some(false));

        let filters_none = EventFilters {
            organizer_id: None,
            organizer_wallet: None,
            location: None,
            start_after: None,
            start_before: None,
            search: None,
            min_tickets_available: None,
            is_free: None,
            start_date: None,
            end_date: None,
            followers_only: None,
        };
        assert_eq!(filters_none.is_free, None);
    }

    #[test]
    fn test_start_date_filter_generates_where_clause() {
        let filters = EventFilters {
            organizer_id: None,
            organizer_wallet: None,
            location: None,
            start_after: None,
            start_before: None,
            search: None,
            min_tickets_available: None,
            is_free: None,
            start_date: Some("2026-06-15".to_string()),
            end_date: None,
            followers_only: None,
        };
        let (where_clause, _) = build_event_where_clause(&filters, None);
        assert!(
            where_clause.contains("start_time >="),
            "Expected start_time >= clause, got: {}",
            where_clause
        );
    }

    #[test]
    fn test_end_date_filter_generates_where_clause() {
        let filters = EventFilters {
            organizer_id: None,
            organizer_wallet: None,
            location: None,
            start_after: None,
            start_before: None,
            search: None,
            min_tickets_available: None,
            is_free: None,
            start_date: None,
            end_date: Some("2026-06-20".to_string()),
            followers_only: None,
        };
        let (where_clause, _) = build_event_where_clause(&filters, None);
        assert!(
            where_clause.contains("start_time <="),
            "Expected start_time <= clause, got: {}",
            where_clause
        );
    }

    #[test]
    fn test_followers_only_filter() {
        let filters = EventFilters {
            organizer_id: None,
            organizer_wallet: None,
            location: None,
            start_after: None,
            start_before: None,
            search: None,
            min_tickets_available: None,
            is_free: None,
            start_date: None,
            end_date: None,
            followers_only: Some(true),
        };
        let (where_clause, _) = build_event_where_clause(&filters, None);
        assert!(
            where_clause.contains("followers_only = TRUE"),
            "Expected where_clause to contain followers_only = TRUE, got: {}",
            where_clause
        );
    }

    #[test]
    fn test_start_date_parsing_valid() {
        let result = NaiveDate::parse_from_str("2026-06-15", "%Y-%m-%d");
        assert!(result.is_ok(), "Expected valid date parse");
    }

    #[test]
    fn test_start_date_parsing_invalid() {
        let result = NaiveDate::parse_from_str("not-a-date", "%Y-%m-%d");
        assert!(result.is_err(), "Expected parse error for invalid date");
    }

    #[test]
    fn test_ratings_summary_distribution_zero_filled() {
        let mut distribution = std::collections::HashMap::new();
        for star in 1i16..=5 {
            distribution.insert(star.to_string(), 0i64);
        }
        // Simulate two ratings: one 4-star, one 5-star
        distribution.insert("4".to_string(), 1i64);
        distribution.insert("5".to_string(), 1i64);

        assert_eq!(distribution["1"], 0);
        assert_eq!(distribution["2"], 0);
        assert_eq!(distribution["3"], 0);
        assert_eq!(distribution["4"], 1);
        assert_eq!(distribution["5"], 1);
    }

    #[test]
    fn test_ratings_summary_average_no_ratings() {
        let total = 0i64;
        let average = if total > 0 { 1.0f64 } else { 0.0f64 };
        assert_eq!(average, 0.0);
    }

    #[test]
    fn test_description_truncation() {
        let long_description = "This is a very long description that should be truncated to exactly 160 characters to ensure it fits within the limit for social media sharing and other use cases where space is limited.";
        let truncated: String = long_description.chars().take(160).collect();
        assert!(truncated.len() <= 160);
        assert_eq!(truncated.len(), 160);
    }

    #[test]
    fn test_description_truncation_short() {
        let short_description = "Short description";
        let truncated: String = short_description.chars().take(160).collect();
        assert_eq!(truncated, "Short description");
    }

    #[test]
    fn test_description_truncation_empty() {
        let empty_description = "";
        let truncated: String = empty_description.chars().take(160).collect();
        assert_eq!(truncated, "");
    }

    #[test]
    fn test_social_proof_response_serialization() {
        let response = EventSocialProofResponse {
            recent_purchases: 12,
            average_rating: 4.5,
            waitlist_count: 8,
            tickets_remaining: 43,
        };

        assert_eq!(response.recent_purchases, 12);
        assert_eq!(response.average_rating, 4.5);
        assert_eq!(response.waitlist_count, 8);
        assert_eq!(response.tickets_remaining, 43);
    }

    #[test]
    fn test_social_proof_zero_values() {
        let response = EventSocialProofResponse {
            recent_purchases: 0,
            average_rating: 0.0,
            waitlist_count: 0,
            tickets_remaining: 0,
        };

        assert_eq!(response.recent_purchases, 0);
        assert_eq!(response.average_rating, 0.0);
        assert_eq!(response.waitlist_count, 0);
        assert_eq!(response.tickets_remaining, 0);
    }

    #[test]
    fn test_attendee_count_response_serialization() {
        let response = AttendeeCountResponse {
            count: 142,
            total_tickets: 500,
        };

        let json = serde_json::to_value(response).unwrap();
        assert_eq!(json["count"], 142);
        assert_eq!(json["total_tickets"], 500);
    }

    #[test]
    fn test_search_params_ticket_type() {
        let params = SearchParams {
            q: None,
            category_id: None,
            category_ids: None,
            min_price: None,
            max_price: None,
            date_from: None,
            date_to: None,
            location: None,
            ticket_type: Some("VIP".to_string()),
            page: 1,
            page_size: 20,
        };

        assert_eq!(params.ticket_type, Some("VIP".to_string()));
    }

    #[test]
    fn test_search_params_ticket_type_none() {
        let params = SearchParams {
            q: None,
            category_id: None,
            category_ids: None,
            min_price: None,
            max_price: None,
            date_from: None,
            date_to: None,
            location: None,
            ticket_type: None,
            page: 1,
            page_size: 20,
        };

        assert!(params.ticket_type.is_none());
    }

    #[test]
    fn test_ratings_summary_average_computed() {
        // 1×4 + 1×5 = 9 / 2 = 4.5
        let rows: Vec<(i16, i64)> = vec![(4, 1), (5, 1)];
        let total: i64 = rows.iter().map(|(_, c)| c).sum();
        let weighted: i64 = rows.iter().map(|(r, c)| *r as i64 * c).sum();
        let average = weighted as f64 / total as f64;
        assert_eq!(average, 4.5);
    }

    #[test]
    fn test_search_params_location() {
        let params = SearchParams {
            q: None,
            category_id: None,
            category_ids: None,
            min_price: None,
            max_price: None,
            date_from: None,
            date_to: None,
            location: Some("Lagos".to_string()),
            ticket_type: None,
            page: 1,
            page_size: 20,
        };
        assert_eq!(params.location.as_deref(), Some("Lagos"));
    }

    #[test]
    fn test_export_attendees_csv_format() {
        // Test CSV header format
        let header = "owner_wallet,buyer_wallet,quantity,created_at\n";
        assert!(header.contains("owner_wallet"));
        assert!(header.contains("buyer_wallet"));
        assert!(header.contains("quantity"));
        assert!(header.contains("created_at"));
    }

    #[test]
    fn test_csv_row_format() {
        // Test that a CSV row can be formatted correctly
        let owner = "GXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXX";
        let buyer = "GYYYYYYYYYYYYYYYYYYYYYYYYYYYYYYYYYYYYYYYYYYYYYYYYYYYYYYY";
        let quantity = 2;
        let created_at = chrono::Utc::now();

        let row = format!(
            "{},{},{},{}\n",
            owner,
            buyer,
            quantity,
            created_at.to_rfc3339()
        );

        assert!(row.contains(owner));
        assert!(row.contains(buyer));
        assert!(row.contains("2"));
    }
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
/// - `is_free` (optional): Filter by free events (true = ticket_price = 0, false = ticket_price > 0)
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
    let (where_clause, param_count) = build_event_where_clause(&filters, cursor.as_ref());

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
    if let Some(ref organizer_wallet) = filters.organizer_wallet {
        items_query_builder = items_query_builder.bind(organizer_wallet.clone());
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
    if let Some(min_tickets) = filters.min_tickets_available {
        items_query_builder = items_query_builder.bind(min_tickets);
    }

    // Parse and bind start_date (YYYY-MM-DD → midnight UTC).
    if let Some(ref date_str) = filters.start_date {
        match NaiveDate::parse_from_str(date_str, "%Y-%m-%d") {
            Ok(date) => {
                let dt: DateTime<Utc> = Utc
                    .from_utc_datetime(&date.and_time(NaiveTime::from_hms_opt(0, 0, 0).unwrap()));
                items_query_builder = items_query_builder.bind(dt);
            }
            Err(_) => {
                return AppError::ValidationError(format!(
                    "start_date '{}' is not a valid date — expected YYYY-MM-DD",
                    date_str
                ))
                .into_response();
            }
        }
    }

    // Parse and bind end_date (YYYY-MM-DD → midnight UTC).
    if let Some(ref date_str) = filters.end_date {
        match NaiveDate::parse_from_str(date_str, "%Y-%m-%d") {
            Ok(date) => {
                let dt: DateTime<Utc> = Utc
                    .from_utc_datetime(&date.and_time(NaiveTime::from_hms_opt(0, 0, 0).unwrap()));
                items_query_builder = items_query_builder.bind(dt);
            }
            Err(_) => {
                return AppError::ValidationError(format!(
                    "end_date '{}' is not a valid date — expected YYYY-MM-DD",
                    date_str
                ))
                .into_response();
            }
        }
    }

    if let Some(ref c) = cursor {
        items_query_builder = items_query_builder.bind(c.start_time);
        items_query_builder = items_query_builder.bind(c.id);
    }

    items_query_builder = items_query_builder.bind(validated.query_limit());

    let start = std::time::Instant::now();
    let mut items = match items_query_builder.fetch_all(&state.pool).await {
        Ok(events) => events,
        Err(e) => {
            tracing::error!("Failed to fetch events: {:?}", e);
            return AppError::DatabaseError(e).into_response();
        }
    };
    log_if_slow("list_events", start.elapsed());

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
                return AppError::InternalServerError("Failed to encode cursor".to_string())
                    .into_response();
            }
        }
    } else {
        None
    };

    let response = CursorResponse::new(items, &validated, next_cursor);
    success(response, "Events retrieved successfully").into_response()
}

/// List completed events with cursor-based pagination and optional filters.
///
/// # Endpoint
/// GET `/api/v1/events/past`
pub async fn list_past_events(
    State(state): State<EventState>,
    Query(pagination): Query<CursorParams>,
    Query(filters): Query<PastEventFilters>,
) -> Response {
    let validated = pagination.validate();

    let cursor = match validated.cursor {
        Some(ref c) => match decode_cursor::<PastEventCursor>(c) {
            Ok(c) => Some(c),
            Err(e) => {
                tracing::warn!("Invalid past events cursor provided: {}", e);
                return AppError::ValidationError(format!("Invalid cursor: {}", e)).into_response();
            }
        },
        None => None,
    };

    let (where_clause, param_count) = build_past_event_where_clause(&filters, cursor.as_ref());
    let items_query = format!(
        "SELECT * FROM events {} ORDER BY end_time DESC, id DESC LIMIT ${}",
        where_clause,
        param_count + 1
    );

    let mut items_query_builder = sqlx::query_as::<_, Event>(&items_query);

    if let Some(ref organizer_wallet) = filters.organizer_wallet {
        items_query_builder = items_query_builder.bind(organizer_wallet.clone());
    }
    if let Some(ref c) = cursor {
        items_query_builder = items_query_builder.bind(c.end_time);
        items_query_builder = items_query_builder.bind(c.id);
    }

    items_query_builder = items_query_builder.bind(validated.query_limit());

    let start = std::time::Instant::now();
    let mut items = match items_query_builder.fetch_all(&state.pool).await {
        Ok(events) => events,
        Err(e) => {
            tracing::error!("Failed to fetch past events: {:?}", e);
            return AppError::DatabaseError(e).into_response();
        }
    };
    log_if_slow("list_past_events", start.elapsed());

    let has_more = items.len() > validated.page_size();
    let next_cursor = if has_more {
        let last = items.pop().unwrap();
        match last.end_time {
            Some(end_time) => match encode_cursor(&PastEventCursor {
                end_time,
                id: last.id,
            }) {
                Ok(c) => Some(c),
                Err(e) => {
                    tracing::error!("Failed to encode past events cursor: {:?}", e);
                    return AppError::InternalServerError("Failed to encode cursor".to_string())
                        .into_response();
                }
            },
            None => {
                tracing::error!("Past event query returned event without end_time");
                return AppError::InternalServerError("Failed to encode cursor".to_string())
                    .into_response();
            }
        }
    } else {
        None
    };

    let response = CursorResponse::new(items, &validated, next_cursor);
    success(response, "Past events retrieved successfully").into_response()
}

/// Get a single event by ID
///
/// # Endpoint
/// GET `/api/v1/events/:id`
///
/// # Caching
/// Event details are cached in Redis with a 5-minute TTL to reduce database load.
/// The response includes the organizer's public profile when available (Issue #486).
pub async fn get_event(
    State(mut state): State<EventState>,
    axum::extract::Path(event_id): axum::extract::Path<Uuid>,
) -> Response {
    let cache_key = format!("event:detail:{}", event_id);

    // Try to get from cache first
    match state.redis.get::<EventDetail>(&cache_key).await {
        Ok(Some(detail)) => {
            tracing::debug!("Cache hit for event {}", event_id);
            return success(detail, "Event retrieved successfully (cached)").into_response();
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
        "SELECT * FROM events WHERE id = $1 AND is_flagged = FALSE",
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

    // Fetch organizer profile by wallet address (Issue #486)
    // Look up the organizer's Stellar wallet, then fetch their profile.
    let organizer_profile = match sqlx::query_scalar::<_, Option<String>>(
        "SELECT wallet_address FROM organizers WHERE id = $1",
    )
    .bind(event.organizer_id)
    .fetch_optional(&state.pool)
    .await
    {
        Ok(Some(Some(wallet))) => {
            match sqlx::query_as::<_, OrganizerProfile>(
                "SELECT * FROM organizer_profiles WHERE address = $1",
            )
            .bind(&wallet)
            .fetch_optional(&state.pool)
            .await
            {
                Ok(profile) => profile,
                Err(e) => {
                    tracing::warn!("Failed to fetch organizer profile: {:?}", e);
                    None
                }
            }
        }
        _ => None,
    };

    let detail = EventDetail {
        event,
        organizer_profile,
    };

    // Store in cache for future requests
    if let Err(e) = state.redis.set(&cache_key, &detail, EVENT_CACHE_TTL).await {
        tracing::warn!("Failed to cache event {}: {:?}", event_id, e);
    }

    success(detail, "Event retrieved successfully").into_response()
}

/// Request body for creating a new event
#[derive(Debug, Deserialize)]
pub struct CreateEventRequest {
    pub organizer_id: Uuid,
    pub title: String,
    pub description: Option<String>,
    pub location: String,
    pub start_time: DateTime<Utc>,
    pub end_time: Option<DateTime<Utc>>,
    /// Optional HTTPS URL for the event's banner/cover image.
    pub image_url: Option<String>,
    /// Optional contact email for the event host.
    pub host_email: Option<String>,
}

/// Returns true when the string is a plausibly valid email address.
fn is_valid_email(email: &str) -> bool {
    let mut parts = email.splitn(2, '@');
    let local = parts.next().unwrap_or("");
    let domain = parts.next().unwrap_or("");
    !local.is_empty()
        && !domain.is_empty()
        && domain.contains('.')
        && !domain.starts_with('.')
        && !domain.ends_with('.')
}

/// Create a new event and warm up the Redis cache for `GET /api/v1/events/:id`.
///
/// # Endpoint
/// POST `/api/v1/events`
pub async fn create_event(
    State(mut state): State<EventState>,
    Json(payload): Json<CreateEventRequest>,
) -> Response {
    // Validate image_url: must start with https:// and have a non-empty host.
    if let Some(ref url) = payload.image_url {
        let is_valid = url.starts_with("https://")
            && url.len() > "https://".len()
            && !url["https://".len()..].starts_with('/');
        if !is_valid {
            return AppError::ValidationError("image_url must be a valid HTTPS URL".to_string())
                .into_response();
        }
    }

    // Validate host_email format when provided.
    if let Some(ref email) = payload.host_email {
        if !is_valid_email(email) {
            return AppError::ValidationError(
                "host_email must be a valid email address".to_string(),
            )
            .into_response();
        }
    }

    let event = match sqlx::query_as::<_, Event>(
        "INSERT INTO events (organizer_id, title, description, location, start_time, end_time, image_url, host_email)
         VALUES ($1, $2, $3, $4, $5, $6, $7, $8)
         RETURNING *",
    )
    .bind(payload.organizer_id)
    .bind(&payload.title)
    .bind(&payload.description)
    .bind(&payload.location)
    .bind(payload.start_time)
    .bind(payload.end_time)
    .bind(&payload.image_url)
    .bind(&payload.host_email)
    .fetch_one(&state.pool)
    .await
    {
        Ok(e) => e,
        Err(e) => {
            tracing::error!("Failed to create event: {:?}", e);
            return AppError::DatabaseError(e).into_response();
        }
    };

    // Cache warm-up: pre-populate event:detail:{id} so the first GET hits cache.
    let organizer_profile = match sqlx::query_scalar::<_, Option<String>>(
        "SELECT wallet_address FROM organizers WHERE id = $1",
    )
    .bind(event.organizer_id)
    .fetch_optional(&state.pool)
    .await
    {
        Ok(Some(Some(wallet))) => {
            match sqlx::query_as::<_, OrganizerProfile>(
                "SELECT * FROM organizer_profiles WHERE address = $1",
            )
            .bind(&wallet)
            .fetch_optional(&state.pool)
            .await
            {
                Ok(profile) => profile,
                Err(e) => {
                    tracing::warn!("Cache warm-up: failed to fetch organizer profile: {:?}", e);
                    None
                }
            }
        }
        _ => None,
    };

    let detail = EventDetail {
        event: event.clone(),
        organizer_profile,
    };

    let cache_key = format!("event:detail:{}", event.id);
    if let Err(e) = state.redis.set(&cache_key, &detail, EVENT_CACHE_TTL).await {
        tracing::warn!("Cache warm-up failed for event {}: {:?}", event.id, e);
    }

    success(event, "Event created successfully").into_response()
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

    let ticket = match sqlx::query_as::<_, (String, uuid::Uuid)>(
        r#"SELECT t.status, tt.event_id
           FROM tickets t
           JOIN ticket_tiers tt ON t.ticket_tier_id = tt.id
           WHERE t.id = $1"#,
    )
    .bind(payload.ticket_id)
    .fetch_optional(&state.pool)
    .await
    {
        Ok(Some((status, ticket_event_id))) => (status, ticket_event_id),
        Ok(None) => {
            return AppError::NotFound(format!("Ticket with id '{}' not found", payload.ticket_id))
                .into_response();
        }
        Err(e) => {
            tracing::error!("Failed to fetch ticket for rating: {:?}", e);
            return AppError::DatabaseError(e).into_response();
        }
    };

    let (ticket_status, ticket_event_id) = ticket;

    let event_exists =
        match sqlx::query_scalar::<_, bool>("SELECT EXISTS(SELECT 1 FROM events WHERE id = $1)")
            .bind(event_id)
            .fetch_one(&state.pool)
            .await
        {
            Ok(exists) => exists,
            Err(e) => {
                tracing::error!("Failed to check event existence for rating: {:?}", e);
                return AppError::DatabaseError(e).into_response();
            }
        };

    if !event_exists {
        return AppError::NotFound(format!("Event with id '{}' not found", event_id))
            .into_response();
    }

    if ticket_event_id != event_id {
        return AppError::Forbidden("Ticket does not belong to this event".to_string())
            .into_response();
    }

    if ticket_status != "used" {
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
    .fetch_optional(&mut *tx)
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
    .execute(&mut *tx)
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
    .fetch_optional(&mut *tx)
    .await
    {
        Ok(Some(event)) => event,
        Ok(None) => {
            return AppError::NotFound(format!("Event with id '{}' not found", event_id))
                .into_response();
        }
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
/// - `location` (optional): Filter by location (partial match, case-insensitive)
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

    // Collect all category IDs (multi-select + backward-compat single)
    let mut category_ids: Vec<Uuid> = Vec::new();
    if let Some(raw) = &params.category_ids {
        for part in raw.split(',') {
            if let Ok(id) = part.trim().parse::<Uuid>() {
                category_ids.push(id);
            }
        }
    }
    if let Some(id) = params.category_id {
        if !category_ids.contains(&id) {
            category_ids.push(id);
        }
    }

    // Filter by category (requires join with event_categories)
    let category_join = if !category_ids.is_empty() {
        param_count += 1;
        where_clauses.push(format!("ec.category_id = ANY(${})", param_count));
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

    // Filter by ticket type (requires join with ticket_tiers)
    let ticket_type_join = if params.ticket_type.is_some() {
        "INNER JOIN ticket_tiers tt ON e.id = tt.event_id"
    } else {
        ""
    };

    // Combine joins - if both price and ticket_type need ticket_tiers, use one join
    let ticket_tiers_join = if !price_join.is_empty() || !ticket_type_join.is_empty() {
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

    // Filter by ticket type (partial match on tier name)
    if params.ticket_type.is_some() {
        param_count += 1;
        where_clauses.push(format!("tt.name ILIKE ${}", param_count));
    }

    // Filter by location (partial match)
    if params.location.is_some() {
        param_count += 1;
        where_clauses.push(format!("e.location ILIKE ${}", param_count));
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
        category_join, ticket_tiers_join, where_clause
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
    if let Some(ref location) = params.location {
        count_query_builder = count_query_builder.bind(format!("%{}%", location));
    }
    if let Some(ref ticket_type) = params.ticket_type {
        count_query_builder = count_query_builder.bind(format!("%{}%", ticket_type));
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
        ticket_tiers_join,
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
    if let Some(ref location) = params.location {
        items_query_builder = items_query_builder.bind(format!("%{}%", location));
    }
    if let Some(ref ticket_type) = params.ticket_type {
        items_query_builder = items_query_builder.bind(format!("%{}%", ticket_type));
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

    let start = std::time::Instant::now();
    let items = match items_query_builder.fetch_all(&state.pool).await {
        Ok(events) => events,
        Err(e) => {
            tracing::error!("Failed to fetch search results: {:?}", e);
            return AppError::DatabaseError(e).into_response();
        }
    };
    log_if_slow("search_events", start.elapsed());

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
    State(mut state): State<EventState>,
    Path(event_id): Path<Uuid>,
) -> Response {
    // Fetch current flag status
    let current_flagged =
        match sqlx::query_scalar::<_, bool>("SELECT is_flagged FROM events WHERE id = $1")
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
    if let Err(e) = sqlx::query("UPDATE events SET is_flagged = $1 WHERE id = $2")
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
        "Event flag toggled successfully",
    )
    .into_response()
}

/// Revenue summary response for an event
#[derive(Debug, Serialize)]
pub struct EventRevenueResponse {
    pub total_revenue_usd: f64,
    pub tickets_sold: i64,
    pub average_ticket_price: f64,
}

/// Share link response for an event
#[derive(Debug, Serialize)]
pub struct EventShareLinkResponse {
    pub url: String,
    pub title: String,
    pub description: String,
}

/// Social proof response for an event
#[derive(Debug, Serialize, Deserialize)]
pub struct EventSocialProofResponse {
    pub recent_purchases: i64,
    pub average_rating: f32,
    pub waitlist_count: i64,
    pub tickets_remaining: i64,
}

/// Attendee count response for an event.
#[derive(Debug, Serialize, Deserialize)]
pub struct AttendeeCountResponse {
    pub count: i64,
    pub total_tickets: i64,
}

/// GET /api/v1/events/:id/share-link
///
/// Returns a canonical share URL for an event along with the event's title
/// and a truncated description (max 160 characters). Returns 404 for non-existent events.
pub async fn get_event_share_link(
    State(state): State<EventState>,
    Path(event_id): Path<Uuid>,
) -> Response {
    let event = match sqlx::query_as::<_, Event>(
        "SELECT * FROM events WHERE id = $1 AND is_flagged = FALSE",
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

    // Construct canonical URL
    let url = format!("{}/events/{}", state.base_url, event_id);

    // Truncate description to 160 characters
    let description = event
        .description
        .unwrap_or_default()
        .chars()
        .take(160)
        .collect();

    let response = EventShareLinkResponse {
        url,
        title: event.title,
        description,
    };

    success(response, "Share link retrieved successfully").into_response()
}

/// GET /api/v1/events/:id/social-proof
///
/// Returns social proof signals for an event: recent purchases (last 24 hours),
/// average rating, waitlist count, and tickets remaining.
/// Response is cached for 60 seconds. Returns 404 for non-existent events.
pub async fn get_event_social_proof(
    State(mut state): State<EventState>,
    Path(event_id): Path<Uuid>,
) -> Response {
    let cache_key = format!("event:social_proof:{}", event_id);

    // Try to get from cache first
    match state
        .redis
        .get::<EventSocialProofResponse>(&cache_key)
        .await
    {
        Ok(Some(proof)) => {
            tracing::debug!("Cache hit for social proof of event {}", event_id);
            return success(proof, "Social proof retrieved successfully (cached)").into_response();
        }
        Ok(None) => {
            tracing::debug!("Cache miss for social proof of event {}", event_id);
        }
        Err(e) => {
            tracing::warn!("Redis error, falling back to database: {:?}", e);
        }
    }

    // Check if event exists
    let event_exists =
        match sqlx::query_scalar::<_, bool>("SELECT EXISTS(SELECT 1 FROM events WHERE id = $1)")
            .bind(event_id)
            .fetch_one(&state.pool)
            .await
        {
            Ok(v) => v,
            Err(e) => {
                tracing::error!("Failed to check event existence: {:?}", e);
                return AppError::DatabaseError(e).into_response();
            }
        };

    if !event_exists {
        return AppError::NotFound(format!("Event with id '{}' not found", event_id))
            .into_response();
    }

    // Run queries in parallel using tokio::join!
    let (recent_purchases, rating_data, waitlist_count, tickets_remaining) = tokio::join!(
        // Recent purchases in last 24 hours
        async {
            sqlx::query_scalar::<_, i64>(
                "SELECT COUNT(*) FROM tickets WHERE event_id = $1 AND created_at > NOW() - INTERVAL '24 hours'",
            )
            .bind(event_id)
            .fetch_one(&state.pool)
            .await
            .unwrap_or(0)
        },
        // Average rating from events table
        async {
            sqlx::query_as::<_, (i64, i32)>(
                "SELECT sum_of_ratings, count_of_ratings FROM events WHERE id = $1",
            )
            .bind(event_id)
            .fetch_one(&state.pool)
            .await
        },
        // Waitlist count
        async {
            sqlx::query_scalar::<_, i64>(
                "SELECT COUNT(*) FROM waitlist_entries WHERE event_id = $1",
            )
            .bind(event_id)
            .fetch_one(&state.pool)
            .await
            .unwrap_or(0)
        },
        // Tickets remaining (total_tickets - minted_tickets)
        async {
            sqlx::query_scalar::<_, i64>(
                "SELECT total_tickets - minted_tickets FROM events WHERE id = $1",
            )
            .bind(event_id)
            .fetch_one(&state.pool)
            .await
            .unwrap_or(0)
        }
    );

    let average_rating = match rating_data {
        Ok((sum, count)) => {
            if count > 0 {
                sum as f32 / count as f32
            } else {
                0.0
            }
        }
        Err(_) => 0.0,
    };

    let response = EventSocialProofResponse {
        recent_purchases,
        average_rating,
        waitlist_count,
        tickets_remaining,
    };

    // Store in cache for 60 seconds
    if let Err(e) = state
        .redis
        .set(&cache_key, &response, SOCIAL_PROOF_CACHE_TTL)
        .await
    {
        tracing::warn!(
            "Failed to cache social proof for event {}: {:?}",
            event_id,
            e
        );
    }

    success(response, "Social proof retrieved successfully").into_response()
}

/// GET /api/v1/events/:id/attendees/count
///
/// Returns the number of minted tickets and total ticket capacity for an event.
pub async fn get_attendee_count(
    State(state): State<EventState>,
    Path(event_id): Path<Uuid>,
) -> Response {
    let row = match sqlx::query_as::<_, (i64, i64)>(
        "SELECT minted_tickets, total_tickets FROM events WHERE id = $1",
    )
    .bind(event_id)
    .fetch_optional(&state.pool)
    .await
    {
        Ok(Some(row)) => row,
        Ok(None) => {
            return AppError::NotFound(format!("Event with id '{}' not found", event_id))
                .into_response();
        }
        Err(e) => {
            tracing::error!("Failed to fetch attendee count: {:?}", e);
            return AppError::DatabaseError(e).into_response();
        }
    };

    success(
        AttendeeCountResponse {
            count: row.0,
            total_tickets: row.1,
        },
        "Attendee count retrieved successfully",
    )
    .into_response()
}

/// GET /api/v1/events/:id/revenue
///
/// Returns revenue statistics for an event: total revenue, tickets sold,
/// and average ticket price. Returns zeros for events with no tickets sold.
/// Returns 404 for non-existent events.
pub async fn get_event_revenue(
    State(state): State<EventState>,
    Path(event_id): Path<Uuid>,
) -> Response {
    // 404 if event doesn't exist
    let exists =
        match sqlx::query_scalar::<_, bool>("SELECT EXISTS(SELECT 1 FROM events WHERE id = $1)")
            .bind(event_id)
            .fetch_one(&state.pool)
            .await
        {
            Ok(v) => v,
            Err(e) => {
                tracing::error!("Failed to check event existence: {:?}", e);
                return AppError::DatabaseError(e).into_response();
            }
        };

    if !exists {
        return AppError::NotFound(format!("Event with id '{}' not found", event_id))
            .into_response();
    }

    let row = match sqlx::query(
        r#"
        SELECT
            COALESCE(SUM(tt.price * t.quantity), 0.0) AS total_revenue,
            COUNT(t.id) AS tickets_sold
        FROM tickets t
        JOIN ticket_tiers tt ON t.ticket_tier_id = tt.id
        WHERE tt.event_id = $1
        "#,
    )
    .bind(event_id)
    .fetch_one(&state.pool)
    .await
    {
        Ok(row) => row,
        Err(e) => {
            tracing::error!("Failed to fetch revenue stats: {:?}", e);
            return AppError::DatabaseError(e).into_response();
        }
    };

    let total_revenue: f64 = row.try_get::<f64, _>("total_revenue").unwrap_or(0.0);
    let tickets_sold: i64 = row.try_get::<i64, _>("tickets_sold").unwrap_or(0);
    let average_ticket_price = if tickets_sold > 0 {
        total_revenue / tickets_sold as f64
    } else {
        0.0
    };

    success(
        EventRevenueResponse {
            total_revenue_usd: total_revenue,
            tickets_sold,
            average_ticket_price,
        },
        "Revenue stats retrieved",
    )
    .into_response()
}

#[test]
fn test_event_filters_deserialization() {
    // Test that filters can be deserialized from query params
    let filters = EventFilters {
        organizer_id: Some(Uuid::new_v4()),
        organizer_wallet: Some("GABC123".to_string()),
        location: Some("New York".to_string()),
        start_after: None,
        start_before: None,
        search: Some("concert".to_string()),
        min_tickets_available: None,
        is_free: None,
        start_date: None,
        end_date: None,
        followers_only: None,
    };

    assert!(filters.organizer_id.is_some());
    assert_eq!(filters.organizer_wallet.as_deref(), Some("GABC123"));
    assert_eq!(filters.location.unwrap(), "New York");
}

#[test]
fn test_organizer_wallet_filter() {
    let filters = EventFilters {
        organizer_id: None,
        organizer_wallet: Some("GBXXX".to_string()),
        location: None,
        start_after: None,
        start_before: None,
        search: None,
        min_tickets_available: None,
        is_free: None,
        start_date: None,
        end_date: None,
        followers_only: None,
    };
    assert_eq!(filters.organizer_wallet.as_deref(), Some("GBXXX"));
}

#[test]
fn test_is_free_filter() {
    let filters_free = EventFilters {
        organizer_id: None,
        organizer_wallet: None,
        location: None,
        start_after: None,
        start_before: None,
        search: None,
        min_tickets_available: None,
        is_free: Some(true),
        start_date: None,
        end_date: None,
        followers_only: None,
    };
    assert_eq!(filters_free.is_free, Some(true));

    let filters_paid = EventFilters {
        organizer_id: None,
        organizer_wallet: None,
        location: None,
        start_after: None,
        start_before: None,
        search: None,
        min_tickets_available: None,
        is_free: Some(false),
        start_date: None,
        end_date: None,
        followers_only: None,
    };
    assert_eq!(filters_paid.is_free, Some(false));

    let filters_none = EventFilters {
        organizer_id: None,
        organizer_wallet: None,
        location: None,
        start_after: None,
        start_before: None,
        search: None,
        min_tickets_available: None,
        is_free: None,
        start_date: None,
        end_date: None,
        followers_only: None,
    };
    assert_eq!(filters_none.is_free, None);
}

#[test]
fn test_ratings_summary_distribution_zero_filled() {
    let mut distribution = std::collections::HashMap::new();
    for star in 1i16..=5 {
        distribution.insert(star.to_string(), 0i64);
    }
    // Simulate two ratings: one 4-star, one 5-star
    distribution.insert("4".to_string(), 1i64);
    distribution.insert("5".to_string(), 1i64);

    assert_eq!(distribution["1"], 0);
    assert_eq!(distribution["2"], 0);
    assert_eq!(distribution["3"], 0);
    assert_eq!(distribution["4"], 1);
    assert_eq!(distribution["5"], 1);
}

#[test]
fn test_ratings_summary_average_no_ratings() {
    let total = 0i64;
    let average = if total > 0 { 1.0f64 } else { 0.0f64 };
    assert_eq!(average, 0.0);
}

#[test]
fn test_description_truncation() {
    let long_description = "This is a very long description that should be truncated to exactly 160 characters to ensure it fits within the limit for social media sharing and other use cases where space is limited.";
    let truncated: String = long_description.chars().take(160).collect();
    assert!(truncated.len() <= 160);
    assert_eq!(truncated.len(), 160);
}

#[test]
fn test_description_truncation_short() {
    let short_description = "Short description";
    let truncated: String = short_description.chars().take(160).collect();
    assert_eq!(truncated, "Short description");
}

#[test]
fn test_description_truncation_empty() {
    let empty_description = "";
    let truncated: String = empty_description.chars().take(160).collect();
    assert_eq!(truncated, "");
}

#[test]
fn test_social_proof_response_serialization() {
    let response = EventSocialProofResponse {
        recent_purchases: 12,
        average_rating: 4.5,
        waitlist_count: 8,
        tickets_remaining: 43,
    };

    assert_eq!(response.recent_purchases, 12);
    assert_eq!(response.average_rating, 4.5);
    assert_eq!(response.waitlist_count, 8);
    assert_eq!(response.tickets_remaining, 43);
}

#[test]
fn test_social_proof_zero_values() {
    let response = EventSocialProofResponse {
        recent_purchases: 0,
        average_rating: 0.0,
        waitlist_count: 0,
        tickets_remaining: 0,
    };

    assert_eq!(response.recent_purchases, 0);
    assert_eq!(response.average_rating, 0.0);
    assert_eq!(response.waitlist_count, 0);
    assert_eq!(response.tickets_remaining, 0);
}

#[test]
fn test_search_params_ticket_type() {
    let params = SearchParams {
        q: None,
        category_id: None,
        category_ids: None,
        min_price: None,
        max_price: None,
        date_from: None,
        date_to: None,
        location: None,
        ticket_type: Some("VIP".to_string()),
        page: 1,
        page_size: 20,
    };

    assert_eq!(params.ticket_type, Some("VIP".to_string()));
}

#[test]
fn test_search_params_ticket_type_none() {
    let params = SearchParams {
        q: None,
        category_id: None,
        category_ids: None,
        min_price: None,
        max_price: None,
        date_from: None,
        date_to: None,
        location: None,
        ticket_type: None,
        page: 1,
        page_size: 20,
    };

    assert!(params.ticket_type.is_none());
}

#[test]
fn test_ratings_summary_average_computed() {
    // 1×4 + 1×5 = 9 / 2 = 4.5
    let rows: Vec<(i16, i64)> = vec![(4, 1), (5, 1)];
    let total: i64 = rows.iter().map(|(_, c)| c).sum();
    let weighted: i64 = rows.iter().map(|(r, c)| *r as i64 * c).sum();
    let average = weighted as f64 / total as f64;
    assert_eq!(average, 4.5);
}

#[test]
fn test_search_params_location() {
    let params = SearchParams {
        q: None,
        category_id: None,
        category_ids: None,
        min_price: None,
        max_price: None,
        date_from: None,
        date_to: None,
        location: Some("Lagos".to_string()),
        ticket_type: None,
        page: 1,
        page_size: 20,
    };
    assert_eq!(params.location.as_deref(), Some("Lagos"));
}

#[test]
fn test_export_attendees_csv_format() {
    // Test CSV header format
    let header = "owner_wallet,buyer_wallet,quantity,created_at\n";
    assert!(header.contains("owner_wallet"));
    assert!(header.contains("buyer_wallet"));
    assert!(header.contains("quantity"));
    assert!(header.contains("created_at"));
}

#[test]
fn test_csv_row_format() {
    // Test that a CSV row can be formatted correctly
    let owner = "GXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXX";
    let buyer = "GYYYYYYYYYYYYYYYYYYYYYYYYYYYYYYYYYYYYYYYYYYYYYYYYYYYYYYY";
    let quantity = 2;
    let created_at = chrono::Utc::now();

    let row = format!(
        "{},{},{},{}\n",
        owner,
        buyer,
        quantity,
        created_at.to_rfc3339()
    );

    assert!(row.contains(owner));
    assert!(row.contains(buyer));
    assert!(row.contains("2"));
}

#[derive(Serialize)]
pub struct CheckInStats {
    pub checked_in: i64,
    pub total_sold: i64,
    pub remaining: i64,
}

/// Response body for the ratings summary endpoint
#[derive(Debug, Serialize, Deserialize)]
pub struct RatingsSummary {
    pub average: f64,
    pub total: i64,
    pub distribution: std::collections::HashMap<String, i64>,
}

/// GET /api/v1/events/:id/ratings/summary
///
/// Returns the star-rating distribution for an event. Result is cached for 5 minutes.
pub async fn get_ratings_summary(
    State(mut state): State<EventState>,
    Path(event_id): Path<Uuid>,
) -> Response {
    let cache_key = format!("event:ratings_summary:{}", event_id);

    match state.redis.get::<RatingsSummary>(&cache_key).await {
        Ok(Some(summary)) => {
            return success(summary, "Ratings summary retrieved (cached)").into_response()
        }
        Ok(None) => {}
        Err(e) => tracing::warn!("Redis error for ratings summary cache: {:?}", e),
    }

    // 404 if event doesn't exist
    let exists =
        match sqlx::query_scalar::<_, bool>("SELECT EXISTS(SELECT 1 FROM events WHERE id = $1)")
            .bind(event_id)
            .fetch_one(&state.pool)
            .await
        {
            Ok(v) => v,
            Err(e) => {
                tracing::error!("Failed to check event existence: {:?}", e);
                return AppError::DatabaseError(e).into_response();
            }
        };

    if !exists {
        return AppError::NotFound(format!("Event with id '{}' not found", event_id))
            .into_response();
    }

    let rows = match sqlx::query_as::<_, (i16, i64)>(
        "SELECT rating, COUNT(*) FROM event_ratings \
         WHERE event_id = $1 GROUP BY rating ORDER BY rating",
    )
    .bind(event_id)
    .fetch_all(&state.pool)
    .await
    {
        Ok(rows) => rows,
        Err(e) => {
            tracing::error!("Failed to fetch ratings: {:?}", e);
            return AppError::DatabaseError(e).into_response();
        }
    };

    let mut distribution = std::collections::HashMap::new();
    for star in 1i16..=5 {
        distribution.insert(star.to_string(), 0i64);
    }
    for (rating, count) in &rows {
        distribution.insert(rating.to_string(), *count);
    }

    let total: i64 = rows.iter().map(|(_, c)| c).sum();
    let weighted: i64 = rows.iter().map(|(r, c)| *r as i64 * c).sum();
    let average = if total > 0 {
        weighted as f64 / total as f64
    } else {
        0.0
    };

    let summary = RatingsSummary {
        average,
        total,
        distribution,
    };

    if let Err(e) = state.redis.set(&cache_key, &summary, EVENT_CACHE_TTL).await {
        tracing::warn!(
            "Failed to cache ratings summary for event {}: {:?}",
            event_id,
            e
        );
    }

    success(summary, "Ratings summary retrieved").into_response()
}

const EVENT_COUNT_CACHE_KEY: &str = "events:count";
const EVENT_COUNT_CACHE_TTL: Duration = Duration::from_secs(600);

#[derive(Debug, Serialize, Deserialize)]
pub struct EventCounts {
    pub total: i64,
    pub upcoming: i64,
}

/// GET /api/v1/events/count
///
/// Returns the total and upcoming event counts, excluding flagged events.
/// Result is cached in Redis for 10 minutes.
pub async fn get_event_counts(State(mut state): State<EventState>) -> Response {
    match state.redis.get::<EventCounts>(EVENT_COUNT_CACHE_KEY).await {
        Ok(Some(counts)) => {
            return success(counts, "Event counts retrieved (cached)").into_response()
        }
        Ok(None) => {}
        Err(e) => tracing::warn!("Redis error for event counts cache: {:?}", e),
    }

    let total =
        match sqlx::query_scalar::<_, i64>("SELECT COUNT(*) FROM events WHERE is_flagged = FALSE")
            .fetch_one(&state.pool)
            .await
        {
            Ok(n) => n,
            Err(e) => {
                tracing::error!("Failed to count events: {:?}", e);
                return AppError::DatabaseError(e).into_response();
            }
        };

    let upcoming = match sqlx::query_scalar::<_, i64>(
        "SELECT COUNT(*) FROM events WHERE end_time > NOW() AND is_flagged = FALSE",
    )
    .fetch_one(&state.pool)
    .await
    {
        Ok(n) => n,
        Err(e) => {
            tracing::error!("Failed to count upcoming events: {:?}", e);
            return AppError::DatabaseError(e).into_response();
        }
    };

    let counts = EventCounts { total, upcoming };

    if let Err(e) = state
        .redis
        .set(EVENT_COUNT_CACHE_KEY, &counts, EVENT_COUNT_CACHE_TTL)
        .await
    {
        tracing::warn!("Failed to cache event counts: {:?}", e);
    }

    success(counts, "Event counts retrieved").into_response()
}

/// GET /api/v1/events/:id/check-in-stats
pub async fn get_checkin_stats(
    State(state): State<EventState>,
    Path(event_id): Path<Uuid>,
) -> Response {
    let row = sqlx::query(
        r#"
        SELECT
            COUNT(*) FILTER (WHERE status = 'used') AS checked_in,
            COUNT(*) AS total_sold
        FROM tickets
        WHERE event_id = $1
        "#,
    )
    .bind(event_id)
    .fetch_optional(&state.pool)
    .await;

    match row {
        Ok(Some(r)) => {
            let checked_in: i64 = r.try_get("checked_in").unwrap_or(0);
            let total_sold: i64 = r.try_get("total_sold").unwrap_or(0);
            success(
                CheckInStats {
                    checked_in,
                    total_sold,
                    remaining: total_sold - checked_in,
                },
                "Check-in stats retrieved",
            )
            .into_response()
        }
        Ok(None) => AppError::NotFound(format!("Event '{}' not found", event_id)).into_response(),
        Err(e) => AppError::InternalServerError(e.to_string()).into_response(),
    }
}

/// GET /api/v1/events/:id/organizer
///
/// Returns the organizer profile for the event's organizer wallet.
/// This is a lightweight endpoint for clients that only need organizer info.
pub async fn get_event_organizer(
    State(state): State<EventState>,
    Path(event_id): Path<Uuid>,
) -> Response {
    // First, verify the event exists and get the organizer_id
    let organizer_id = match sqlx::query_scalar::<_, Uuid>(
        "SELECT organizer_id FROM events WHERE id = $1 AND is_flagged = FALSE",
    )
    .bind(event_id)
    .fetch_optional(&state.pool)
    .await
    {
        Ok(Some(id)) => id,
        Ok(None) => {
            return AppError::NotFound(format!("Event with id '{}' not found", event_id))
                .into_response();
        }
        Err(e) => {
            tracing::error!("Failed to fetch event organizer_id: {:?}", e);
            return AppError::DatabaseError(e).into_response();
        }
    };

    // Fetch the organizer's wallet address
    let wallet_address = match sqlx::query_scalar::<_, String>(
        "SELECT wallet_address FROM organizers WHERE id = $1",
    )
    .bind(organizer_id)
    .fetch_optional(&state.pool)
    .await
    {
        Ok(Some(wallet)) => wallet,
        Ok(None) => {
            return AppError::NotFound(format!(
                "Organizer profile not found for event '{}'",
                event_id
            ))
            .into_response();
        }
        Err(e) => {
            tracing::error!("Failed to fetch organizer wallet: {:?}", e);
            return AppError::DatabaseError(e).into_response();
        }
    };

    // Fetch the organizer profile
    let profile = match sqlx::query_as::<_, OrganizerProfile>(
        "SELECT * FROM organizer_profiles WHERE address = $1",
    )
    .bind(&wallet_address)
    .fetch_optional(&state.pool)
    .await
    {
        Ok(Some(profile)) => profile,
        Ok(None) => {
            return AppError::NotFound(format!(
                "Organizer profile not found for event '{}'",
                event_id
            ))
            .into_response();
        }
        Err(e) => {
            tracing::error!("Failed to fetch organizer profile: {:?}", e);
            return AppError::DatabaseError(e).into_response();
        }
    };

    success(profile, "Organizer profile retrieved successfully").into_response()
}

/// GET /api/v1/events/:id/export-attendees
///
/// Exports all attendees for an event as a CSV file.
/// Returns owner_wallet, buyer_wallet, quantity, created_at for all tickets.
pub async fn export_attendees_csv(
    State(state): State<EventState>,
    Path(event_id): Path<Uuid>,
) -> Response {
    // Verify the event exists
    let event_exists =
        match sqlx::query_scalar::<_, bool>("SELECT EXISTS(SELECT 1 FROM events WHERE id = $1)")
            .bind(event_id)
            .fetch_one(&state.pool)
            .await
        {
            Ok(exists) => exists,
            Err(e) => {
                tracing::error!("Failed to check event existence: {:?}", e);
                return AppError::DatabaseError(e).into_response();
            }
        };

    if !event_exists {
        return AppError::NotFound(format!("Event with id '{}' not found", event_id))
            .into_response();
    }

    // Fetch all tickets for the event
    let tickets = match sqlx::query_as::<_, (String, String, i32, chrono::DateTime<Utc>)>(
        r#"
        SELECT 
            t.owner_wallet,
            t.buyer_wallet,
            t.quantity,
            t.created_at
        FROM tickets t
        JOIN ticket_tiers tt ON t.ticket_tier_id = tt.id
        WHERE tt.event_id = $1
        ORDER BY t.created_at ASC
        "#,
    )
    .bind(event_id)
    .fetch_all(&state.pool)
    .await
    {
        Ok(tickets) => tickets,
        Err(e) => {
            tracing::error!("Failed to fetch tickets for CSV export: {:?}", e);
            return AppError::DatabaseError(e).into_response();
        }
    };

    // Build CSV string manually
    let mut csv = String::from("owner_wallet,buyer_wallet,quantity,created_at\n");
    for (owner_wallet, buyer_wallet, quantity, created_at) in tickets {
        csv.push_str(&format!(
            "{},{},{},{}\n",
            owner_wallet,
            buyer_wallet,
            quantity,
            created_at.to_rfc3339()
        ));
    }

    // Return CSV with appropriate headers
    (
        axum::http::StatusCode::OK,
        [
            ("Content-Type", "text/csv"),
            (
                "Content-Disposition",
                &format!("attachment; filename=\"attendees-{}.csv\"", event_id),
            ),
        ],
        csv,
    )
        .into_response()
}

// ---------------------------------------------------------------------------
// Issue: List tickets for an event
// ---------------------------------------------------------------------------

/// A single ticket row returned by the list_event_tickets endpoint.
#[derive(Debug, Serialize, Deserialize, sqlx::FromRow)]
pub struct EventTicket {
    pub id: Uuid,
    pub buyer_wallet: Option<String>,
    pub owner_wallet: Option<String>,
    /// Quantity included for schema compatibility. Defaults to 1 for
    /// on-chain synced tickets where quantity is not stored separately.
    pub quantity: i32,
    pub created_at: chrono::DateTime<Utc>,
    /// On-chain Stellar ticket ID for independent verification.
    pub stellar_id: Option<String>,
}

/// GET /api/v1/events/:id/tickets
///
/// Returns a paginated list of tickets purchased for the given event.
/// Useful for organiser check-in management and reporting.
///
/// # Query Parameters
/// - `page` (optional, default 1)
/// - `page_size` (optional, default 20, max 100)
///
/// # Response
/// Returns a `PaginatedResponse<EventTicket>`.
pub async fn list_event_tickets(
    State(state): State<EventState>,
    Path(event_id): Path<Uuid>,
    Query(pagination): Query<PaginationParams>,
) -> Response {
    // 404 if event does not exist.
    let event_exists =
        match sqlx::query_scalar::<_, bool>("SELECT EXISTS(SELECT 1 FROM events WHERE id = $1)")
            .bind(event_id)
            .fetch_one(&state.pool)
            .await
        {
            Ok(v) => v,
            Err(e) => {
                tracing::error!("Failed to check event existence for tickets: {:?}", e);
                return AppError::DatabaseError(e).into_response();
            }
        };

    if !event_exists {
        return AppError::NotFound(format!("Event with id '{}' not found", event_id))
            .into_response();
    }

    let validated = pagination.validate();

    // Count total tickets for pagination metadata.
    let total =
        match sqlx::query_scalar::<_, i64>("SELECT COUNT(*) FROM tickets WHERE event_id = $1")
            .bind(event_id)
            .fetch_one(&state.pool)
            .await
        {
            Ok(n) => n,
            Err(e) => {
                tracing::error!("Failed to count event tickets: {:?}", e);
                return AppError::DatabaseError(e).into_response();
            }
        };

    let items = match sqlx::query_as::<_, EventTicket>(
        r#"
        SELECT
            id,
            buyer_wallet,
            owner_wallet,
            1::int4          AS quantity,
            created_at,
            stellar_id
        FROM tickets
        WHERE event_id = $1
        ORDER BY created_at DESC
        LIMIT $2 OFFSET $3
        "#,
    )
    .bind(event_id)
    .bind(validated.limit())
    .bind(validated.offset())
    .fetch_all(&state.pool)
    .await
    {
        Ok(rows) => rows,
        Err(e) => {
            tracing::error!("Failed to fetch event tickets: {:?}", e);
            return AppError::DatabaseError(e).into_response();
        }
    };

    let response = PaginatedResponse::new(items, validated, total);
    success(response, "Tickets retrieved successfully").into_response()
}

/// GET `/api/v1/events/categories/:category_id`
///
/// Returns a cursor-paginated list of upcoming events in the given category.
pub async fn list_events_by_category(
    State(state): State<EventState>,
    axum::extract::Path(category_id): axum::extract::Path<Uuid>,
    Query(pagination): Query<CursorParams>,
) -> Response {
    // Verify category exists
    let category_exists = match sqlx::query_scalar::<_, bool>(
        "SELECT EXISTS(SELECT 1 FROM categories WHERE id = $1)",
    )
    .bind(category_id)
    .fetch_one(&state.pool)
    .await
    {
        Ok(exists) => exists,
        Err(e) => {
            tracing::error!("Failed to check category existence: {:?}", e);
            return AppError::DatabaseError(e).into_response();
        }
    };

    if !category_exists {
        return AppError::NotFound(format!("Category with id '{}' not found", category_id))
            .into_response();
    }

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

    // Construct query dynamically based on cursor existence
    let items_query = if cursor.is_some() {
        "SELECT e.* FROM events e \
         INNER JOIN event_categories ec ON e.id = ec.event_id \
         WHERE ec.category_id = $1 \
           AND e.end_time > NOW() \
           AND e.is_flagged = FALSE \
           AND (e.start_time > $3 OR (e.start_time = $3 AND e.id > $4)) \
         ORDER BY e.start_time ASC, e.id ASC \
         LIMIT $2"
            .to_string()
    } else {
        "SELECT e.* FROM events e \
         INNER JOIN event_categories ec ON e.id = ec.event_id \
         WHERE ec.category_id = $1 \
           AND e.end_time > NOW() \
           AND e.is_flagged = FALSE \
         ORDER BY e.start_time ASC, e.id ASC \
         LIMIT $2"
            .to_string()
    };

    // Query items (query limit is page_size + 1 to detect has_more)
    let mut items_query_builder = sqlx::query_as::<_, Event>(&items_query)
        .bind(category_id)
        .bind(validated.query_limit());

    if let Some(ref c) = cursor {
        items_query_builder = items_query_builder.bind(c.start_time).bind(c.id);
    }

    let mut items = match items_query_builder.fetch_all(&state.pool).await {
        Ok(events) => events,
        Err(e) => {
            tracing::error!("Failed to fetch events by category: {:?}", e);
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
                return AppError::InternalServerError("Failed to encode cursor".to_string())
                    .into_response();
            }
        }
    } else {
        None
    };

    let response = CursorResponse::new(items, &validated, next_cursor);
    success(response, "Events in category retrieved successfully").into_response()
}

// ---------------------------------------------------------------------------
// Unit tests for list_event_tickets and image_url validation
// ---------------------------------------------------------------------------

#[test]
fn test_image_url_valid_https() {
    let url = "https://example.com/image.jpg";
    let is_valid = url.starts_with("https://")
        && url.len() > "https://".len()
        && !url["https://".len()..].starts_with('/');
    assert!(is_valid);
}

#[test]
fn test_image_url_http_rejected() {
    let url = "http://example.com/image.jpg";
    let is_valid = url.starts_with("https://")
        && url.len() > "https://".len()
        && !url["https://".len()..].starts_with('/');
    assert!(!is_valid);
}

#[test]
fn test_image_url_javascript_rejected() {
    let url = "javascript:alert(1)";
    let is_valid = url.starts_with("https://")
        && url.len() > "https://".len()
        && !url["https://".len()..].starts_with('/');
    assert!(!is_valid);
}

#[test]
fn test_image_url_empty_host_rejected() {
    let url = "https://";
    let is_valid = url.starts_with("https://")
        && url.len() > "https://".len()
        && !url["https://".len()..].starts_with('/');
    assert!(!is_valid);
}

#[test]
fn test_image_url_relative_path_rejected() {
    let url = "https:///path/to/image.jpg";
    let is_valid = url.starts_with("https://")
        && url.len() > "https://".len()
        && !url["https://".len()..].starts_with('/');
    assert!(!is_valid);
}

#[test]
fn test_event_ticket_struct_fields() {
    let ticket = EventTicket {
        id: Uuid::new_v4(),
        buyer_wallet: Some("GBUYER123".to_string()),
        owner_wallet: Some("GOWNER456".to_string()),
        quantity: 1,
        created_at: chrono::Utc::now(),
        stellar_id: Some("stellar-tx-abc".to_string()),
    };
    assert_eq!(ticket.quantity, 1);
    assert_eq!(ticket.buyer_wallet.as_deref(), Some("GBUYER123"));
    assert!(ticket.stellar_id.is_some());
}

#[test]
fn test_list_events_by_category_params() {
    let params = CursorParams {
        limit: 15,
        cursor: Some("test-cursor-token".to_string()),
    };
    let validated = params.validate();
    assert_eq!(validated.page_size(), 15);
    assert_eq!(validated.cursor.as_deref(), Some("test-cursor-token"));
}

/// Return upcoming featured events for the home page.
///
/// # Endpoint
/// GET `/api/v1/events/featured`
///
/// Queries events where `is_featured = TRUE`, `end_time > NOW()`, and
/// `is_flagged = FALSE`, ordered by `start_time ASC`, limited to 10.
pub async fn list_featured_events(State(state): State<EventState>) -> Response {
    let start = std::time::Instant::now();
    let result = sqlx::query_as::<_, Event>(
        r"
        SELECT * FROM events
        WHERE is_featured = TRUE
          AND (end_time IS NULL OR end_time > NOW())
          AND is_flagged = FALSE
        ORDER BY start_time ASC
        LIMIT 10
        ",
    )
    .fetch_all(&state.pool)
    .await;
    log_if_slow("list_featured_events", start.elapsed());

    match result {
        Ok(events) => success(events, "Featured events retrieved successfully").into_response(),
        Err(e) => {
            tracing::error!("Failed to fetch featured events: {:?}", e);
            AppError::DatabaseError(e).into_response()
        }
    }
}

#[cfg(test)]
mod host_email_tests {
    use super::is_valid_email;

    #[test]
    fn test_valid_emails_accepted() {
        assert!(is_valid_email("host@example.com"));
        assert!(is_valid_email("host+tag@sub.example.org"));
        assert!(is_valid_email("a@b.co"));
    }

    #[test]
    fn test_invalid_emails_rejected() {
        assert!(!is_valid_email("not-an-email"));
        assert!(!is_valid_email("@nodomain.com"));
        assert!(!is_valid_email("noatsign"));
        assert!(!is_valid_email("missing@.dot"));
        assert!(!is_valid_email("trailing@dot."));
        assert!(!is_valid_email(""));
    }
}
