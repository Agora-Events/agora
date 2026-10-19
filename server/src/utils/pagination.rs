//! # Pagination Utilities
//!
//! This module provides standardized pagination support for list endpoints.
//! All paginated responses follow a consistent structure with metadata about
//! the current page, total items, and navigation links.

use serde::{Deserialize, Serialize};

/// Default page size if not specified
pub const DEFAULT_PAGE_SIZE: u32 = 20;

/// Maximum allowed page size to prevent abuse
pub const MAX_PAGE_SIZE: u32 = 100;

/// Query parameters for pagination
#[derive(Debug, Deserialize)]
pub struct PaginationParams {
    /// Page number (1-indexed)
    #[serde(default = "default_page")]
    pub page: u32,

    /// Number of items per page
    #[serde(default = "default_page_size")]
    pub page_size: u32,

    /// When `false`, skip the COUNT(*) query and omit `meta.total`.
    #[serde(default = "default_count")]
    pub count: bool,
}

fn default_page() -> u32 {
    1
}

fn default_page_size() -> u32 {
    DEFAULT_PAGE_SIZE
}

fn default_count() -> bool {
    true
}

impl PaginationParams {
    /// Validate and normalize pagination parameters
    pub fn validate(self) -> ValidatedPagination {
        let page = if self.page == 0 { 1 } else { self.page };
        let page_size = self.page_size.clamp(1, MAX_PAGE_SIZE);

        ValidatedPagination {
            page,
            page_size,
            include_count: self.count,
        }
    }
}

/// Validated pagination parameters
#[derive(Debug, Clone, Copy)]
pub struct ValidatedPagination {
    pub page: u32,
    pub page_size: u32,
    /// When `false`, callers should skip COUNT(*) and omit `meta.total`.
    pub include_count: bool,
}

impl ValidatedPagination {
    /// Calculate the SQL OFFSET value
    pub fn offset(&self) -> i64 {
        ((self.page - 1) * self.page_size) as i64
    }

    /// Get the SQL LIMIT value
    pub fn limit(&self) -> i64 {
        self.page_size as i64
    }

    /// Create pagination metadata from total count
    pub fn metadata(&self, total: i64) -> PaginationMeta {
        let total_pages = if total == 0 {
            0
        } else {
            ((total as f64) / (self.page_size as f64)).ceil() as u32
        };

        PaginationMeta {
            page: self.page,
            page_size: self.page_size,
            total_items: total,
            total_pages,
            has_next: self.page < total_pages,
            has_previous: self.page > 1,
        }
    }

    /// Compact list metadata (`total`, `page_size`, `has_more`).
    ///
    /// `total` is omitted when `include_count` is false.
    pub fn list_meta(&self, total: Option<i64>, item_count: usize) -> ListMeta {
        let has_more = match total {
            Some(t) => {
                let total_pages = if t == 0 {
                    0
                } else {
                    ((t as f64) / (self.page_size as f64)).ceil() as u32
                };
                self.page < total_pages
            }
            None => item_count as u32 >= self.page_size,
        };
        ListMeta {
            total: total.filter(|_| self.include_count),
            page_size: self.page_size,
            has_more,
        }
    }
}

/// Pagination metadata included in responses
#[derive(Debug, Serialize, Deserialize)]
pub struct PaginationMeta {
    /// Current page number (1-indexed)
    pub page: u32,

    /// Number of items per page
    pub page_size: u32,

    /// Total number of items across all pages
    pub total_items: i64,

    /// Total number of pages
    pub total_pages: u32,

    /// Whether there is a next page
    pub has_next: bool,

    /// Whether there is a previous page
    pub has_previous: bool,
}

/// Compact metadata included on paginated list responses.
///
/// Used by clients to render copy such as "Showing 12 of 340" and to size a pager.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ListMeta {
    /// Total matching rows. Omitted when the caller passed `?count=false`.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub total: Option<i64>,

    /// Requested page size.
    pub page_size: u32,

    /// Whether another page of results exists.
    pub has_more: bool,
}

/// Standard paginated response wrapper
#[derive(Debug, Serialize, Deserialize)]
pub struct PaginatedResponse<T> {
    /// The data items for this page
    pub items: Vec<T>,

    /// Pagination metadata
    pub pagination: PaginationMeta,

    /// Compact total-count metadata for list UIs.
    pub meta: ListMeta,
}

impl<T> PaginatedResponse<T> {
    /// Create a new paginated response
    pub fn new(items: Vec<T>, pagination: ValidatedPagination, total: i64) -> Self {
        let item_count = items.len();
        Self {
            items,
            pagination: pagination.metadata(total),
            meta: pagination.list_meta(Some(total), item_count),
        }
    }

    /// Create a paginated response, omitting `meta.total` when `include_count` is false.
    pub fn new_with_optional_total(
        items: Vec<T>,
        pagination: ValidatedPagination,
        total: Option<i64>,
    ) -> Self {
        let item_count = items.len();
        let pagination_meta = pagination.metadata(total.unwrap_or(0));
        Self {
            items,
            pagination: pagination_meta,
            meta: pagination.list_meta(total, item_count),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_default_pagination() {
        let params = PaginationParams {
            page: 0,
            page_size: 0,
            count: true,
        };
        let validated = params.validate();

        assert_eq!(validated.page, 1);
        assert_eq!(validated.page_size, 1);
    }

    #[test]
    fn test_max_page_size() {
        let params = PaginationParams {
            page: 1,
            page_size: 1000,
            count: true,
        };
        let validated = params.validate();

        assert_eq!(validated.page_size, MAX_PAGE_SIZE);
    }

    #[test]
    fn test_offset_calculation() {
        let validated = ValidatedPagination {
            page: 1,
            page_size: 20,
            include_count: true,
        };
        assert_eq!(validated.offset(), 0);

        let validated = ValidatedPagination {
            page: 2,
            page_size: 20,
            include_count: true,
        };
        assert_eq!(validated.offset(), 20);

        let validated = ValidatedPagination {
            page: 5,
            page_size: 10,
            include_count: true,
        };
        assert_eq!(validated.offset(), 40);
    }

    #[test]
    fn test_pagination_metadata() {
        let validated = ValidatedPagination {
            page: 2,
            page_size: 10,
            include_count: true,
        };
        let meta = validated.metadata(45);

        assert_eq!(meta.page, 2);
        assert_eq!(meta.page_size, 10);
        assert_eq!(meta.total_items, 45);
        assert_eq!(meta.total_pages, 5);
        assert!(meta.has_next);
        assert!(meta.has_previous);
    }

    #[test]
    fn test_pagination_metadata_first_page() {
        let validated = ValidatedPagination {
            page: 1,
            page_size: 10,
            include_count: true,
        };
        let meta = validated.metadata(45);

        assert!(meta.has_next);
        assert!(!meta.has_previous);
    }

    #[test]
    fn test_pagination_metadata_last_page() {
        let validated = ValidatedPagination {
            page: 5,
            page_size: 10,
            include_count: true,
        };
        let meta = validated.metadata(45);

        assert!(!meta.has_next);
        assert!(meta.has_previous);
    }

    #[test]
    fn test_pagination_metadata_empty() {
        let validated = ValidatedPagination {
            page: 1,
            page_size: 10,
            include_count: true,
        };
        let meta = validated.metadata(0);

        assert_eq!(meta.total_pages, 0);
        assert!(!meta.has_next);
        assert!(!meta.has_previous);
    }

    #[test]
    fn test_list_meta_includes_total_by_default() {
        let validated = ValidatedPagination {
            page: 1,
            page_size: 20,
            include_count: true,
        };
        let meta = validated.list_meta(Some(340), 12);
        assert_eq!(meta.total, Some(340));
        assert_eq!(meta.page_size, 20);
        assert!(meta.has_more);
    }

    #[test]
    fn test_list_meta_omits_total_when_count_disabled() {
        let validated = ValidatedPagination {
            page: 1,
            page_size: 20,
            include_count: false,
        };
        let meta = validated.list_meta(Some(340), 12);
        assert_eq!(meta.total, None);
        assert_eq!(meta.page_size, 20);
    }

    #[test]
    fn test_paginated_response_includes_meta() {
        let validated = ValidatedPagination {
            page: 1,
            page_size: 2,
            include_count: true,
        };
        let response = PaginatedResponse::new(vec!["a", "b"], validated, 5);
        assert_eq!(response.meta.total, Some(5));
        assert_eq!(response.meta.page_size, 2);
        assert!(response.meta.has_more);
    }
}
