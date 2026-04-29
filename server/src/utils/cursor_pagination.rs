//! # Cursor-Based Pagination Utilities
//!
//! This module provides cursor-based pagination support for list endpoints.
//! Unlike OFFSET-based pagination, cursor pagination is stable under inserts/deletes
//! and scales efficiently for large datasets by using indexed key comparisons.
//!
//! The cursor is a base64-encoded JSON object containing the sort key values
//! of the last item on the previous page.

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
}

fn default_page_size() -> u32 {
    DEFAULT_PAGE_SIZE
}

impl CursorParams {
    /// Validate and normalize pagination parameters
    pub fn validate(self) -> ValidatedCursorParams {
        let limit = self.limit.min(MAX_PAGE_SIZE).max(1);
        ValidatedCursorParams {
            limit,
            cursor: self.cursor,
        }
    }
}

/// Validated cursor pagination parameters
#[derive(Debug, Clone)]
pub struct ValidatedCursorParams {
    pub limit: u32,
    pub cursor: Option<String>,
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
#[derive(Debug, Serialize)]
pub struct CursorMeta {
    /// Number of items in the current page
    pub page_size: u32,

    /// Whether there are more items after this page
    pub has_more: bool,

    /// Cursor to fetch the next page, if any
    pub next_cursor: Option<String>,
}

/// Standard cursor-paginated response wrapper
#[derive(Debug, Serialize)]
pub struct CursorResponse<T> {
    /// The data items for this page
    pub items: Vec<T>,

    /// Pagination metadata
    pub pagination: CursorMeta,
}

impl<T> CursorResponse<T> {
    /// Create a new cursor-paginated response.
    ///
    /// `items` may contain up to `limit + 1` rows; if it contains the extra row,
    /// that row is removed and used to generate `next_cursor`.
    pub fn new(mut items: Vec<T>, params: &ValidatedCursorParams, next_cursor: Option<String>) -> Self {
        let has_more = next_cursor.is_some();
        let returned_count = items.len() as u32;

        Self {
            items,
            pagination: CursorMeta {
                page_size: returned_count,
                has_more,
                next_cursor,
            },
        }
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

/// Cursor structure for event listings ordered by (start_time ASC, id ASC).
#[derive(Debug, Serialize, Deserialize)]
pub struct EventCursor {
    pub start_time: chrono::DateTime<chrono::Utc>,
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
        };
        let validated = params.validate();
        assert_eq!(validated.limit, MAX_PAGE_SIZE);
    }

    #[test]
    fn test_query_limit() {
        let params = ValidatedCursorParams {
            limit: 20,
            cursor: None,
        };
        assert_eq!(params.query_limit(), 21);
    }

    #[test]
    fn test_encode_decode_event_cursor() {
        let cursor = EventCursor {
            start_time: Utc::now(),
            id: Uuid::new_v4(),
        };

        let encoded = encode_cursor(&cursor).unwrap();
        let decoded: EventCursor = decode_cursor(&encoded).unwrap();

        assert_eq!(cursor.start_time, decoded.start_time);
        assert_eq!(cursor.id, decoded.id);
    }

    #[test]
    fn test_decode_invalid_base64() {
        let result: Result<EventCursor, _> = decode_cursor("!!!");
        assert!(result.is_err());
    }

    #[test]
    fn test_cursor_response_has_more() {
        let params = ValidatedCursorParams {
            limit: 2,
            cursor: None,
        };
        let response: CursorResponse<i32> = CursorResponse::new(vec![1, 2], &params, Some("abc".to_string()));
        assert!(response.pagination.has_more);
        assert_eq!(response.pagination.next_cursor, Some("abc".to_string()));
    }

    #[test]
    fn test_cursor_response_no_more() {
        let params = ValidatedCursorParams {
            limit: 2,
            cursor: None,
        };
        let response: CursorResponse<i32> = CursorResponse::new(vec![1, 2], &params, None);
        assert!(!response.pagination.has_more);
        assert!(response.pagination.next_cursor.is_none());
    }
}

