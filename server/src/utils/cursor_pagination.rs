//! # Cursor-Based Pagination Utilities
//!
//! This module provides cursor-based pagination support for list endpoints.
//! Unlike OFFSET-based pagination, cursor pagination is stable under inserts/deletes
//! and scales efficiently for large datasets by using indexed key comparisons.
//!
//! The cursor is a base64-encoded JSON object containing the sort key values
//! of the last item on the previous page.

use crate::utils::pagination::ListMeta;
use base64::{engine::general_purpose::URL_SAFE_NO_PAD, Engine};
use serde::{Deserialize, Serialize};
use serde_json;

/// Default page size if not specified
pub const DEFAULT_PAGE_SIZE: u32 = 20;

/// Maximum allowed page size to prevent abuse
pub const MAX_PAGE_SIZE: u32 = 100;

/// Query parameters for cursor-based pagination
#[derive(Debug, Deserialize)]
pub struct CursorParams {
    /// Number of items per page
    #[serde(default = "default_page_size")]
    pub limit: u32,

    /// Opaque cursor string for fetching the next page
    pub cursor: Option<String>,

    /// When `false`, skip the COUNT(*) query and omit `meta.total`.
    #[serde(default = "default_count")]
    pub count: bool,
}

fn default_page_size() -> u32 {
    DEFAULT_PAGE_SIZE
}

fn default_count() -> bool {
    true
}

impl CursorParams {
    /// Validate and normalize pagination parameters
    pub fn validate(self) -> ValidatedCursorParams {
        let limit = self.limit.clamp(1, MAX_PAGE_SIZE);
        ValidatedCursorParams {
            limit,
            cursor: self.cursor,
            include_count: self.count,
        }
    }
}

/// Validated cursor pagination parameters
#[derive(Debug, Clone)]
pub struct ValidatedCursorParams {
    pub limit: u32,
    pub cursor: Option<String>,
    /// When `false`, callers should skip COUNT(*) and omit `meta.total`.
    pub include_count: bool,
}

impl ValidatedCursorParams {
    /// Get the SQL LIMIT value (we fetch one extra to detect has_more)
    pub fn query_limit(&self) -> i64 {
        (self.limit + 1) as i64
    }

    /// The actual page size to return to the client
    pub fn page_size(&self) -> usize {
        self.limit as usize
    }
}

/// Pagination metadata included in cursor-based responses
#[derive(Debug, Serialize, Deserialize)]
pub struct CursorMeta {
    /// Number of items in the current page
    pub page_size: u32,

    /// Whether there are more items after this page
    pub has_more: bool,

    /// Cursor to fetch the next page, if any
    pub next_cursor: Option<String>,
}

/// Standard cursor-paginated response wrapper
#[derive(Debug, Serialize, Deserialize)]
pub struct CursorResponse<T> {
    /// The data items for this page
    pub items: Vec<T>,

    /// Pagination metadata
    pub pagination: CursorMeta,

    /// Compact total-count metadata for list UIs.
    pub meta: ListMeta,
}

impl<T> CursorResponse<T> {
    /// Create a new cursor-paginated response.
    ///
    /// `items` may contain up to `limit + 1` rows; if it contains the extra row,
    /// that row is removed and used to generate `next_cursor`.
    pub fn new(
        items: Vec<T>,
        params: &ValidatedCursorParams,
        next_cursor: Option<String>,
    ) -> Self {
        let has_more = next_cursor.is_some();
        let returned_count = items.len() as u32;

        Self {
            items,
            pagination: CursorMeta {
                page_size: returned_count,
                has_more,
                next_cursor,
            },
            meta: ListMeta {
                total: None,
                page_size: params.limit,
                has_more,
            },
        }
    }

    /// Attach a COUNT(*) total. No-op when `?count=false` was supplied.
    pub fn with_total(mut self, total: i64, include_count: bool) -> Self {
        if include_count {
            self.meta.total = Some(total);
        }
        self
    }
}

/// Encode a serializable cursor value into a base64 string.
///
/// # Errors
/// Returns an error if JSON serialization fails.
pub fn encode_cursor<C: Serialize>(cursor: &C) -> Result<String, serde_json::Error> {
    let json = serde_json::to_string(cursor)?;
    Ok(URL_SAFE_NO_PAD.encode(json.as_bytes()))
}

/// Decode a base64 cursor string back into a cursor value.
///
/// # Errors
/// Returns an error if base64 decoding or JSON deserialization fails.
pub fn decode_cursor<C: for<'de> Deserialize<'de>>(cursor: &str) -> Result<C, CursorError> {
    let bytes = URL_SAFE_NO_PAD
        .decode(cursor.as_bytes())
        .map_err(CursorError::Decode)?;
    let json = String::from_utf8(bytes).map_err(|e| CursorError::InvalidUtf8(e.utf8_error()))?;
    serde_json::from_str(&json).map_err(CursorError::Deserialize)
}

/// Errors that can occur when decoding a cursor.
#[derive(Debug, thiserror::Error)]
pub enum CursorError {
    #[error("failed to decode base64 cursor: {0}")]
    Decode(#[from] base64::DecodeError),

    #[error("cursor contains invalid utf-8: {0}")]
    InvalidUtf8(#[from] std::str::Utf8Error),

    #[error("failed to deserialize cursor: {0}")]
    Deserialize(#[from] serde_json::Error),
}

/// Cursor structure for event listings. The active sort key is determined by the
/// request's `sort_by` parameter; all fields are stored for stable pagination.
#[derive(Debug, Serialize, Deserialize)]
pub struct EventCursor {
    pub start_time: chrono::DateTime<chrono::Utc>,
    pub id: uuid::Uuid,
    #[serde(default)]
    pub created_at: Option<chrono::DateTime<chrono::Utc>>,
    #[serde(default)]
    pub minted_tickets: Option<i64>,
    /// Sort key for `count_of_ratings`-based ordering ("popular" sort).
    #[serde(default)]
    pub count_of_ratings: Option<i64>,
    /// Sort key for `price_asc` / `price_desc` (minimum ticket-tier price).
    #[serde(default)]
    pub min_ticket_price: Option<f64>,
}

/// Cursor structure for past event listings ordered by (end_time DESC, id DESC).
#[derive(Debug, Serialize, Deserialize)]
pub struct PastEventCursor {
    pub end_time: chrono::DateTime<chrono::Utc>,
    pub id: uuid::Uuid,
}

/// Cursor structure for event attendee listings ordered by (created_at ASC, id ASC).
#[derive(Debug, Serialize, Deserialize)]
pub struct AttendeeCursor {
    pub created_at: chrono::DateTime<chrono::Utc>,
    pub id: uuid::Uuid,
}

#[cfg(test)]
mod tests {
    use super::*;
    use chrono::Utc;
    use uuid::Uuid;

    #[test]
    fn test_default_cursor_params() {
        let params = CursorParams {
            limit: 0,
            cursor: None,
            count: true,
        };
        let validated = params.validate();
        assert_eq!(validated.limit, 1);
        assert!(validated.cursor.is_none());
    }

    #[test]
    fn test_max_page_size() {
        let params = CursorParams {
            limit: 1000,
            cursor: None,
            count: true,
        };
        let validated = params.validate();
        assert_eq!(validated.limit, MAX_PAGE_SIZE);
    }

    #[test]
    fn test_query_limit() {
        let params = ValidatedCursorParams {
            limit: 20,
            cursor: None,
            include_count: true,
        };
        assert_eq!(params.query_limit(), 21);
    }

    #[test]
    fn test_encode_decode_event_cursor_roundtrip() {
        let cursor = EventCursor {
            start_time: Utc::now(),
            id: Uuid::new_v4(),
            created_at: Some(Utc::now()),
            minted_tickets: Some(42),
            count_of_ratings: Some(10),
            min_ticket_price: Some(25.0),
        };

        let encoded = encode_cursor(&cursor).unwrap();
        let decoded: EventCursor = decode_cursor(&encoded).unwrap();

        assert_eq!(cursor.start_time, decoded.start_time);
        assert_eq!(cursor.id, decoded.id);
        assert_eq!(cursor.created_at, decoded.created_at);
        assert_eq!(cursor.minted_tickets, decoded.minted_tickets);
        assert_eq!(cursor.count_of_ratings, decoded.count_of_ratings);
    }

    #[test]
    fn test_encode_decode_past_event_cursor_roundtrip() {
        let cursor = PastEventCursor {
            end_time: Utc::now(),
            id: Uuid::new_v4(),
        };

        let encoded = encode_cursor(&cursor).unwrap();
        let decoded: PastEventCursor = decode_cursor(&encoded).unwrap();

        assert_eq!(cursor.end_time, decoded.end_time);
        assert_eq!(cursor.id, decoded.id);
    }

    #[test]
    fn test_decode_invalid_base64() {
        let result: Result<EventCursor, _> = decode_cursor("!!!not-valid-base64!!!");
        assert!(matches!(result, Err(CursorError::Decode(_))));
    }

    #[test]
    fn test_decode_truncated_base64() {
        let cursor = EventCursor {
            start_time: Utc::now(),
            id: Uuid::new_v4(),
            created_at: None,
            minted_tickets: None,
            count_of_ratings: None,
            min_ticket_price: None,
        };
        let encoded = encode_cursor(&cursor).unwrap();
        let truncated = &encoded[..encoded.len() / 2];
        let result: Result<EventCursor, _> = decode_cursor(truncated);
        assert!(result.is_err());
    }

    #[test]
    fn test_cursor_response_has_more() {
        let params = ValidatedCursorParams {
            limit: 2,
            cursor: None,
            include_count: true,
        };
        let response: CursorResponse<i32> =
            CursorResponse::new(vec![1, 2], &params, Some("abc".to_string()));
        assert!(response.pagination.has_more);
        assert_eq!(response.pagination.next_cursor, Some("abc".to_string()));
    }

    #[test]
    fn test_cursor_response_no_more() {
        let params = ValidatedCursorParams {
            limit: 2,
            cursor: None,
            include_count: true,
        };
        let response: CursorResponse<i32> = CursorResponse::new(vec![1, 2], &params, None);
        assert!(!response.pagination.has_more);
        assert!(response.pagination.next_cursor.is_none());
    }

    // -----------------------------------------------------------------------
    // Issue #1266 — additional coverage
    // -----------------------------------------------------------------------

    /// An empty result set must return `has_more = false` and no next cursor.
    #[test]
    fn test_cursor_response_empty_result() {
        let params = ValidatedCursorParams {
            limit: 20,
            cursor: None,
            include_count: true,
        };
        let response: CursorResponse<i32> = CursorResponse::new(vec![], &params, None);
        assert!(!response.pagination.has_more);
        assert!(response.pagination.next_cursor.is_none());
        assert!(response.items.is_empty());
    }

    /// A single-item result must behave correctly (no overflow, has_more = false).
    #[test]
    fn test_cursor_response_single_result() {
        let params = ValidatedCursorParams {
            limit: 20,
            cursor: None,
            include_count: true,
        };
        let response: CursorResponse<i32> = CursorResponse::new(vec![42], &params, None);
        assert!(!response.pagination.has_more);
        assert_eq!(response.items.len(), 1);
    }

    /// When the result count equals the page size exactly, `has_more` must be
    /// determined by whether a `next_cursor` was passed, not by item count.
    #[test]
    fn test_cursor_response_exact_page_boundary_no_more() {
        let params = ValidatedCursorParams {
            limit: 3,
            cursor: None,
            include_count: true,
        };
        // Exactly page_size items returned — caller did NOT pass a next cursor,
        // meaning the DB had no additional rows.
        let response: CursorResponse<i32> = CursorResponse::new(vec![1, 2, 3], &params, None);
        assert!(!response.pagination.has_more);
        assert!(response.pagination.next_cursor.is_none());
        assert_eq!(response.items.len(), 3);
    }

    #[test]
    fn test_cursor_response_exact_page_boundary_has_more() {
        let params = ValidatedCursorParams {
            limit: 3,
            cursor: None,
            include_count: true,
        };
        // Caller stripped the extra item and encoded a cursor.
        let response: CursorResponse<i32> =
            CursorResponse::new(vec![1, 2, 3], &params, Some("next-token".to_string()));
        assert!(response.pagination.has_more);
        assert_eq!(
            response.pagination.next_cursor.as_deref(),
            Some("next-token")
        );
        assert_eq!(response.meta.page_size, 3);
        assert!(response.meta.has_more);
        assert!(response.meta.total.is_none());
    }

    #[test]
    fn test_cursor_response_with_total() {
        let params = ValidatedCursorParams {
            limit: 20,
            cursor: None,
            include_count: true,
        };
        let response: CursorResponse<i32> =
            CursorResponse::new(vec![1, 2], &params, None).with_total(340, true);
        assert_eq!(response.meta.total, Some(340));
        assert_eq!(response.meta.page_size, 20);
        assert!(!response.meta.has_more);
    }

    #[test]
    fn test_cursor_response_count_false_omits_total() {
        let params = ValidatedCursorParams {
            limit: 20,
            cursor: None,
            include_count: false,
        };
        let response: CursorResponse<i32> =
            CursorResponse::new(vec![1], &params, None).with_total(340, false);
        assert!(response.meta.total.is_none());
        let json = serde_json::to_value(&response).unwrap();
        assert!(json["meta"].get("total").is_none());
    }

    /// A tampered (invalid characters) base64 cursor must return an error.
    #[test]
    fn test_decode_tampered_cursor_is_rejected() {
        // Valid base64 chars but content that cannot deserialise into EventCursor.
        let tampered = URL_SAFE_NO_PAD.encode(b"this is not valid json");
        let result: Result<EventCursor, _> = decode_cursor(&tampered);
        assert!(
            matches!(result, Err(CursorError::Deserialize(_))),
            "tampered cursor should fail with Deserialize error"
        );
    }

    /// Round-trip encode/decode of an AttendeeCursor.
    #[test]
    fn test_encode_decode_attendee_cursor_roundtrip() {
        let cursor = AttendeeCursor {
            created_at: Utc::now(),
            id: Uuid::new_v4(),
        };
        let encoded = encode_cursor(&cursor).unwrap();
        let decoded: AttendeeCursor = decode_cursor(&encoded).unwrap();
        assert_eq!(cursor.created_at, decoded.created_at);
        assert_eq!(cursor.id, decoded.id);
    }

    /// Cursor with all optional fields set to None still round-trips correctly.
    #[test]
    fn test_encode_decode_event_cursor_minimal() {
        let cursor = EventCursor {
            start_time: Utc::now(),
            id: Uuid::new_v4(),
            created_at: None,
            minted_tickets: None,
            count_of_ratings: None,
            min_ticket_price: None,
        };
        let encoded = encode_cursor(&cursor).unwrap();
        let decoded: EventCursor = decode_cursor(&encoded).unwrap();
        assert_eq!(cursor.start_time, decoded.start_time);
        assert_eq!(cursor.id, decoded.id);
        assert!(decoded.created_at.is_none());
        assert!(decoded.minted_tickets.is_none());
        assert!(decoded.count_of_ratings.is_none());
    }
}
