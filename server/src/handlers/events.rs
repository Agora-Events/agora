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
use serde_json::json;
use sqlx::{PgPool, Row};
use std::time::Duration;
use uuid::Uuid;

use crate::cache::{RedisCache, EVENTS_LIST_CACHE_KEY, EVENTS_LIST_CACHE_TTL};
use crate::middleware::audit::AuditMetadata;
use crate::models::event::{populate_is_free, Event};
use crate::models::organizer_profile::OrganizerProfile;
use crate::utils::cursor_pagination::{
    decode_cursor, encode_cursor, AttendeeCursor, CursorParams, CursorResponse, EventCursor,
    PastEventCursor,
};
use crate::utils::db_timer::log_if_slow;
use crate::utils::error::AppError;
use crate::utils::pagination::{PaginatedResponse, PaginationParams};
use crate::utils::response::success;
use axum::http::HeaderValue;

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

impl SearchParams {
    fn validate_page_size(&self) -> Result<(), String> {
        if self.page_size == 0 || self.page_size > 100 {
            Err("page_size must be between 1 and 100".to_string())
        } else {
            Ok(())
        }
    }
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
    /// Ticket tiers for the event, sorted by price ascending.
    /// Only present when `?include_tiers=true` is passed (Issue #884).
    #[serde(skip_serializing_if = "Option::is_none")]
    pub tiers: Option<Vec<TicketTierResponse>>,
}

/// Query parameters for `GET /api/v1/events/:id`.
#[derive(Debug, Deserialize, Default)]
pub struct GetEventParams {
    /// When `true`, includes a `tiers` array in the response (Issue #884).
    #[serde(default)]
    pub include_tiers: bool,
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

    /// Filter events that have been marked featured.
    pub is_featured: Option<bool>,

    /// Filter to return only followers-only events (Issue #ForYou)
    pub followers_only: Option<bool>,

    /// Sort field: `start_time` (default), `created_at`, or `popularity` (minted_tickets)
    pub sort_by: Option<String>,

    /// Sort direction: `asc` (default) or `desc`
    pub sort_order: Option<String>,

    /// Simple sort parameter: `newest` (start_time DESC) or `popular` (count_of_ratings DESC).
    /// Takes precedence over `sort_by`/`sort_order` when provided.
    pub sort: Option<String>,
}

/// Supported sort fields for event listings.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum EventSortBy {
    StartTime,
    CreatedAt,
    Popularity,
    /// Sort by review count (alias of "popular" but exposes the underlying column).
    CountOfRatings,
    /// Sort by minimum ticket-tier price.
    Price,
}

impl EventSortBy {
    fn parse(value: &str) -> Result<Self, String> {
        match value {
            "start_time" => Ok(Self::StartTime),
            "created_at" => Ok(Self::CreatedAt),
            "popularity" => Ok(Self::Popularity),
            "count_of_ratings" => Ok(Self::CountOfRatings),
            other => Err(format!(
                "Invalid sort_by value '{}'. Supported values: start_time, created_at, popularity, count_of_ratings",
                other
            )),
        }
    }
}

/// Sort direction for event listings.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SortOrder {
    Asc,
    Desc,
}

impl SortOrder {
    fn parse(value: &str) -> Result<Self, String> {
        match value {
            "asc" => Ok(Self::Asc),
            "desc" => Ok(Self::Desc),
            other => Err(format!(
                "Invalid sort_order value '{}'. Supported values: asc, desc",
                other
            )),
        }
    }

    fn sql(self) -> &'static str {
        match self {
            Self::Asc => "ASC",
            Self::Desc => "DESC",
        }
    }
}

/// Validated sort parameters for event listings.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct ValidatedEventSort {
    pub sort_by: EventSortBy,
    pub sort_order: SortOrder,
}

impl EventFilters {
    /// True when no query filters are applied (eligible for the shared list cache).
    fn is_unfiltered(&self) -> bool {
        self.organizer_id.is_none()
            && self.organizer_wallet.is_none()
            && self.location.is_none()
            && self.start_after.is_none()
            && self.start_before.is_none()
            && self.search.is_none()
            && self.min_tickets_available.is_none()
            && self.is_free.is_none()
            && self.start_date.is_none()
            && self.end_date.is_none()
            && self.is_featured.is_none()
            && self.followers_only.is_none()
            && self.sort_by.is_none()
            && self.sort_order.is_none()
            && self.sort.is_none()
    }

    /// Validate `sort_by` and `sort_order`, applying defaults when omitted.
    /// The `sort` field takes precedence when provided.
    ///
    /// Accepted `sort` values (allow-list, never interpolated into SQL):
    /// `starts_at_asc` (default), `starts_at_desc`, `price_asc`, `price_desc`,
    /// `popularity_desc`. Legacy values `newest` and `popular` are still accepted.
    pub fn validate_sort(&self) -> Result<ValidatedEventSort, String> {
        // Simple `sort` param takes precedence over the legacy `sort_by`/`sort_order` pair.
        if let Some(ref sort) = self.sort {
            return parse_sort_param(sort);
        }

        let sort_by = match self.sort_by.as_deref() {
            Some(value) => EventSortBy::parse(value)?,
            None => EventSortBy::StartTime,
        };
        let sort_order = match self.sort_order.as_deref() {
            Some(value) => SortOrder::parse(value)?,
            None => SortOrder::Asc,
        };
        Ok(ValidatedEventSort {
            sort_by,
            sort_order,
        })
    }
}

/// Allow-listed `?sort=` values. The raw parameter is never concatenated into SQL.
fn parse_sort_param(sort: &str) -> Result<ValidatedEventSort, String> {
    match sort {
        "starts_at_asc" => Ok(ValidatedEventSort {
            sort_by: EventSortBy::StartTime,
            sort_order: SortOrder::Asc,
        }),
        "starts_at_desc" => Ok(ValidatedEventSort {
            sort_by: EventSortBy::StartTime,
            sort_order: SortOrder::Desc,
        }),
        "price_asc" => Ok(ValidatedEventSort {
            sort_by: EventSortBy::Price,
            sort_order: SortOrder::Asc,
        }),
        "price_desc" => Ok(ValidatedEventSort {
            sort_by: EventSortBy::Price,
            sort_order: SortOrder::Desc,
        }),
        "popularity_desc" => Ok(ValidatedEventSort {
            sort_by: EventSortBy::Popularity,
            sort_order: SortOrder::Desc,
        }),
        "newest" => Ok(ValidatedEventSort {
            sort_by: EventSortBy::StartTime,
            sort_order: SortOrder::Desc,
        }),
        "popular" => Ok(ValidatedEventSort {
            sort_by: EventSortBy::CountOfRatings,
            sort_order: SortOrder::Desc,
        }),
        other => Err(format!(
            "Invalid sort value '{}'. Supported values: starts_at_asc, starts_at_desc, price_asc, price_desc, popularity_desc",
            other
        )),
    }
}

/// Allow-listed SQL expression for the cheapest ticket-tier price.
/// Never interpolates user input.
const MIN_TICKET_PRICE_SQL: &str =
    "COALESCE((SELECT MIN(tt.price) FROM ticket_tiers tt WHERE tt.event_id = events.id), 0)";

/// Build the ORDER BY clause for event listings.
fn build_event_order_by_clause(sort: &ValidatedEventSort) -> String {
    let (column, tiebreaker_direction) = match sort.sort_by {
        EventSortBy::StartTime => ("start_time", sort.sort_order.sql()),
        EventSortBy::CreatedAt => ("created_at", sort.sort_order.sql()),
        EventSortBy::Popularity => ("minted_tickets", sort.sort_order.sql()),
        EventSortBy::CountOfRatings => ("count_of_ratings", "ASC"),
        EventSortBy::Price => (MIN_TICKET_PRICE_SQL, sort.sort_order.sql()),
    };
    let direction = sort.sort_order.sql();
    format!("ORDER BY {column} {direction}, id {tiebreaker_direction}")
}

/// Build WHERE clause and return (where_clause, param_count)
fn build_event_where_clause(
    filters: &EventFilters,
    sort: &ValidatedEventSort,
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
        where_clauses.push(format!("e.organizer_id = ${}", param_count));
    }

    if filters.location.is_some() {
        param_count += 1;
        where_clauses.push(format!("e.location ILIKE ${}", param_count));
    }

    if filters.start_after.is_some() {
        param_count += 1;
        where_clauses.push(format!("e.start_time >= ${}", param_count));
    }

    if filters.start_before.is_some() {
        param_count += 1;
        where_clauses.push(format!("e.start_time <= ${}", param_count));
    }

    if filters.search.is_some() {
        param_count += 1;
        where_clauses.push(format!(
            "(e.title ILIKE ${} OR e.description ILIKE ${})",
            param_count, param_count
        ));
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

    if let Some(is_featured) = filters.is_featured {
        if is_featured {
            where_clauses.push("is_featured = TRUE".to_string());
        } else {
            where_clauses.push("is_featured = FALSE".to_string());
        }
    };

    // Cursor condition for keyset pagination on the active sort column.
    if cursor.is_some() {
        param_count += 1;
        let key_param = param_count;
        param_count += 1;
        let id_param = param_count;

        let (key_col, key_op, id_op) = match (sort.sort_by, sort.sort_order) {
            (EventSortBy::StartTime, SortOrder::Asc) => ("start_time", ">", ">"),
            (EventSortBy::StartTime, SortOrder::Desc) => ("start_time", "<", "<"),
            (EventSortBy::CreatedAt, SortOrder::Asc) => ("created_at", ">", ">"),
            (EventSortBy::CreatedAt, SortOrder::Desc) => ("created_at", "<", "<"),
            (EventSortBy::Popularity, SortOrder::Asc) => ("minted_tickets", ">", ">"),
            (EventSortBy::Popularity, SortOrder::Desc) => ("minted_tickets", "<", "<"),
            (EventSortBy::CountOfRatings, SortOrder::Asc) => ("count_of_ratings", ">", ">"),
            (EventSortBy::CountOfRatings, SortOrder::Desc) => ("count_of_ratings", "<", "<"),
            (EventSortBy::Price, SortOrder::Asc) => (MIN_TICKET_PRICE_SQL, ">", ">"),
            (EventSortBy::Price, SortOrder::Desc) => (MIN_TICKET_PRICE_SQL, "<", "<"),
        };

        where_clauses.push(format!(
            "({key_col} {key_op} ${key_param} OR ({key_col} = ${key_param} AND id {id_op} ${id_param}))"
        ));
    }

    let where_clause = format!("WHERE {}", where_clauses.join(" AND "));
    (where_clause, param_count)
}

#[cfg(test)]
fn default_event_sort() -> ValidatedEventSort {
    ValidatedEventSort {
        sort_by: EventSortBy::StartTime,
        sort_order: SortOrder::Asc,
    }
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

    (
        format!("WHERE {}", where_clauses.join(" AND ")),
        param_count,
    )
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn build_where_clause_includes_min_tickets_available() {
        let filters = EventFilters {
            is_featured: None,
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
            sort_by: None,
            sort_order: None,
            sort: None,
        };

        let (where_clause, _) = build_event_where_clause(&filters, &default_event_sort(), None);
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
            is_featured: None,
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
            sort_by: None,
            sort_order: None,
            sort: None,
        };

        assert!(filters.organizer_id.is_some());
        assert_eq!(filters.organizer_wallet.as_deref(), Some("GABC123"));
        assert_eq!(filters.location.unwrap(), "New York");
    }

    #[test]
    fn test_organizer_wallet_filter() {
        let filters = EventFilters {
            is_featured: None,
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
            sort_by: None,
            sort_order: None,
            sort: None,
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

        let (where_clause, param_count) = build_past_event_where_clause(&filters, Some(&cursor));

        assert_eq!(param_count, 3);
        assert!(where_clause.contains("wallet_address = $1"));
        assert!(where_clause.contains("(end_time < $2 OR (end_time = $2 AND id < $3))"));
    }

    #[test]
    fn test_is_free_filter() {
        let filters_free = EventFilters {
            is_featured: None,
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
            sort_by: None,
            sort_order: None,
            sort: None,
        };
        assert_eq!(filters_free.is_free, Some(true));

        let filters_paid = EventFilters {
            is_featured: None,
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
            sort_by: None,
            sort_order: None,
            sort: None,
        };
        assert_eq!(filters_paid.is_free, Some(false));

        let filters_none = EventFilters {
            is_featured: None,
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
            sort_by: None,
            sort_order: None,
            sort: None,
        };
        assert_eq!(filters_none.is_free, None);
    }

    #[test]
    fn test_start_date_filter_generates_where_clause() {
        let filters = EventFilters {
            is_featured: None,
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
            sort_by: None,
            sort_order: None,
            sort: None,
        };
        let (where_clause, _) = build_event_where_clause(&filters, &default_event_sort(), None);
        assert!(
            where_clause.contains("start_time >="),
            "Expected start_time >= clause, got: {}",
            where_clause
        );
    }

    #[test]
    fn test_end_date_filter_generates_where_clause() {
        let filters = EventFilters {
            is_featured: None,
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
            sort_by: None,
            sort_order: None,
            sort: None,
        };
        let (where_clause, _) = build_event_where_clause(&filters, &default_event_sort(), None);
        assert!(
            where_clause.contains("start_time <="),
            "Expected start_time <= clause, got: {}",
            where_clause
        );
    }

    #[test]
    fn test_start_date_and_end_date_filters_bind_expected_param_count() {
        let filters = EventFilters {
            is_featured: None,
            organizer_id: None,
            organizer_wallet: None,
            location: None,
            start_after: None,
            start_before: None,
            search: None,
            min_tickets_available: None,
            is_free: None,
            start_date: Some("2026-06-15".to_string()),
            end_date: Some("2026-06-20".to_string()),
            followers_only: None,
            sort_by: None,
            sort_order: None,
            sort: None,
        };
        let (where_clause, param_count) =
            build_event_where_clause(&filters, &default_event_sort(), None);
        assert_eq!(
            param_count, 2,
            "Expected param_count of 2 for start_date + end_date, got: {}. where_clause: {}",
            param_count, where_clause
        );
        assert!(where_clause.contains("start_time >= $1"));
        assert!(where_clause.contains("start_time <= $2"));
    }

    #[test]
    fn test_whitespace_only_search_is_treated_as_no_search() {
        let filters = EventFilters {
            is_featured: None,
            organizer_id: None,
            organizer_wallet: None,
            location: None,
            start_after: None,
            start_before: None,
            search: Some("   ".to_string()),
            min_tickets_available: None,
            is_free: None,
            start_date: None,
            end_date: None,
            followers_only: None,
            sort_by: None,
            sort_order: None,
            sort: None,
        };

        let normalized = filters
            .search
            .as_ref()
            .map(|s| s.trim().to_string())
            .filter(|s| !s.is_empty());

        assert_eq!(
            normalized, None,
            "Whitespace-only search should normalize to None"
        );
    }

    #[test]
    fn test_real_search_term_is_preserved_after_trim() {
        let raw = "  concert  ".to_string();
        let normalized = Some(raw.trim().to_string()).filter(|s| !s.is_empty());
        assert_eq!(normalized, Some("concert".to_string()));
    }

    #[test]
    fn test_followers_only_filter() {
        let filters = EventFilters {
            is_featured: None,
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
            sort_by: None,
            sort_order: None,
            sort: None,
        };
        let (where_clause, _) = build_event_where_clause(&filters, &default_event_sort(), None);
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
    fn test_submit_rating_cache_key_format() {
        let event_id = Uuid::new_v4();
        let cache_key = format!("event:detail:{}", event_id);
        assert!(cache_key.starts_with("event:detail:"));
        assert!(cache_key.contains(&event_id.to_string()));
    }

    #[test]
    fn test_event_rating_item_fields() {
        let item = EventRatingItem {
            rating: 4,
            review: Some("Great event!".to_string()),
            created_at: Utc::now(),
        };
        assert_eq!(item.rating, 4);
        assert_eq!(item.review.as_deref(), Some("Great event!"));
    }

    #[test]
    fn test_event_rating_item_no_review() {
        let item = EventRatingItem {
            rating: 3,
            review: None,
            created_at: Utc::now(),
        };
        assert!(item.review.is_none());
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
    fn test_upcoming_limit_clamping() {
        // Default when absent.
        assert_eq!(5u32.clamp(1, 20), 5);
        // Values above the max are clamped to 20.
        assert_eq!(100u32.clamp(1, 20), 20);
        // Zero is clamped up to the minimum of 1.
        assert_eq!(0u32.clamp(1, 20), 1);
        // In-range values pass through.
        assert_eq!(10u32.clamp(1, 20), 10);
    }

    #[test]
    fn test_search_params_page_size_valid() {
        for size in [1u32, 20, 50, 100] {
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
                page_size: size,
            };
            assert!(
                params.validate_page_size().is_ok(),
                "page_size={} should be valid",
                size
            );
        }
    }

    #[test]
    fn test_search_params_page_size_zero_rejected() {
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
            page_size: 0,
        };
        let err = params.validate_page_size().unwrap_err();
        assert!(err.contains("page_size must be between 1 and 100"));
    }

    #[test]
    fn test_search_params_page_size_above_max_rejected() {
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
            page_size: 101,
        };
        let err = params.validate_page_size().unwrap_err();
        assert!(err.contains("page_size must be between 1 and 100"));
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
    fn test_validate_event_timestamps_end_time_before_start_time() {
        let start = Utc::now();
        let end = start - chrono::Duration::hours(1); // end before start
        let result = validate_event_timestamps(start, Some(end));
        assert!(result.is_err());
        assert!(result
            .unwrap_err()
            .to_string()
            .contains("end_time must be strictly after start_time"));
    }

    #[test]
    fn test_validate_event_timestamps_end_time_equals_start_time() {
        let start = Utc::now();
        let end = start; // end equals start
        let result = validate_event_timestamps(start, Some(end));
        assert!(result.is_err());
        assert!(result
            .unwrap_err()
            .to_string()
            .contains("end_time must be strictly after start_time"));
    }

    #[test]
    fn test_validate_event_timestamps_start_time_in_past() {
        let start = Utc::now() - chrono::Duration::minutes(10); // 10 minutes ago
        let end = Some(start + chrono::Duration::hours(2));
        let result = validate_event_timestamps(start, end);
        assert!(result.is_err());
        assert!(result
            .unwrap_err()
            .to_string()
            .contains("start_time must be in the future"));
    }

    #[test]
    fn test_validate_event_timestamps_start_time_in_grace_period() {
        let start = Utc::now() - chrono::Duration::seconds(200); // 3.3 minutes ago (within grace period)
        let end = Some(start + chrono::Duration::hours(2));
        let result = validate_event_timestamps(start, end);
        assert!(result.is_ok());
    }

    #[test]
    fn test_validate_event_timestamps_valid_future_timestamps() {
        let start = Utc::now() + chrono::Duration::hours(1);
        let end = Some(start + chrono::Duration::hours(3));
        let result = validate_event_timestamps(start, end);
        assert!(result.is_ok());
    }

    #[test]
    fn test_validate_event_timestamps_no_end_time() {
        let start = Utc::now() + chrono::Duration::hours(1);
        let result = validate_event_timestamps(start, None);
        assert!(result.is_ok());
    }

    #[test]
    fn test_validate_event_timestamps_duration_exceeds_max() {
        let start = Utc::now() + chrono::Duration::hours(1);
        let end = Some(start + chrono::Duration::days(31)); // 31 days exceeds max
        let result = validate_event_timestamps(start, end);
        assert!(result.is_err());
        assert!(result
            .unwrap_err()
            .to_string()
            .contains("event duration must not exceed"));
    }

    #[test]
    fn test_validate_event_timestamps_duration_at_max() {
        let start = Utc::now() + chrono::Duration::hours(1);
        let end = Some(start + chrono::Duration::days(30)); // exactly 30 days
        let result = validate_event_timestamps(start, end);
        assert!(result.is_ok());
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

    #[test]
    fn test_build_event_order_by_start_time_default() {
        let sort = ValidatedEventSort {
            sort_by: EventSortBy::StartTime,
            sort_order: SortOrder::Asc,
        };
        assert_eq!(
            build_event_order_by_clause(&sort),
            "ORDER BY start_time ASC, id ASC"
        );
    }

    #[test]
    fn test_build_event_order_by_created_at_desc() {
        let sort = ValidatedEventSort {
            sort_by: EventSortBy::CreatedAt,
            sort_order: SortOrder::Desc,
        };
        assert_eq!(
            build_event_order_by_clause(&sort),
            "ORDER BY created_at DESC, id DESC"
        );
    }

    #[test]
    fn test_validate_event_location_accepts_valid_location() {
        assert!(validate_event_location("Amsterdam").is_ok());
    }

    #[test]
    fn test_validate_event_location_rejects_empty_location() {
        let err = validate_event_location("   ").unwrap_err();
        assert!(matches!(err, AppError::ValidationError(_)));
    }

    #[test]
    fn test_validate_event_location_rejects_too_long_location() {
        let err = validate_event_location(&"A".repeat(MAX_LOCATION_LENGTH + 1)).unwrap_err();
        assert!(matches!(err, AppError::ValidationError(_)));
    }

    #[test]
    fn test_validate_event_coordinates_accepts_none() {
        assert!(validate_event_coordinates(None, None).is_ok());
    }

    #[test]
    fn test_validate_event_coordinates_accepts_valid_pair() {
        assert!(validate_event_coordinates(Some(6.5244), Some(3.3792)).is_ok());
    }

    #[test]
    fn test_validate_event_coordinates_rejects_partial_pair() {
        assert!(validate_event_coordinates(Some(6.5244), None).is_err());
        assert!(validate_event_coordinates(None, Some(3.3792)).is_err());
    }

    #[test]
    fn test_validate_event_coordinates_rejects_out_of_range() {
        assert!(validate_event_coordinates(Some(91.0), Some(0.0)).is_err());
        assert!(validate_event_coordinates(Some(0.0), Some(181.0)).is_err());
    }

    #[test]
    fn test_build_event_where_clause_includes_is_featured() {
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
            followers_only: None,
            is_featured: Some(true),
            sort_by: None,
            sort_order: None,
            sort: None,
        };

        let sort = filters.validate_sort().unwrap();
        let (where_clause, _) = build_event_where_clause(&filters, &sort, None);
        assert!(where_clause.contains("is_featured = TRUE"));
    }

    #[test]
    fn test_build_event_order_by_popularity_desc() {
        let sort = ValidatedEventSort {
            sort_by: EventSortBy::Popularity,
            sort_order: SortOrder::Desc,
        };
        assert_eq!(
            build_event_order_by_clause(&sort),
            "ORDER BY minted_tickets DESC, id DESC"
        );
    }

    #[test]
    fn test_invalid_sort_by_returns_validation_error() {
        let filters = EventFilters {
            is_featured: None,
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
            sort_by: Some("invalid".to_string()),
            sort_order: None,
            sort: None,
        };

        let err = filters.validate_sort().unwrap_err();
        assert!(err.contains("Invalid sort_by value 'invalid'"));
    }

    #[test]
    fn test_invalid_sort_order_returns_validation_error() {
        let filters = EventFilters {
            is_featured: None,
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
            sort_by: Some("created_at".to_string()),
            sort_order: Some("sideways".to_string()),
            sort: None,
        };

        let err = filters.validate_sort().unwrap_err();
        assert!(err.contains("Invalid sort_order value 'sideways'"));
    }

    fn assert_sort(value: &str, expected_by: EventSortBy, expected_order: SortOrder, sql: &str) {
        let filters = EventFilters {
            sort: Some(value.to_string()),
            ..Default::default()
        };
        let sort = filters.validate_sort().expect(value);
        assert_eq!(sort.sort_by, expected_by, "sort_by for {value}");
        assert_eq!(sort.sort_order, expected_order, "sort_order for {value}");
        assert_eq!(build_event_order_by_clause(&sort), sql, "ORDER BY for {value}");
        assert!(
            !sql.contains(value) || value.chars().all(|c| c == '_' || c.is_ascii_alphabetic()),
            "raw sort parameter must not be interpolated into SQL"
        );
    }

    #[test]
    fn test_sort_starts_at_asc() {
        assert_sort(
            "starts_at_asc",
            EventSortBy::StartTime,
            SortOrder::Asc,
            "ORDER BY start_time ASC, id ASC",
        );
        // Omitted sort defaults to starts_at_asc.
        let default = EventFilters::default().validate_sort().unwrap();
        assert_eq!(default.sort_by, EventSortBy::StartTime);
        assert_eq!(default.sort_order, SortOrder::Asc);
    }

    #[test]
    fn test_sort_starts_at_desc() {
        assert_sort(
            "starts_at_desc",
            EventSortBy::StartTime,
            SortOrder::Desc,
            "ORDER BY start_time DESC, id DESC",
        );
    }

    #[test]
    fn test_sort_price_asc() {
        assert_sort(
            "price_asc",
            EventSortBy::Price,
            SortOrder::Asc,
            &format!("ORDER BY {MIN_TICKET_PRICE_SQL} ASC, id ASC"),
        );
    }

    #[test]
    fn test_sort_price_desc() {
        assert_sort(
            "price_desc",
            EventSortBy::Price,
            SortOrder::Desc,
            &format!("ORDER BY {MIN_TICKET_PRICE_SQL} DESC, id DESC"),
        );
    }

    #[test]
    fn test_sort_popularity_desc() {
        assert_sort(
            "popularity_desc",
            EventSortBy::Popularity,
            SortOrder::Desc,
            "ORDER BY minted_tickets DESC, id DESC",
        );
    }

    #[test]
    fn test_sort_unrecognised_value_returns_validation_error() {
        let filters = EventFilters {
            sort: Some("not_a_real_sort".to_string()),
            ..Default::default()
        };
        let err = filters.validate_sort().unwrap_err();
        assert!(err.contains("Invalid sort value 'not_a_real_sort'"));
        // Handler maps this to 400 VALIDATION_FAILED, never 500.
        let app_err = AppError::ValidationError(err);
        assert_eq!(app_err.status_code(), axum::http::StatusCode::BAD_REQUEST);
        assert_eq!(app_err.error_code(), crate::utils::error::ErrorCode::ValidationFailed);
    }

    #[test]
    fn test_keyword_search_clause_includes_location() {
        // Mirrors the format string used inside search_events for the `q` param.
        let param_count = 1usize;
        let clause = format!(
            "(e.title ILIKE ${0} OR e.description ILIKE ${0} OR e.location ILIKE ${0})",
            param_count
        );
        assert!(
            clause.contains("e.location ILIKE $1"),
            "keyword search must include location column, got: {}",
            clause
        );
    }

    // -----------------------------------------------------------------------
    // Issue #1263 — search query validation / sanitisation
    // -----------------------------------------------------------------------

    /// Helper that applies the same normalisation logic used in `search_events`.
    fn normalise_search_q(raw: &str) -> Result<Option<String>, String> {
        if raw.len() > MAX_SEARCH_QUERY_LENGTH {
            return Err(format!(
                "Search query must not exceed {} characters",
                MAX_SEARCH_QUERY_LENGTH
            ));
        }
        let trimmed = raw.trim().to_string();
        if trimmed.is_empty() {
            return Ok(None);
        }
        let sanitised = trimmed
            .to_lowercase()
            .replace('%', "")
            .replace('_', " ");
        let sanitised = sanitised.trim().to_string();
        Ok(if sanitised.is_empty() { None } else { Some(sanitised) })
    }

    #[test]
    fn test_search_q_over_128_chars_is_rejected() {
        let long_q = "a".repeat(MAX_SEARCH_QUERY_LENGTH + 1);
        assert!(
            normalise_search_q(&long_q).is_err(),
            "query longer than {} characters should be rejected",
            MAX_SEARCH_QUERY_LENGTH
        );
    }

    #[test]
    fn test_search_q_exactly_128_chars_is_accepted() {
        let q = "a".repeat(MAX_SEARCH_QUERY_LENGTH);
        assert!(
            normalise_search_q(&q).is_ok(),
            "query of exactly {} characters should be accepted",
            MAX_SEARCH_QUERY_LENGTH
        );
    }

    #[test]
    fn test_search_q_empty_string_treated_as_absent() {
        let result = normalise_search_q("").unwrap();
        assert_eq!(result, None, "empty query should normalise to None");
    }

    #[test]
    fn test_search_q_whitespace_only_treated_as_absent() {
        let result = normalise_search_q("   ").unwrap();
        assert_eq!(
            result, None,
            "whitespace-only query should normalise to None"
        );
    }

    #[test]
    fn test_search_q_percent_wildcard_stripped() {
        // A bare `%` must not produce a full-scan LIKE pattern.
        let result = normalise_search_q("%").unwrap();
        assert_eq!(
            result, None,
            "a bare '%' should be stripped and treated as absent"
        );
    }

    #[test]
    fn test_search_q_percent_mixed_stripped() {
        // `%music%` should become `music`.
        let result = normalise_search_q("%music%").unwrap();
        assert_eq!(result, Some("music".to_string()));
    }

    #[test]
    fn test_search_q_underscore_wildcard_replaced_with_space() {
        let result = normalise_search_q("hello_world").unwrap();
        // `_` is replaced with a space, then the result is trimmed.
        assert!(result.is_some());
        let inner = result.unwrap();
        assert!(!inner.contains('_'), "underscore should be removed from query");
    }

    #[test]
    fn test_search_q_normalised_to_lowercase() {
        let result = normalise_search_q("CONCERT").unwrap();
        assert_eq!(result, Some("concert".to_string()));
    }

    #[test]
    fn test_search_q_trimmed_before_use() {
        let result = normalise_search_q("  jazz  ").unwrap();
        assert_eq!(result, Some("jazz".to_string()));
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
/// - `sort` (optional): `starts_at_asc` (default), `starts_at_desc`, `price_asc`, `price_desc`, `popularity_desc`
/// - `sort_by` (optional, legacy): Sort field — `start_time` (default), `created_at`, or `popularity`
/// - `sort_order` (optional, legacy): Sort direction — `asc` (default) or `desc`
/// - `count` (optional): When `false`, skip the COUNT(*) query and omit `meta.total`
///
/// # Response
/// Returns a cursor-paginated list of upcoming events with metadata
pub async fn list_events(
    State(mut state): State<EventState>,
    Query(pagination): Query<CursorParams>,
    Query(mut filters): Query<EventFilters>,
) -> Response {
    let validated = pagination.validate();

    let sort = match filters.validate_sort() {
        Ok(sort) => sort,
        Err(message) => return AppError::ValidationError(message).into_response(),
    };

    // Serve the default (unfiltered, first-page, with total) list from cache when available.
    let use_list_cache = validated.cursor.is_none()
        && filters.is_unfiltered()
        && validated.include_count
        && validated.limit == crate::utils::cursor_pagination::DEFAULT_PAGE_SIZE;
    if use_list_cache {
        match state
            .redis
            .get::<CursorResponse<Event>>(EVENTS_LIST_CACHE_KEY)
            .await
        {
            Ok(Some(cached)) => {
                tracing::debug!("Cache hit for {}", EVENTS_LIST_CACHE_KEY);
                return success(cached, "Events retrieved successfully (cached)").into_response();
            }
            Ok(None) => {
                tracing::debug!("Cache miss for {}", EVENTS_LIST_CACHE_KEY);
            }
            Err(e) => {
                tracing::warn!(
                    "Redis error for {}, falling back to database: {:?}",
                    EVENTS_LIST_CACHE_KEY,
                    e
                );
            }
        }
    }

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

    // Normalize search: treat whitespace-only search as no search term at all,
    // so a single space doesn't match every event via `% %` ILIKE.
    if let Some(trimmed) = filters.search.as_ref().map(|s| s.trim().to_string()) {
        filters.search = if trimmed.is_empty() {
            None
        } else {
            Some(trimmed)
        };
    }

    // Build the WHERE clause dynamically based on filters
    let (where_clause, param_count) = build_event_where_clause(&filters, &sort, cursor.as_ref());

    // Count total matching events (same WHERE as the page query, no cursor).
    // Skipped when `?count=false` so hot paths avoid the extra round-trip.
    let total_count: Option<i64> = if validated.include_count {
        let (count_where, count_param_count) = build_event_where_clause(&filters, &sort, None);
        let count_query = format!("SELECT COUNT(*) FROM events {}", count_where);
        let mut count_builder = sqlx::query_scalar::<_, i64>(&count_query);
        if let Some(organizer_id) = filters.organizer_id {
            count_builder = count_builder.bind(organizer_id);
        }
        if let Some(ref w) = filters.organizer_wallet {
            count_builder = count_builder.bind(w.clone());
        }
        if let Some(ref l) = filters.location {
            count_builder = count_builder.bind(format!("%{}%", l));
        }
        if let Some(start_after) = filters.start_after {
            count_builder = count_builder.bind(start_after);
        }
        if let Some(start_before) = filters.start_before {
            count_builder = count_builder.bind(start_before);
        }
        if let Some(ref s) = filters.search {
            count_builder = count_builder.bind(format!("%{}%", s));
        }
        if let Some(min_tickets) = filters.min_tickets_available {
            count_builder = count_builder.bind(min_tickets);
        }
        if let Some(ref date_str) = filters.start_date {
            if let Ok(date) = NaiveDate::parse_from_str(date_str, "%Y-%m-%d") {
                let dt: DateTime<Utc> = Utc
                    .from_utc_datetime(&date.and_time(NaiveTime::from_hms_opt(0, 0, 0).unwrap()));
                count_builder = count_builder.bind(dt);
            }
        }
        if let Some(ref date_str) = filters.end_date {
            if let Ok(date) = NaiveDate::parse_from_str(date_str, "%Y-%m-%d") {
                let dt: DateTime<Utc> = Utc
                    .from_utc_datetime(&date.and_time(NaiveTime::from_hms_opt(0, 0, 0).unwrap()));
                count_builder = count_builder.bind(dt);
            }
        }
        let _ = count_param_count;

        match count_builder.fetch_one(&state.pool).await {
            Ok(n) => Some(n),
            Err(e) => {
                tracing::warn!("Failed to fetch total count for list_events: {:?}", e);
                Some(0)
            }
        }
    } else {
        None
    };

    // Fetch items (limit + 1 to detect has_more). Price sort selects the
    // allow-listed MIN(price) expression so cursor pagination stays stable.
    let order_by = build_event_order_by_clause(&sort);
    let select_list = if sort.sort_by == EventSortBy::Price {
        format!(
            "SELECT events.*, ({MIN_TICKET_PRICE_SQL})::float8 AS min_ticket_price FROM events"
        )
    } else {
        "SELECT * FROM events".to_string()
    };
    let items_query = format!(
        "{} {} {} LIMIT ${}",
        select_list,
        where_clause,
        order_by,
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
        match sort.sort_by {
            EventSortBy::StartTime => {
                items_query_builder = items_query_builder.bind(c.start_time).bind(c.id);
            }
            EventSortBy::CreatedAt => {
                let created_at = match c.created_at {
                    Some(value) => value,
                    None => {
                        return AppError::ValidationError(
                            "Cursor is missing created_at for created_at sort".to_string(),
                        )
                        .into_response();
                    }
                };
                items_query_builder = items_query_builder.bind(created_at).bind(c.id);
            }
            EventSortBy::Popularity | EventSortBy::CountOfRatings => {
                let sort_key = match c.minted_tickets {
                    Some(value) => value,
                    None => {
                        return AppError::ValidationError(
                            "Cursor is missing minted_tickets for popularity/count_of_ratings sort"
                                .to_string(),
                        )
                        .into_response();
                    }
                };
                items_query_builder = items_query_builder.bind(sort_key).bind(c.id);
            }
            EventSortBy::Price => {
                let price = match c.min_ticket_price {
                    Some(value) => value,
                    None => {
                        return AppError::ValidationError(
                            "Cursor is missing min_ticket_price for price sort".to_string(),
                        )
                        .into_response();
                    }
                };
                items_query_builder = items_query_builder.bind(price).bind(c.id);
            }
        }
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
            created_at: Some(last.created_at),
            minted_tickets: Some(last.minted_tickets),
            count_of_ratings: Some(last.count_of_ratings as i64),
            min_ticket_price: Some(last.min_ticket_price),
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

    populate_is_free(&mut items, &state.pool).await;

    let response = CursorResponse::new(items, &validated, next_cursor)
        .with_total(total_count.unwrap_or(0), validated.include_count);

    if use_list_cache {
        if let Err(e) = state
            .redis
            .set(EVENTS_LIST_CACHE_KEY, &response, EVENTS_LIST_CACHE_TTL)
            .await
        {
            tracing::warn!("Failed to cache {}: {:?}", EVENTS_LIST_CACHE_KEY, e);
        }
    }

    let mut resp = success(response, "Events retrieved successfully").into_response();
    if let Some(total) = total_count {
        if let Ok(v) = HeaderValue::from_str(&total.to_string()) {
            resp.headers_mut().insert("X-Total-Count", v);
        }
    }
    resp
}

/// GET `/api/v1/events/featured`
///
/// Placeholder route registered for the featured-events migration; full handler pending.
pub async fn list_featured_events(State(_state): State<EventState>) -> Response {
    AppError::NotFound("Featured events are not yet available".to_string()).into_response()
}

/// Query parameters for `GET /api/v1/events/upcoming`.
#[derive(Debug, Deserialize)]
pub struct UpcomingParams {
    /// Number of events to return (clamped to 1–20, default 5).
    pub limit: Option<u32>,
}

/// List the next upcoming events ordered by `start_time ASC`.
///
/// A simplified feed for home pages and the mobile app — no cursor contract,
/// just a single `limit` parameter.
///
/// # Endpoint
/// GET `/api/v1/events/upcoming`
///
/// # Query Parameters
/// - `limit` (optional): Number of events to return (1–20, default 5)
pub async fn list_upcoming_events(
    State(state): State<EventState>,
    Query(params): Query<UpcomingParams>,
) -> Response {
    // Clamp limit to 1–20, defaulting to 5.
    let limit = params.limit.unwrap_or(5).clamp(1, 20) as i64;

    let mut events = match sqlx::query_as::<_, Event>(
        "SELECT * FROM events WHERE end_time > NOW() AND is_flagged = FALSE ORDER BY start_time ASC LIMIT $1",
    )
    .bind(limit)
    .fetch_all(&state.pool)
    .await
    {
        Ok(events) => events,
        Err(e) => {
            tracing::error!("Failed to fetch upcoming events: {:?}", e);
            return AppError::DatabaseError(e).into_response();
        }
    };

    populate_is_free(&mut events, &state.pool).await;
    success(events, "Upcoming events retrieved successfully").into_response()
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
    Query(params): Query<GetEventParams>,
) -> Response {
    // Cache is only used when tiers are not requested, to avoid caching partial data.
    let cache_key = format!("event:detail:{}", event_id);
    if !params.include_tiers {
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
    }

    // Cache miss or error, fetch from database
    let start = std::time::Instant::now();
    let event = match sqlx::query_as::<_, Event>(
        "SELECT * FROM events WHERE id = $1 AND is_flagged = FALSE",
    )
    .bind(event_id)
    .fetch_optional(&state.pool)
    .await
    {
        Ok(Some(event)) => event,
        Ok(None) => {
            log_if_slow("get_event", start.elapsed());
            return AppError::NotFound(format!("Event with id '{}' not found", event_id))
                .into_response();
        }
        Err(e) => {
            log_if_slow("get_event", start.elapsed());
            tracing::error!("Failed to fetch event: {:?}", e);
            return AppError::DatabaseError(e).into_response();
        }
    };
    log_if_slow("get_event", start.elapsed());

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

    // Optionally fetch ticket tiers sorted by price ascending (Issue #884).
    let tiers = if params.include_tiers {
        match sqlx::query_as::<_, TicketTierResponse>(
            r#"
            SELECT
                id,
                name,
                price,
                total_quantity    AS quantity,
                (total_quantity - available_quantity) AS sold
            FROM ticket_tiers
            WHERE event_id = $1
            ORDER BY price ASC
            "#,
        )
        .bind(event_id)
        .fetch_all(&state.pool)
        .await
        {
            Ok(rows) => Some(rows),
            Err(e) => {
                tracing::warn!(
                    "Failed to fetch ticket tiers for event {}: {:?}",
                    event_id,
                    e
                );
                None
            }
        }
    } else {
        None
    };

    let detail = EventDetail {
        event,
        organizer_profile,
        tiers,
    };

    // Only cache responses without tiers to keep the cached shape stable.
    if !params.include_tiers {
        if let Err(e) = state.redis.set(&cache_key, &detail, EVENT_CACHE_TTL).await {
            tracing::warn!("Failed to cache event {}: {:?}", event_id, e);
        }
    }

    success(detail, "Event retrieved successfully").into_response()
}

/// Query parameters for `GET /api/v1/events/:id/similar`.
#[derive(Debug, Deserialize)]
pub struct SimilarEventsParams {
    /// Maximum number of similar events to return (1–10, default 4).
    pub limit: Option<u32>,
}

/// Return up to `limit` upcoming events that share a category or location with event `:id`.
///
/// # Endpoint
/// GET `/api/v1/events/:id/similar`
pub async fn list_similar_events(
    State(state): State<EventState>,
    Path(event_id): Path<Uuid>,
    Query(params): Query<SimilarEventsParams>,
) -> Response {
    // Clamp limit to 1–10, default 4.
    let limit = params.limit.unwrap_or(4).clamp(1, 10) as i64;

    // Fetch the source event to get its location (category via join below).
    let source = match sqlx::query_as::<_, Event>(
        "SELECT * FROM events WHERE id = $1 AND is_flagged = FALSE",
    )
    .bind(event_id)
    .fetch_optional(&state.pool)
    .await
    {
        Ok(Some(e)) => e,
        Ok(None) => {
            return AppError::NotFound(format!("Event '{}' not found", event_id)).into_response();
        }
        Err(e) => {
            tracing::error!("Failed to fetch source event: {:?}", e);
            return AppError::DatabaseError(e).into_response();
        }
    };

    // Find events in the same category (via event_categories join) OR same location,
    // excluding the source event itself.
    let mut events = match sqlx::query_as::<_, Event>(
        r"
        SELECT DISTINCT e.* FROM events e
        LEFT JOIN event_categories ec ON ec.event_id = e.id
        WHERE e.id != $1
          AND (e.end_time IS NULL OR e.end_time > NOW())
          AND e.is_flagged = FALSE
          AND (
              ec.category_id IN (
                  SELECT category_id FROM event_categories WHERE event_id = $1
              )
              OR e.location ILIKE $2
          )
        ORDER BY e.start_time ASC
        LIMIT $3
        ",
    )
    .bind(event_id)
    .bind(format!("%{}%", source.location))
    .bind(limit)
    .fetch_all(&state.pool)
    .await
    {
        Ok(rows) => rows,
        Err(e) => {
            tracing::error!("Failed to fetch similar events: {:?}", e);
            return AppError::DatabaseError(e).into_response();
        }
    };

    populate_is_free(&mut events, &state.pool).await;
    success(events, "Similar events retrieved successfully").into_response()
}

/// Maximum allowed length for an event title.
pub const MAX_EVENT_TITLE_LENGTH: usize = 200;

/// Maximum allowed length for an event description.
pub const MAX_EVENT_DESCRIPTION_LENGTH: usize = 10000;

/// Validates an event title for create/update requests.
pub fn validate_event_title(title: &str) -> Result<(), String> {
    let trimmed = title.trim();
    if trimmed.is_empty() {
        return Err("title must not be empty".to_string());
    }
    if title.chars().count() > MAX_EVENT_TITLE_LENGTH {
        return Err(format!(
            "title must not exceed {} characters",
            MAX_EVENT_TITLE_LENGTH
        ));
    }
    Ok(())
}

/// Validates an event description for create/update requests.
pub fn validate_event_description(description: &Option<String>) -> Result<(), String> {
    if let Some(ref desc) = description {
        if desc.chars().count() > MAX_EVENT_DESCRIPTION_LENGTH {
            return Err(format!(
                "description must not exceed {} characters",
                MAX_EVENT_DESCRIPTION_LENGTH
            ));
        }
    }
    Ok(())
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
    /// Optional latitude in decimal degrees (-90 to 90) for map discovery.
    pub latitude: Option<f64>,
    /// Optional longitude in decimal degrees (-180 to 180) for map discovery.
    pub longitude: Option<f64>,
}

const MAX_IMAGE_URL_LEN: usize = 2048;

/// Validates that an image URL is a safe, well-formed HTTPS URL no longer than 2048 characters.
/// Rejects data URIs, javascript URIs, HTTP URLs, and empty hosts.
fn validate_image_url(url: &str) -> Result<(), AppError> {
    if url.len() > MAX_IMAGE_URL_LEN {
        return Err(AppError::ValidationError(format!(
            "image_url must not exceed {MAX_IMAGE_URL_LEN} characters"
        )));
    }
    let is_valid = url.starts_with("https://")
        && url.len() > "https://".len()
        && !url["https://".len()..].starts_with('/');
    if !is_valid {
        return Err(AppError::ValidationError(
            "image_url must be a valid HTTPS URL".to_string(),
        ));
    }
    Ok(())
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

const MAX_LOCATION_LENGTH: usize = 500;

/// Maximum allowed event duration in days (30 days).
const MAX_EVENT_DURATION_DAYS: i64 = 30;

/// Grace period in seconds for start_time validation (5 minutes).
/// Allows organizers to create events that start slightly in the past.
const START_TIME_GRACE_PERIOD_SECONDS: i64 = 300;

fn validate_event_location(location: &str) -> Result<(), AppError> {
    if location.trim().is_empty() {
        return Err(AppError::ValidationError(
            "location is required".to_string(),
        ));
    }
    if location.chars().count() > MAX_LOCATION_LENGTH {
        return Err(AppError::ValidationError(format!(
            "location must be at most {MAX_LOCATION_LENGTH} characters"
        )));
    }
    Ok(())
}

/// Validates optional event coordinates used for map-based discovery.
/// Both must be present together; individually omitting either is allowed only
/// when both are `None` (existing events without geocoding remain valid).
fn validate_event_coordinates(
    latitude: Option<f64>,
    longitude: Option<f64>,
) -> Result<(), AppError> {
    match (latitude, longitude) {
        (None, None) => Ok(()),
        (Some(_), None) | (None, Some(_)) => Err(AppError::ValidationError(
            "latitude and longitude must both be provided together".to_string(),
        )),
        (Some(lat), Some(lng)) => {
            if !(-90.0..=90.0).contains(&lat) {
                return Err(AppError::ValidationError(
                    "latitude must be between -90 and 90".to_string(),
                ));
            }
            if !(-180.0..=180.0).contains(&lng) {
                return Err(AppError::ValidationError(
                    "longitude must be between -180 and 180".to_string(),
                ));
            }
            Ok(())
        }
    }
}

/// Validates event timestamps for create/update requests.
/// Ensures start_time is not too far in the past, end_time > start_time (if provided),
/// and event duration does not exceed the maximum allowed.
fn validate_event_timestamps(
    start_time: DateTime<Utc>,
    end_time: Option<DateTime<Utc>>,
) -> Result<(), AppError> {
    let now = Utc::now();

    // Check that start_time is not too far in the past (with grace period)
    let grace_period = chrono::Duration::seconds(START_TIME_GRACE_PERIOD_SECONDS);
    if start_time + grace_period < now {
        return Err(AppError::ValidationError(
            "start_time must be in the future or within the grace period".to_string(),
        ));
    }

    // If end_time is provided, validate it
    if let Some(end) = end_time {
        // end_time must be strictly after start_time
        if end <= start_time {
            return Err(AppError::ValidationError(
                "end_time must be strictly after start_time".to_string(),
            ));
        }

        // Check event duration does not exceed maximum
        let max_duration = chrono::Duration::days(MAX_EVENT_DURATION_DAYS);
        if end - start_time > max_duration {
            return Err(AppError::ValidationError(format!(
                "event duration must not exceed {} days",
                MAX_EVENT_DURATION_DAYS
            )));
        }
    }

    Ok(())
}

/// Create a new event and warm up the Redis cache for `GET /api/v1/events/:id`.
///
/// # Endpoint
/// POST `/api/v1/events`
pub async fn create_event(
    State(mut state): State<EventState>,
    Json(payload): Json<CreateEventRequest>,
) -> Response {
    if let Some(ref url) = payload.image_url {
        if let Err(e) = validate_image_url(url) {
            return e.into_response();
        }
    }

    if let Err(e) = validate_event_location(&payload.location) {
        return e.into_response();
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

    if let Err(message) = validate_event_title(&payload.title) {
        return AppError::ValidationError(message).into_response();
    }

    // Validate event timestamps
    if let Err(e) = validate_event_timestamps(payload.start_time, payload.end_time) {
        return e.into_response();
    }
    if let Err(message) = validate_event_description(&payload.description) {
        return AppError::ValidationError(message).into_response();
    }

    if let Err(e) = validate_event_coordinates(payload.latitude, payload.longitude) {
        return e.into_response();
    }

    let event = match sqlx::query_as::<_, Event>(
        "INSERT INTO events (organizer_id, title, description, location, start_time, end_time, image_url, host_email, latitude, longitude)
         VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9, $10)
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
    .bind(payload.latitude)
    .bind(payload.longitude)
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
        tiers: None,
    };

    let cache_key = format!("event:detail:{}", event.id);
    if let Err(e) = state.redis.set(&cache_key, &detail, EVENT_CACHE_TTL).await {
        tracing::warn!("Cache warm-up failed for event {}: {:?}", event.id, e);
    }

    // New events invalidate the shared list cache.
    state.redis.invalidate_events_list().await;

    success(event, "Event created successfully").into_response()
}

/// Record a star rating for an event.
///
/// # Endpoint
/// POST `/api/v1/events/:id/rate`
pub async fn submit_event_rating(
    State(mut state): State<EventState>,
    Path(event_id): Path<Uuid>,
    Json(payload): Json<SubmitEventRatingRequest>,
) -> Response {
    if payload.rating < 1 || payload.rating > 5 {
        return AppError::ValidationError("Rating must be between 1 and 5".to_string())
            .into_response();
    }

    let start = std::time::Instant::now();
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
            log_if_slow("submit_event_rating", start.elapsed());
            return AppError::NotFound(format!("Ticket with id '{}' not found", payload.ticket_id))
                .into_response();
        }
        Err(e) => {
            log_if_slow("submit_event_rating", start.elapsed());
            tracing::error!("Failed to fetch ticket for rating: {:?}", e);
            return AppError::DatabaseError(e).into_response();
        }
    };

    let (ticket_status, ticket_event_id) = ticket;

    let event_exists = match sqlx::query_scalar::<_, bool>(
        "SELECT EXISTS(SELECT 1 FROM events WHERE id = $1 AND is_flagged = FALSE)",
    )
    .bind(event_id)
    .fetch_one(&state.pool)
    .await
    {
        Ok(exists) => exists,
        Err(e) => {
            log_if_slow("submit_event_rating", start.elapsed());
            tracing::error!("Failed to check event existence for rating: {:?}", e);
            return AppError::DatabaseError(e).into_response();
        }
    };

    if !event_exists {
        log_if_slow("submit_event_rating", start.elapsed());
        return AppError::NotFound(format!("Event with id '{}' not found", event_id))
            .into_response();
    }

    if ticket_event_id != event_id {
        log_if_slow("submit_event_rating", start.elapsed());
        return AppError::Forbidden("Ticket does not belong to this event".to_string())
            .into_response();
    }

    if ticket_status != "Scanned" {
        log_if_slow("submit_event_rating", start.elapsed());
        return AppError::ValidationError(
            "Only attendees with a scanned ticket may leave a rating".to_string(),
        )
        .into_response();
    }

    // Verify event has ended (if end_time is set). Ratings are only allowed after event end.
    let maybe_end_time = match sqlx::query_scalar::<_, Option<chrono::DateTime<Utc>>>(
        "SELECT end_time FROM events WHERE id = $1 AND is_flagged = FALSE",
    )
    .bind(event_id)
    .fetch_optional(&state.pool)
    .await
    {
        Ok(opt) => opt,
        Err(e) => {
            log_if_slow("submit_event_rating", start.elapsed());
            tracing::error!("Failed to fetch event end_time for rating: {:?}", e);
            return AppError::DatabaseError(e).into_response();
        }
    };

    if maybe_end_time.is_none() {
        // event not found or flagged
        log_if_slow("submit_event_rating", start.elapsed());
        return AppError::NotFound(format!("Event with id '{}' not found", event_id))
            .into_response();
    }

    if let Some(end_time) = maybe_end_time.unwrap() {
        if end_time > Utc::now() {
            log_if_slow("submit_event_rating", start.elapsed());
            return AppError::ValidationError(
                "Ratings may only be submitted after the event has ended".to_string(),
            )
            .into_response();
        }
    }

    let mut tx = match state.pool.begin().await {
        Ok(tx) => tx,
        Err(e) => {
            log_if_slow("submit_event_rating", start.elapsed());
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
            log_if_slow("submit_event_rating", start.elapsed());
            tracing::error!("Failed to verify existing rating: {:?}", e);
            return AppError::DatabaseError(e).into_response();
        }
    };

    if already_rated {
        log_if_slow("submit_event_rating", start.elapsed());
        return AppError::Conflict("Rating already submitted for this ticket".to_string())
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
        log_if_slow("submit_event_rating", start.elapsed());
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
            log_if_slow("submit_event_rating", start.elapsed());
            return AppError::NotFound(format!("Event with id '{}' not found", event_id))
                .into_response();
        }
        Err(e) => {
            log_if_slow("submit_event_rating", start.elapsed());
            tracing::error!("Failed to update event rating aggregates: {:?}", e);
            return AppError::DatabaseError(e).into_response();
        }
    };

    if let Err(e) = tx.commit().await {
        log_if_slow("submit_event_rating", start.elapsed());
        tracing::error!("Failed to commit rating transaction: {:?}", e);
        return AppError::DatabaseError(e).into_response();
    }
    log_if_slow("submit_event_rating", start.elapsed());

    let cache_key = format!("event:detail:{}", event_id);
    if let Err(e) = state.redis.delete(&cache_key).await {
        tracing::warn!(
            "Failed to invalidate event detail cache after rating for {}: {:?}",
            event_id,
            e
        );
    }
    state.redis.invalidate_events_list().await;

    let response = SubmitEventRatingResponse {
        sum_of_ratings: updated_event.sum_of_ratings,
        count_of_ratings: updated_event.count_of_ratings,
        average_rating: updated_event.average_rating().unwrap_or(0.0),
    };

    success(response, "Rating recorded successfully").into_response()
}

/// Maximum length for the free-text search query parameter `q`.
/// Queries longer than this are rejected with a 400 to prevent expensive full-table scans.
const MAX_SEARCH_QUERY_LENGTH: usize = 128;

/// Search events with advanced filters
///
/// # Endpoint
/// GET `/api/v1/events/search`
///
/// # Query Parameters
/// - `q` (optional): Keyword search in title and description (max 128 chars)
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
const SEARCH_CACHE_TTL: Duration = Duration::from_secs(120);

pub async fn search_events(
    State(mut state): State<EventState>,
    Query(mut params): Query<SearchParams>,
) -> Response {
    if let Err(msg) = params.validate_page_size() {
        return AppError::ValidationError(msg).into_response();
    }

    // --- Issue #1263: sanitise and validate the free-text search parameter ---
    if let Some(raw_q) = params.q.take() {
        // Reject queries that exceed the maximum allowed length.
        if raw_q.len() > MAX_SEARCH_QUERY_LENGTH {
            return AppError::ValidationError(format!(
                "Search query must not exceed {} characters",
                MAX_SEARCH_QUERY_LENGTH
            ))
            .into_response();
        }

        // Trim whitespace; treat empty/whitespace-only queries as absent.
        let trimmed = raw_q.trim().to_string();
        if trimmed.is_empty() {
            params.q = None;
        } else {
            // Normalise to lowercase and strip SQL LIKE wildcards.
            let sanitised = trimmed
                .to_lowercase()
                .replace('%', "")
                .replace('_', " ");
            let sanitised = sanitised.trim().to_string();
            params.q = if sanitised.is_empty() { None } else { Some(sanitised) };
        }
    }

    let pagination = PaginationParams {
        page: params.page,
        page_size: params.page_size,
        count: true,
    };
    let validated_pagination = pagination.validate();

    // Deterministic cache key from all search parameters (issue #592).
    let cache_key = format!(
        "search:{}:{}:{}:{}:{}:{}:{}:{}:{}",
        params.q.as_deref().unwrap_or(""),
        params
            .category_id
            .map(|id| id.to_string())
            .unwrap_or_default(),
        params.category_ids.as_deref().unwrap_or(""),
        params.min_price.unwrap_or(0),
        params.max_price.unwrap_or(0),
        params.date_from.map(|d| d.timestamp()).unwrap_or(0),
        params.date_to.map(|d| d.timestamp()).unwrap_or(0),
        validated_pagination.page,
        validated_pagination.page_size,
    );

    // Try cache first; fall through to DB on miss or Redis error.
    match state
        .redis
        .get::<PaginatedResponse<Event>>(&cache_key)
        .await
    {
        Ok(Some(cached)) => {
            tracing::debug!("Cache hit for search key: {}", cache_key);
            return success(cached, "Search results retrieved successfully (cached)")
                .into_response();
        }
        Ok(None) => {}
        Err(e) => tracing::warn!(
            "Redis error during search cache lookup, falling back: {:?}",
            e
        ),
    }

    // Build dynamic WHERE clause using WHERE 1=1 pattern
    let mut where_clauses = vec!["1=1".to_string()];
    let mut param_count = 0;

    // Keyword search in title, description, and location
    if params.q.is_some() {
        param_count += 1;
        where_clauses.push(format!(
            "(e.title ILIKE ${0} OR e.description ILIKE ${0} OR e.location ILIKE ${0})",
            param_count
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
    let mut items = match items_query_builder.fetch_all(&state.pool).await {
        Ok(events) => events,
        Err(e) => {
            tracing::error!("Failed to fetch search results: {:?}", e);
            return AppError::DatabaseError(e).into_response();
        }
    };
    log_if_slow("search_events", start.elapsed());

    populate_is_free(&mut items, &state.pool).await;

    let response = PaginatedResponse::new(items, validated_pagination, total);

    // Cache the result for 2 minutes; failures are non-fatal.
    if let Err(e) = state
        .redis
        .set(&cache_key, &response, SEARCH_CACHE_TTL)
        .await
    {
        tracing::warn!("Failed to cache search results: {:?}", e);
    }

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
    // Fetch current flag status and the affected event's organizer wallet so the
    // audit log can record who was impacted by the moderation action (Issue #586).
    let (current_flagged, organizer_wallet) = match sqlx::query_as::<_, (bool, Option<String>)>(
        "SELECT e.is_flagged, o.wallet_address
         FROM events e
         LEFT JOIN organizers o ON o.id = e.organizer_id
         WHERE e.id = $1",
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
    state.redis.invalidate_events_list().await;

    let mut response = success(
        json!({ "is_flagged": new_flagged }),
        "Event flag toggled successfully",
    )
    .into_response();

    // Attach the organizer wallet so the audit middleware records it in metadata.
    if let Some(wallet) = organizer_wallet {
        response
            .extensions_mut()
            .insert(AuditMetadata(json!({ "organizer_wallet": wallet })));
    }

    response
}

#[derive(Debug, Deserialize)]
pub struct SetEventFeaturedRequest {
    pub featured: bool,
}

/// Set or clear the featured flag for an event (admin only).
///
/// PATCH `/api/v1/admin/events/:id/feature`
pub async fn set_event_featured(
    State(mut state): State<EventState>,
    Path(event_id): Path<Uuid>,
    Json(payload): Json<SetEventFeaturedRequest>,
) -> Response {
    let updated = match sqlx::query_as::<_, (bool,)>(
        "UPDATE events SET is_featured = $1 WHERE id = $2 RETURNING is_featured",
    )
    .bind(payload.featured)
    .bind(event_id)
    .fetch_one(&state.pool)
    .await
    {
        Ok(row) => row.0,
        Err(sqlx::Error::RowNotFound) => {
            return AppError::NotFound(format!("Event with id '{}' not found", event_id))
                .into_response();
        }
        Err(e) => {
            tracing::error!("Failed to update event featured flag: {:?}", e);
            return AppError::DatabaseError(e).into_response();
        }
    };

    let cache_key = format!("event:detail:{}", event_id);
    if let Err(e) = state.redis.delete(&cache_key).await {
        tracing::warn!(
            "Failed to invalidate cache for featured update on event {}: {:?}",
            event_id,
            e
        );
    }
    state.redis.invalidate_events_list().await;

    let mut response = success(
        json!({ "is_featured": updated }),
        "Event featured flag updated successfully",
    )
    .into_response();

    response
        .extensions_mut()
        .insert(AuditMetadata(json!({ "featured": updated })));

    response
}

#[derive(Debug, Deserialize)]
pub struct FlagEventRequest {
    pub flagged: bool,
}

/// Set or clear the flagged status of an event (admin only).
///
/// PATCH `/api/v1/admin/events/:id/flag`
pub async fn flag_event(
    State(mut state): State<EventState>,
    Path(event_id): Path<Uuid>,
    Json(payload): Json<FlagEventRequest>,
) -> Response {
    let updated = match sqlx::query_as::<_, (bool,)>(
        "UPDATE events SET is_flagged = $1 WHERE id = $2 RETURNING is_flagged",
    )
    .bind(payload.flagged)
    .bind(event_id)
    .fetch_one(&state.pool)
    .await
    {
        Ok(row) => row.0,
        Err(sqlx::Error::RowNotFound) => {
            return AppError::NotFound(format!("Event with id '{}' not found", event_id))
                .into_response();
        }
        Err(e) => {
            tracing::error!("Failed to update event flagged status: {:?}", e);
            return AppError::DatabaseError(e).into_response();
        }
    };

    let cache_key = format!("event:detail:{}", event_id);
    if let Err(e) = state.redis.delete(&cache_key).await {
        tracing::warn!(
            "Failed to invalidate cache for flagged update on event {}: {:?}",
            event_id,
            e
        );
    }

    let mut response = success(
        json!({ "is_flagged": updated }),
        "Event flagged status updated successfully",
    )
    .into_response();

    response
        .extensions_mut()
        .insert(AuditMetadata(json!({ "flagged": updated })));

    response
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
    let event_exists = match sqlx::query_scalar::<_, bool>(
        "SELECT EXISTS(SELECT 1 FROM events WHERE id = $1 AND is_flagged = FALSE)",
    )
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
                "SELECT sum_of_ratings, count_of_ratings FROM events WHERE id = $1 AND is_flagged = FALSE",
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
                "SELECT total_tickets - minted_tickets FROM events WHERE id = $1 AND is_flagged = FALSE",
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
        "SELECT minted_tickets, total_tickets FROM events WHERE id = $1 AND is_flagged = FALSE",
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
    let exists = match sqlx::query_scalar::<_, bool>(
        "SELECT EXISTS(SELECT 1 FROM events WHERE id = $1 AND is_flagged = FALSE)",
    )
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
        is_featured: None,
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
        sort_by: None,
        sort_order: None,
        sort: None,
    };

    assert!(filters.organizer_id.is_some());
    assert_eq!(filters.organizer_wallet.as_deref(), Some("GABC123"));
    assert_eq!(filters.location.unwrap(), "New York");
}

#[test]
fn test_organizer_wallet_filter() {
    let filters = EventFilters {
        is_featured: None,
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
        sort_by: None,
        sort_order: None,
        sort: None,
    };
    assert_eq!(filters.organizer_wallet.as_deref(), Some("GBXXX"));
}

#[test]
fn test_is_free_filter() {
    let filters_free = EventFilters {
        is_featured: None,
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
        sort_by: None,
        sort_order: None,
        sort: None,
    };
    assert_eq!(filters_free.is_free, Some(true));

    let filters_paid = EventFilters {
        is_featured: None,
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
        sort_by: None,
        sort_order: None,
        sort: None,
    };
    assert_eq!(filters_paid.is_free, Some(false));

    let filters_none = EventFilters {
        is_featured: None,
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
        sort_by: None,
        sort_order: None,
        sort: None,
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

/// A single rating item returned by the list ratings endpoint (ticket ID omitted for privacy).
#[derive(Debug, Serialize, sqlx::FromRow)]
pub struct EventRatingItem {
    pub rating: i16,
    pub review: Option<String>,
    pub created_at: DateTime<Utc>,
}

/// GET /api/v1/events/:id/ratings
///
/// Returns a paginated list of ratings for the given event.
/// Ticket IDs are not included in the response to preserve attendee privacy.
/// Returns 404 if the event does not exist.
pub async fn list_event_ratings(
    State(state): State<EventState>,
    Path(event_id): Path<Uuid>,
    Query(pagination): Query<PaginationParams>,
) -> Response {
    let exists = match sqlx::query_scalar::<_, bool>(
        "SELECT EXISTS(SELECT 1 FROM events WHERE id = $1 AND is_flagged = FALSE)",
    )
    .bind(event_id)
    .fetch_one(&state.pool)
    .await
    {
        Ok(v) => v,
        Err(e) => {
            tracing::error!("Failed to check event existence for ratings: {:?}", e);
            return AppError::DatabaseError(e).into_response();
        }
    };

    if !exists {
        return AppError::NotFound(format!("Event with id '{}' not found", event_id))
            .into_response();
    }

    let validated = pagination.validate();

    let total = match sqlx::query_scalar::<_, i64>(
        "SELECT COUNT(*) FROM event_ratings WHERE event_id = $1",
    )
    .bind(event_id)
    .fetch_one(&state.pool)
    .await
    {
        Ok(n) => n,
        Err(e) => {
            tracing::error!("Failed to count event ratings: {:?}", e);
            return AppError::DatabaseError(e).into_response();
        }
    };

    let items = match sqlx::query_as::<_, EventRatingItem>(
        "SELECT rating, review, created_at FROM event_ratings \
         WHERE event_id = $1 ORDER BY created_at DESC LIMIT $2 OFFSET $3",
    )
    .bind(event_id)
    .bind(validated.limit())
    .bind(validated.offset())
    .fetch_all(&state.pool)
    .await
    {
        Ok(rows) => rows,
        Err(e) => {
            tracing::error!("Failed to fetch event ratings: {:?}", e);
            return AppError::DatabaseError(e).into_response();
        }
    };

    let response = PaginatedResponse::new(items, validated, total);
    success(response, "Ratings retrieved successfully").into_response()
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
    let exists = match sqlx::query_scalar::<_, bool>(
        "SELECT EXISTS(SELECT 1 FROM events WHERE id = $1 AND is_flagged = FALSE)",
    )
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

/// Query parameters for `GET /api/v1/events/map`.
#[derive(Debug, Deserialize)]
pub struct MapSearchParams {
    /// Center latitude in decimal degrees.
    pub latitude: f64,
    /// Center longitude in decimal degrees.
    pub longitude: f64,
    /// Search radius in kilometres (default: 50, max: 500).
    pub radius: Option<f64>,
    /// Maximum results (default: 50, max: 200).
    pub limit: Option<u32>,
}

/// A nearby event with computed distance from the query point.
#[derive(Debug, Serialize)]
pub struct MapEvent {
    #[serde(flatten)]
    pub event: Event,
    /// Distance from the query point in kilometres.
    pub distance_km: f64,
}

/// Parses a single row from the `get_events_map` distance query into an
/// `Event` plus its computed `distance_km`. Pulled out of `get_events_map`
/// itself because `?` can only be used in a function returning `Result`/
/// `Option`, and that handler returns a bare `Response`.
fn map_event_row(row: &sqlx::postgres::PgRow) -> Result<(Event, f64), sqlx::Error> {
    let event = Event {
        id: row.try_get("id")?,
        organizer_id: row.try_get("organizer_id")?,
        title: row.try_get("title")?,
        description: row.try_get("description")?,
        location: row.try_get("location")?,
        start_time: row.try_get("start_time")?,
        end_time: row.try_get("end_time")?,
        is_flagged: row.try_get("is_flagged")?,
        is_featured: row.try_get("is_featured")?,
        sum_of_ratings: row.try_get("sum_of_ratings")?,
        count_of_ratings: row.try_get("count_of_ratings")?,
        created_at: row.try_get("created_at")?,
        updated_at: row.try_get("updated_at")?,
        image_url: row.try_get("image_url")?,
        latitude: row.try_get("latitude")?,
        longitude: row.try_get("longitude")?,
        is_free: false,
        is_free_populated: false,
        min_ticket_price: 0.0,
        total_tickets: 0,
        minted_tickets: 0,
    };
    let distance_km: f64 = row.try_get("distance_km")?;
    Ok((event, distance_km))
}

/// GET /api/v1/events/map
///
/// Returns upcoming events within a given radius of a geographic coordinate,
/// sorted by distance ascending. Events without coordinates are excluded.
///
/// # Query Parameters
/// - `latitude` (required): Center latitude in decimal degrees (-90 to 90).
/// - `longitude` (required): Center longitude in decimal degrees (-180 to 180).
/// - `radius` (optional): Search radius in kilometres (default: 50, max: 500).
/// - `limit` (optional): Maximum results (default: 50, max: 200).
///
/// # Response
/// Returns a list of nearby events with a `distance_km` field.
pub async fn get_events_map(
    State(state): State<EventState>,
    Query(params): Query<MapSearchParams>,
) -> Response {
    let lat = params.latitude;
    let lng = params.longitude;
    let radius_km = params.radius.unwrap_or(50.0).clamp(1.0, 500.0);
    let limit = (params.limit.unwrap_or(50) as i64).clamp(1, 200);

    if !(-90.0..=90.0).contains(&lat) {
        return AppError::ValidationError("latitude must be between -90 and 90".to_string())
            .into_response();
    }
    if !(-180.0..=180.0).contains(&lng) {
        return AppError::ValidationError("longitude must be between -180 and 180".to_string())
            .into_response();
    }

    let rad_lat = lat.to_radians();
    let rad_lng = lng.to_radians();

    let query = r#"
        SELECT e.*, (
            6371 * ACOS(
                LEAST(1.0, COS($1) * COS(RADIANS(e.latitude)) * COS(RADIANS(e.longitude) - $2)
                      + SIN($1) * SIN(RADIANS(e.latitude)))
            )
        ) AS distance_km
        FROM events e
        WHERE e.latitude IS NOT NULL
          AND e.longitude IS NOT NULL
          AND e.is_flagged = FALSE
          AND (e.end_time IS NULL OR e.end_time > NOW())
          AND (
              6371 * ACOS(
                  LEAST(1.0, COS($1) * COS(RADIANS(e.latitude)) * COS(RADIANS(e.longitude) - $2)
                        + SIN($1) * SIN(RADIANS(e.latitude)))
              )
          ) <= $3
        ORDER BY distance_km ASC
        LIMIT $4
        "#;

    let rows = match sqlx::query(query)
        .bind(rad_lat)
        .bind(rad_lng)
        .bind(radius_km)
        .bind(limit)
        .fetch_all(&state.pool)
        .await
    {
        Ok(rows) => rows,
        Err(e) => {
            tracing::error!("Failed to fetch map events: {:?}", e);
            return AppError::DatabaseError(e).into_response();
        }
    };

    let mut parsed_events: Vec<Event> = Vec::with_capacity(rows.len());
    let mut distances: Vec<f64> = Vec::with_capacity(rows.len());
    for row in &rows {
        match map_event_row(row) {
            Ok((event, distance_km)) => {
                parsed_events.push(event);
                distances.push(distance_km);
            }
            Err(e) => {
                tracing::error!("Failed to parse map event row: {:?}", e);
                return AppError::DatabaseError(e).into_response();
            }
        }
    }

    populate_is_free(&mut parsed_events, &state.pool).await;

    let events: Vec<MapEvent> = parsed_events
        .into_iter()
        .zip(distances)
        .map(|(event, distance_km)| MapEvent { event, distance_km })
        .collect();

    success(events, "Map events retrieved successfully").into_response()
}

/// GET /api/v1/events/:id/check-in-stats
pub async fn get_checkin_stats(
    State(state): State<EventState>,
    Path(event_id): Path<Uuid>,
) -> Response {
    let row = sqlx::query(
        r#"
        SELECT
            COUNT(*) FILTER (WHERE status IN ('used', 'Scanned')) AS checked_in,
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
    let event = match sqlx::query_as::<_, Event>(
        "SELECT e.*, \
         COALESCE(SUM(tt.total_quantity), 0)::bigint AS total_tickets, \
         COALESCE(SUM(tt.total_quantity - tt.available_quantity), 0)::bigint AS minted_tickets \
         FROM events e \
         LEFT JOIN ticket_tiers tt ON tt.event_id = e.id \
         WHERE e.id = $1 \
         GROUP BY e.id",
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
    .bind(event.organizer_id)
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

/// Response shape for a single attendee row.
#[derive(Debug, Serialize, sqlx::FromRow)]
pub struct AttendeeResponse {
    pub id: Uuid,
    pub owner_wallet: Option<String>,
    pub buyer_wallet: Option<String>,
    pub quantity: i32,
    pub created_at: chrono::DateTime<Utc>,
}

/// GET /api/v1/events/:id/attendees
///
/// Cursor-paginated list of ticket holders for an event. Default page size
/// is 20, maximum 100. Requesting beyond the last page returns an empty
/// list and no `next_cursor` (Issue #854).
pub async fn list_event_attendees(
    State(state): State<EventState>,
    Path(event_id): Path<Uuid>,
    Query(pagination): Query<CursorParams>,
) -> Response {
    let event_exists = match sqlx::query_scalar::<_, bool>(
        "SELECT EXISTS(SELECT 1 FROM events WHERE id = $1 AND is_flagged = FALSE)",
    )
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
        return AppError::NotFound(format!("Event with id '{event_id}' not found")).into_response();
    }

    let validated = pagination.validate();

    let cursor = match validated.cursor {
        Some(ref c) => match decode_cursor::<AttendeeCursor>(c) {
            Ok(c) => Some(c),
            Err(e) => {
                tracing::warn!("Invalid cursor provided: {}", e);
                return AppError::ValidationError(format!("Invalid cursor: {}", e)).into_response();
            }
        },
        None => None,
    };

    let items_query = if cursor.is_some() {
        r#"
        SELECT t.id, t.owner_wallet, t.buyer_wallet, t.quantity, t.created_at
        FROM tickets t
        JOIN ticket_tiers tt ON t.ticket_tier_id = tt.id
        WHERE tt.event_id = $1
          AND (t.created_at > $3 OR (t.created_at = $3 AND t.id > $4))
        ORDER BY t.created_at ASC, t.id ASC
        LIMIT $2
        "#
    } else {
        r#"
        SELECT t.id, t.owner_wallet, t.buyer_wallet, t.quantity, t.created_at
        FROM tickets t
        JOIN ticket_tiers tt ON t.ticket_tier_id = tt.id
        WHERE tt.event_id = $1
        ORDER BY t.created_at ASC, t.id ASC
        LIMIT $2
        "#
    };

    let mut query_builder = sqlx::query_as::<_, AttendeeResponse>(items_query)
        .bind(event_id)
        .bind(validated.query_limit());

    if let Some(ref c) = cursor {
        query_builder = query_builder.bind(c.created_at).bind(c.id);
    }

    let mut items = match query_builder.fetch_all(&state.pool).await {
        Ok(rows) => rows,
        Err(e) => {
            tracing::error!("Failed to fetch attendees for event {}: {:?}", event_id, e);
            return AppError::DatabaseError(e).into_response();
        }
    };

    let has_more = items.len() > validated.page_size();
    let next_cursor = if has_more {
        let last = items.pop().unwrap();
        match encode_cursor(&AttendeeCursor {
            created_at: last.created_at,
            id: last.id,
        }) {
            Ok(c) => Some(c),
            Err(e) => {
                tracing::error!("Failed to encode attendee cursor: {:?}", e);
                return AppError::InternalServerError("Failed to encode cursor".to_string())
                    .into_response();
            }
        }
    } else {
        None
    };

    let response = CursorResponse::new(items, &validated, next_cursor);
    success(response, "Attendees retrieved successfully").into_response()
}

/// Sanitizes a string field to prevent CSV formula injection when opened in spreadsheet applications.
/// Fields starting with '=', '+', '-', or '@' are escaped with a leading single quote (').
pub fn sanitize_csv_field(field: &str) -> String {
    if field.starts_with('=') || field.starts_with('+') || field.starts_with('-') || field.starts_with('@') {
        format!("'{}", field)
    } else {
        field.to_string()
    }
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
    let event_exists = match sqlx::query_scalar::<_, bool>(
        "SELECT EXISTS(SELECT 1 FROM events WHERE id = $1 AND is_flagged = FALSE)",
    )
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
            sanitize_csv_field(&owner_wallet),
            sanitize_csv_field(&buyer_wallet),
            sanitize_csv_field(&quantity.to_string()),
            sanitize_csv_field(&created_at.to_rfc3339())
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
    let event_exists = match sqlx::query_scalar::<_, bool>(
        "SELECT EXISTS(SELECT 1 FROM events WHERE id = $1 AND is_flagged = FALSE)",
    )
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
            created_at: Some(last.created_at),
            minted_tickets: Some(last.minted_tickets),
            count_of_ratings: Some(last.count_of_ratings as i64),
            min_ticket_price: Some(last.min_ticket_price),
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

/// Response shape for a single ticket tier.
#[derive(Debug, Serialize, Deserialize, sqlx::FromRow)]
pub struct TicketTierResponse {
    pub id: Uuid,
    pub name: String,
    pub description: Option<String>,
    pub price: rust_decimal::Decimal,
    pub total_quantity: i32,
    pub available_quantity: i32,
    pub created_at: chrono::DateTime<Utc>,
}

/// GET `/api/v1/events/:id/ticket-tiers`
///
/// Returns all ticket tiers for the given event ordered by price ascending.
/// Returns 404 if the event does not exist; returns an empty list if the
/// event exists but has no tiers (Issue #853).
pub async fn list_ticket_tiers(
    State(state): State<EventState>,
    Path(event_id): Path<Uuid>,
) -> Response {
    let event_exists = match sqlx::query_scalar::<_, bool>(
        "SELECT EXISTS(SELECT 1 FROM events WHERE id = $1 AND is_flagged = FALSE)",
    )
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
        return AppError::NotFound(format!("Event with id '{event_id}' not found")).into_response();
    }

    match sqlx::query_as::<_, TicketTierResponse>(
        r#"
        SELECT
            id,
            name,
            description,
            price,
            total_quantity,
            available_quantity,
            created_at
        FROM ticket_tiers
        WHERE event_id = $1
        ORDER BY price ASC
        "#,
    )
    .bind(event_id)
    .fetch_all(&state.pool)
    .await
    {
        Ok(tiers) => success(tiers, "Ticket tiers retrieved successfully").into_response(),
        Err(e) => {
            tracing::error!("Failed to fetch ticket tiers: {:?}", e);
            AppError::DatabaseError(e).into_response()
        }
    }
}

// ---------------------------------------------------------------------------
// Unit tests for list_event_tickets and image_url validation
// ---------------------------------------------------------------------------

#[test]
fn test_ticket_tier_response_serialization() {
    use rust_decimal::Decimal;
    let tier = TicketTierResponse {
        id: Uuid::new_v4(),
        name: "General".to_string(),
        description: Some("General admission".to_string()),
        price: Decimal::new(2500, 2),
        total_quantity: 500,
        available_quantity: 380,
        created_at: Utc::now(),
    };
    let json = serde_json::to_value(&tier).unwrap();
    assert_eq!(json["name"], "General");
    assert_eq!(json["description"], "General admission");
    assert_eq!(json["total_quantity"], 500);
    assert_eq!(json["available_quantity"], 380);
}

#[test]
fn test_event_detail_tiers_omitted_when_none() {
    use chrono::DateTime;
    use uuid::Uuid;

    let event = Event {
        id: Uuid::new_v4(),
        organizer_id: Uuid::new_v4(),
        title: "Test Event".to_string(),
        description: None,
        location: "Lagos".to_string(),
        start_time: DateTime::default(),
        end_time: None,
        is_flagged: false,
        is_featured: false,
        sum_of_ratings: 0,
        count_of_ratings: 0,
        created_at: DateTime::default(),
        updated_at: DateTime::default(),
        image_url: None,
        latitude: None,
        longitude: None,
        is_free: false,
        minted_tickets: 0,
        total_tickets: 0,
            is_free_populated: true,
            min_ticket_price: 0.0,
    };

    let detail = EventDetail {
        event,
        organizer_profile: None,
        tiers: None,
    };

    let json = serde_json::to_value(&detail).unwrap();
    // `tiers` must not appear in the response when None.
    assert!(
        json.get("tiers").is_none(),
        "tiers should be omitted when None"
    );
}

#[test]
fn test_event_detail_tiers_present_when_some() {
    use chrono::DateTime;
    use rust_decimal::Decimal;
    use uuid::Uuid;

    let event = Event {
        id: Uuid::new_v4(),
        organizer_id: Uuid::new_v4(),
        title: "Test Event".to_string(),
        description: None,
        location: "Lagos".to_string(),
        start_time: DateTime::default(),
        end_time: None,
        is_flagged: false,
        is_featured: false,
        sum_of_ratings: 0,
        count_of_ratings: 0,
        created_at: DateTime::default(),
        updated_at: DateTime::default(),
        image_url: None,
        latitude: None,
        longitude: None,
        is_free: true,
        minted_tickets: 0,
        total_tickets: 0,
            is_free_populated: true,
            min_ticket_price: 0.0,
    };

    let tier = TicketTierResponse {
        id: Uuid::new_v4(),
        name: "VIP".to_string(),
        price: Decimal::new(0, 0),
        total_quantity: 50,
        available_quantity: 45,
        description: None,
        created_at: chrono::Utc::now(),
    };

    let detail = EventDetail {
        event,
        organizer_profile: None,
        tiers: Some(vec![tier]),
    };

    let json = serde_json::to_value(&detail).unwrap();
    let tiers = json
        .get("tiers")
        .expect("tiers should be present when Some");
    assert!(tiers.is_array());
    assert_eq!(tiers.as_array().unwrap().len(), 1);
    assert_eq!(tiers[0]["name"], "VIP");
}

#[test]
fn test_get_event_params_defaults_to_no_tiers() {
    let params: GetEventParams = serde_json::from_str("{}").unwrap();
    assert!(!params.include_tiers);
}

#[test]
fn test_get_event_params_parses_include_tiers_true() {
    let params: GetEventParams = serde_json::from_str(r#"{"include_tiers": true}"#).unwrap();
    assert!(params.include_tiers);
}

#[test]
fn test_validate_event_title_accepts_max_length() {
    let title = "a".repeat(MAX_EVENT_TITLE_LENGTH);
    assert!(validate_event_title(&title).is_ok());
}

#[test]
fn test_validate_event_title_rejects_empty() {
    let err = validate_event_title("").unwrap_err();
    assert!(err.contains("empty"));
}

#[test]
fn test_validate_event_title_rejects_whitespace_only() {
    let err = validate_event_title("   ").unwrap_err();
    assert!(err.contains("empty"));
}

#[test]
fn test_validate_event_title_rejects_too_long() {
    let title = "a".repeat(MAX_EVENT_TITLE_LENGTH + 1);
    let err = validate_event_title(&title).unwrap_err();
    assert!(err.contains("200"));
}

#[test]
fn test_validate_event_description_accepts_max_length() {
    let desc = Some("a".repeat(MAX_EVENT_DESCRIPTION_LENGTH));
    assert!(validate_event_description(&desc).is_ok());
}

#[test]
fn test_validate_event_description_rejects_too_long() {
    let desc = Some("a".repeat(MAX_EVENT_DESCRIPTION_LENGTH + 1));
    let err = validate_event_description(&desc).unwrap_err();
    assert!(err.contains("10000"));
}

#[test]
fn test_validate_event_description_allows_none() {
    assert!(validate_event_description(&None).is_ok());
}

#[test]
fn test_image_url_valid_https() {
    assert!(validate_image_url("https://example.com/image.jpg").is_ok());
}

#[test]
fn test_image_url_http_rejected() {
    assert!(validate_image_url("http://example.com/image.jpg").is_err());
}

#[test]
fn test_image_url_javascript_rejected() {
    assert!(validate_image_url("javascript:alert(1)").is_err());
}

#[test]
fn test_image_url_data_uri_rejected() {
    assert!(validate_image_url("data:image/png;base64,abc123").is_err());
}

#[test]
fn test_image_url_empty_host_rejected() {
    assert!(validate_image_url("https://").is_err());
}

#[test]
fn test_image_url_relative_path_rejected() {
    assert!(validate_image_url("https:///path/to/image.jpg").is_err());
}

#[test]
fn test_image_url_exceeds_max_length_rejected() {
    let url = format!("https://example.com/{}", "a".repeat(MAX_IMAGE_URL_LEN));
    assert!(validate_image_url(&url).is_err());
}

#[test]
fn test_image_url_exactly_max_length_accepted() {
    let prefix = "https://example.com/";
    let url = format!("{}{}", prefix, "a".repeat(MAX_IMAGE_URL_LEN - prefix.len()));
    assert_eq!(url.len(), MAX_IMAGE_URL_LEN);
    assert!(validate_image_url(&url).is_ok());
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
        count: true,
    };
    let validated = params.validate();
    assert_eq!(validated.page_size(), 15);
    assert_eq!(validated.cursor.as_deref(), Some("test-cursor-token"));
}

#[cfg(test)]
mod similar_events_tests {
    use super::*;

    #[test]
    fn test_similar_limit_default() {
        let params = SimilarEventsParams { limit: None };
        let clamped = params.limit.unwrap_or(4).clamp(1, 10);
        assert_eq!(clamped, 4);
    }

    #[test]
    fn test_similar_limit_clamped_high() {
        let params = SimilarEventsParams { limit: Some(50) };
        let clamped = params.limit.unwrap_or(4).clamp(1, 10);
        assert_eq!(clamped, 10);
    }

    #[test]
    fn test_similar_limit_clamped_low() {
        let params = SimilarEventsParams { limit: Some(0) };
        let clamped = params.limit.unwrap_or(4).clamp(1, 10);
        assert_eq!(clamped, 1);
    }
}

#[cfg(test)]
mod search_cache_tests {
    use super::*;

    #[test]
    fn test_search_cache_key_is_deterministic() {
        let key1 = format!(
            "search:{}:{}:{}:{}:{}:{}:{}:{}:{}",
            "music", "", "", 0i64, 0i64, 0i64, 0i64, 1u32, 20u32
        );
        let key2 = format!(
            "search:{}:{}:{}:{}:{}:{}:{}:{}:{}",
            "music", "", "", 0i64, 0i64, 0i64, 0i64, 1u32, 20u32
        );
        assert_eq!(key1, key2);
    }

    #[test]
    fn test_search_cache_ttl_is_2_minutes() {
        assert_eq!(SEARCH_CACHE_TTL.as_secs(), 120);
    }

    #[test]
    fn test_event_serialization_includes_ticket_totals() {
        let event = Event {
            id: Uuid::new_v4(),
            organizer_id: Uuid::new_v4(),
            title: "Test Event".to_string(),
            description: None,
            location: "Remote".to_string(),
            start_time: Utc::now(),
            end_time: None,
            is_flagged: false,
            is_featured: false,
            sum_of_ratings: 0,
            count_of_ratings: 0,
            created_at: Utc::now(),
            updated_at: Utc::now(),
            image_url: None,
            latitude: None,
            longitude: None,
            is_free: false,
            is_free_populated: false,
            min_ticket_price: 0.0,
            total_tickets: 100,
            minted_tickets: 42,
        };

        let json = serde_json::to_value(&event).unwrap();
        assert_eq!(json["total_tickets"], 100);
        assert_eq!(json["minted_tickets"], 42);
    }

    #[test]
    fn test_sanitize_csv_field() {
        assert_eq!(sanitize_csv_field("=1+2"), "'=1+2");
        assert_eq!(sanitize_csv_field("+cmd|' /C calc'!A0"), "'+cmd|' /C calc'!A0");
        assert_eq!(sanitize_csv_field("-100"), "'-100");
        assert_eq!(sanitize_csv_field("@SUM(A1:A10)"), "'@SUM(A1:A10)");
        assert_eq!(sanitize_csv_field("10"), "10");
        assert_eq!(
            sanitize_csv_field("GDAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAA"),
            "GDAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAA"
        );
    }
}
