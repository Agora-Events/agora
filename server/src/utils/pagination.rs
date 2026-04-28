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
}

fn default_page() -> u32 {
    1
}

fn default_page_size() -> u32 {
    DEFAULT_PAGE_SIZE
}

impl PaginationParams {
    /// Validate and normalize pagination parameters
    pub fn validate(self) -> ValidatedPagination {
        let page = if self.page == 0 { 1 } else { self.page };
        let page_size = self.page_size.min(MAX_PAGE_SIZE).max(1);
        
        ValidatedPagination { page, page_size }
    }
}

/// Validated pagination parameters
#[derive(Debug, Clone, Copy)]
pub struct ValidatedPagination {
    pub page: u32,
    pub page_size: u32,
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
}

/// Pagination metadata included in responses
#[derive(Debug, Serialize)]
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

/// Standard paginated response wrapper
#[derive(Debug, Serialize)]
pub struct PaginatedResponse<T> {
    /// The data items for this page
    pub items: Vec<T>,
    
    /// Pagination metadata
    pub pagination: PaginationMeta,
}

impl<T> PaginatedResponse<T> {
    /// Create a new paginated response
    pub fn new(items: Vec<T>, pagination: ValidatedPagination, total: i64) -> Self {
        Self {
            items,
            pagination: pagination.metadata(total),
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
        };
        let validated = params.validate();
        
        assert_eq!(validated.page_size, MAX_PAGE_SIZE);
    }

    #[test]
    fn test_offset_calculation() {
        let validated = ValidatedPagination {
            page: 1,
            page_size: 20,
        };
        assert_eq!(validated.offset(), 0);
        
        let validated = ValidatedPagination {
            page: 2,
            page_size: 20,
        };
        assert_eq!(validated.offset(), 20);
        
        let validated = ValidatedPagination {
            page: 5,
            page_size: 10,
        };
        assert_eq!(validated.offset(), 40);
    }

    #[test]
    fn test_pagination_metadata() {
        let validated = ValidatedPagination {
            page: 2,
            page_size: 10,
        };
        let meta = validated.metadata(45);
        
        assert_eq!(meta.page, 2);
        assert_eq!(meta.page_size, 10);
        assert_eq!(meta.total_items, 45);
        assert_eq!(meta.total_pages, 5);
        assert_eq!(meta.has_next, true);
        assert_eq!(meta.has_previous, true);
    }

    #[test]
    fn test_pagination_metadata_first_page() {
        let validated = ValidatedPagination {
            page: 1,
            page_size: 10,
        };
        let meta = validated.metadata(45);
        
        assert_eq!(meta.has_next, true);
        assert_eq!(meta.has_previous, false);
    }

    #[test]
    fn test_pagination_metadata_last_page() {
        let validated = ValidatedPagination {
            page: 5,
            page_size: 10,
        };
        let meta = validated.metadata(45);
        
        assert_eq!(meta.has_next, false);
        assert_eq!(meta.has_previous, true);
    }

    #[test]
    fn test_pagination_metadata_empty() {
        let validated = ValidatedPagination {
            page: 1,
            page_size: 10,
        };
        let meta = validated.metadata(0);
        
        assert_eq!(meta.total_pages, 0);
        assert_eq!(meta.has_next, false);
        assert_eq!(meta.has_previous, false);
    }
}
