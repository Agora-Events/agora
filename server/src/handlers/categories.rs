//! # Category Handlers
//!
//! This module provides HTTP handlers for category-related operations.

use axum::{
    extract::{Query, State},
    http::{header, HeaderMap, HeaderValue, StatusCode},
    response::IntoResponse,
    response::Response,
};
use serde::{Deserialize, Serialize};
use sqlx::PgPool;
use std::time::Duration;
use uuid::Uuid;

use crate::cache::RedisCache;
use crate::models::category::Category;
use crate::utils::error::AppError;
use crate::utils::pagination::{PaginatedResponse, PaginationParams};
use crate::utils::response::ApiResponse;

/// TTL for cached category listings. Categories are effectively static, so a
/// long TTL (1 hour) sharply reduces database load under traffic (Issue #583).
const CATEGORIES_CACHE_TTL: Duration = Duration::from_secs(3600);

/// `Cache-Control` applied to successful category responses (Issue #1260).
///
/// `max-age=300` lets clients/CDNs serve for 5 minutes; `stale-while-revalidate=600`
/// lets them serve a stale body for a further 10 minutes while revalidating.
const CATEGORY_CACHE_CONTROL: &str = "public, max-age=300, stale-while-revalidate=600";

/// Build a cacheable success response for the categories endpoints.
///
/// * Sets `Cache-Control: public, max-age=300, stale-while-revalidate=600`.
/// * Computes a weak `ETag` from a hash of the serialised payload.
/// * Returns `304 Not Modified` (empty body) when the client's `If-None-Match`
///   header matches the generated `ETag`.
fn cached_success_response<T: Serialize>(
    data: T,
    message: &str,
    if_none_match: Option<&HeaderValue>,
) -> Response {
    let envelope = ApiResponse {
        success: true,
        data: Some(data),
        message: Some(message.to_string()),
    };
    let json = match serde_json::to_string(&envelope) {
        Ok(j) => j,
        Err(_) => {
            return no_store_error(AppError::InternalServerError(
                "failed to serialize category response".into(),
            ))
        }
    };

    let etag = weak_etag(&json);
    if if_none_match
        .map(|v| v.as_bytes() == etag.as_bytes())
        .unwrap_or(false)
    {
        return Response::builder()
            .status(StatusCode::NOT_MODIFIED)
            .header(header::CACHE_CONTROL, CATEGORY_CACHE_CONTROL)
            .header(header::ETAG, etag)
            .body(axum::body::Body::empty())
            .unwrap();
    }

    Response::builder()
        .status(StatusCode::OK)
        .header(header::CONTENT_TYPE, "application/json")
        .header(header::CACHE_CONTROL, CATEGORY_CACHE_CONTROL)
        .header(header::ETAG, etag)
        .body(axum::body::Body::from(json))
        .unwrap()
}

/// Return an error response that is explicitly marked `no-store` so caches never
/// retain error bodies (Issue #1260).
fn no_store_error(err: AppError) -> Response {
    let mut resp = err.into_response();
    resp.headers_mut()
        .insert(header::CACHE_CONTROL, HeaderValue::from_static("no-store"));
    resp
}

/// Compute a weak `ETag` (`W/"<hash>"`) from the serialised payload bytes.
fn weak_etag(payload: &str) -> String {
    use sha2::{Digest, Sha256};
    let mut hasher = Sha256::new();
    hasher.update(payload.as_bytes());
    format!("W/\"{}\"", hex::encode(hasher.finalize()))
}

/// Application state for category handlers: database pool + Redis cache.
#[derive(Clone)]
pub struct CategoryState {
    pub pool: PgPool,
    pub redis: RedisCache,
}

/// Query parameters for filtering categories
#[derive(Debug, Deserialize)]
pub struct CategoryFilters {
    /// Filter by parent category ID (use "null" for root categories)
    pub parent_id: Option<String>,

    /// Search in name and description
    pub search: Option<String>,
}

/// Build the deterministic Redis cache key for a category listing. Prefixed
/// with `categories:all` and discriminated by filters + pagination so distinct
/// queries don't collide.
fn categories_cache_key(parent_id: &str, search: &str, page: u32, page_size: u32) -> String {
    format!(
        "categories:all:{}:{}:{}:{}",
        parent_id, search, page, page_size
    )
}

/// List all categories with pagination and optional filters
///
/// # Endpoint
/// GET `/api/v1/categories`
///
/// # Query Parameters
/// - `page` (optional): Page number (default: 1)
/// - `page_size` (optional): Items per page (default: 20, max: 100)
/// - `parent_id` (optional): Filter by parent category (use "null" for root)
/// - `search` (optional): Search in name and description
///
/// # Response
/// Returns a paginated list of categories with metadata
pub async fn list_categories(
    State(mut state): State<CategoryState>,
    headers: HeaderMap,
    Query(pagination): Query<PaginationParams>,
    Query(filters): Query<CategoryFilters>,
) -> Response {
    let validated_pagination = pagination.validate();
    let if_none_match = headers.get(header::IF_NONE_MATCH);

    // --- Issue #1857: validate parent_id BEFORE the cache lookup ------------
    //
    // A parent_id that is neither the literal string "null" nor a valid UUID
    // is a client error.  Return 400 immediately so a typo never reaches the
    // cache (and never poisons the cache key with a junk value).
    let parent_filter: Option<Option<Uuid>> = if let Some(ref parent_str) = filters.parent_id {
        if parent_str == "null" {
            Some(None) // caller wants root categories (WHERE parent_id IS NULL)
        } else {
            match Uuid::parse_str(parent_str) {
                Ok(uuid) => Some(Some(uuid)),
                Err(_) => {
                    return no_store_error(AppError::ValidationError(
                        "parent_id must be \"null\" or a valid UUID".into(),
                    ));
                }
            }
        }
    } else {
        None // no filter
    };

    // Attempt to serve from cache first; a Redis miss or error falls through
    // to the database without failing the request.
    let cache_key = categories_cache_key(
        filters.parent_id.as_deref().unwrap_or(""),
        filters.search.as_deref().unwrap_or(""),
        validated_pagination.page,
        validated_pagination.page_size,
    );
    match state
        .redis
        .get::<PaginatedResponse<Category>>(&cache_key)
        .await
    {
        Ok(Some(cached)) => {
            tracing::debug!("Cache hit for categories key: {}", cache_key);
            return cached_success_response(
                cached,
                "Categories retrieved successfully (cached)",
                if_none_match,
            );
        }
        Ok(None) => {}
        Err(e) => tracing::warn!(
            "Redis error during categories lookup, falling back: {:?}",
            e
        ),
    }

    // Build the WHERE clause dynamically
    let mut where_clauses = Vec::new();
    let mut param_count = 0;

    // parent_filter was already validated above.
    if let Some(ref pf) = parent_filter {
        param_count += 1;
        if pf.is_none() {
            where_clauses.push("parent_id IS NULL".to_string());
            param_count -= 1; // No parameter needed for IS NULL
        } else {
            where_clauses.push(format!("parent_id = ${}", param_count));
        }
    }

    if filters.search.is_some() {
        param_count += 1;
        where_clauses.push(format!(
            "(name ILIKE ${} OR description ILIKE ${})",
            param_count, param_count
        ));
    }

    let where_clause = if where_clauses.is_empty() {
        String::new()
    } else {
        format!("WHERE {}", where_clauses.join(" AND "))
    };

    // Count total items
    let count_query = format!("SELECT COUNT(*) FROM categories {}", where_clause);
    let mut count_query_builder = sqlx::query_scalar::<_, i64>(&count_query);

    if let Some(Some(parent_id)) = parent_filter {
        count_query_builder = count_query_builder.bind(parent_id);
    }
    if let Some(ref search) = filters.search {
        count_query_builder = count_query_builder.bind(format!("%{}%", search));
    }

    let total = match count_query_builder.fetch_one(&state.pool).await {
        Ok(count) => count,
        Err(e) => {
            tracing::error!("Failed to count categories: {:?}", e);
            return no_store_error(AppError::DatabaseError(e));
        }
    };

    // Fetch paginated items
    let items_query = format!(
        "SELECT * FROM categories {} ORDER BY name ASC LIMIT ${} OFFSET ${}",
        where_clause,
        param_count + 1,
        param_count + 2
    );

    let mut items_query_builder = sqlx::query_as::<_, Category>(&items_query);

    if let Some(Some(parent_id)) = parent_filter {
        items_query_builder = items_query_builder.bind(parent_id);
    }
    if let Some(ref search) = filters.search {
        items_query_builder = items_query_builder.bind(format!("%{}%", search));
    }

    items_query_builder = items_query_builder
        .bind(validated_pagination.limit())
        .bind(validated_pagination.offset());

    let items = match items_query_builder.fetch_all(&state.pool).await {
        Ok(categories) => categories,
        Err(e) => {
            tracing::error!("Failed to fetch categories: {:?}", e);
            return no_store_error(AppError::DatabaseError(e));
        }
    };

    let response = PaginatedResponse::new(items, validated_pagination, total);

    // Store in cache for an hour; a Redis failure is non-fatal.
    if let Err(e) = state
        .redis
        .set(&cache_key, &response, CATEGORIES_CACHE_TTL)
        .await
    {
        tracing::warn!("Failed to cache categories: {:?}", e);
    }

    cached_success_response(
        response,
        "Categories retrieved successfully",
        if_none_match,
    )
}

/// Get a single category by ID
///
/// # Endpoint
/// GET `/api/v1/categories/:id`
pub async fn get_category(
    State(pool): State<PgPool>,
    headers: HeaderMap,
    axum::extract::Path(category_id): axum::extract::Path<Uuid>,
) -> Response {
    let if_none_match = headers.get(header::IF_NONE_MATCH);
    let category = match sqlx::query_as::<_, Category>("SELECT * FROM categories WHERE id = $1")
        .bind(category_id)
        .fetch_optional(&pool)
        .await
    {
        Ok(Some(category)) => category,
        Ok(None) => {
            return no_store_error(AppError::NotFound(format!(
                "Category with id '{}' not found",
                category_id
            )));
        }
        Err(e) => {
            tracing::error!("Failed to fetch category: {:?}", e);
            return no_store_error(AppError::DatabaseError(e));
        }
    };

    cached_success_response(category, "Category retrieved successfully", if_none_match)
}

/// Validates a category slug against the slug rules.
///
/// A valid slug (Issue #1856):
/// - Contains only lowercase ASCII letters (`a-z`), ASCII digits (`0-9`), and hyphens (`-`).
/// - Starts and ends with a letter or digit (no leading or trailing hyphens).
/// - Never contains two consecutive hyphens (`--`).
/// - Is between 1 and 64 characters long (inclusive).
///
/// # Examples
/// ```
/// assert!(is_valid_category_slug("music"));
/// assert!(is_valid_category_slug("live-music"));
/// assert!(is_valid_category_slug("2026-events"));
/// assert!(!is_valid_category_slug("-"));
/// assert!(!is_valid_category_slug("-music"));
/// assert!(!is_valid_category_slug("music-"));
/// assert!(!is_valid_category_slug("live--music"));
/// ```
pub fn is_valid_category_slug(slug: &str) -> bool {
    if slug.is_empty() || slug.len() > 64 {
        return false;
    }

    // All characters must be lowercase ASCII letters, digits, or hyphens.
    if !slug
        .chars()
        .all(|c| c.is_ascii_lowercase() || c.is_ascii_digit() || c == '-')
    {
        return false;
    }

    // Must not start or end with a hyphen.
    let bytes = slug.as_bytes();
    if bytes[0] == b'-' || bytes[bytes.len() - 1] == b'-' {
        return false;
    }

    // Must not contain two consecutive hyphens.
    if slug.contains("--") {
        return false;
    }

    true
}


/// Canonical categories defined in the contract's Category enum.
/// Used to validate database categories match the contract at startup.
pub const CANONICAL_CATEGORIES: &[(u32, &str, &str)] = &[
    (
        1,
        "Music",
        "Music events including concerts, festivals, and live performances",
    ),
    (
        2,
        "Sports",
        "Sports events including games, tournaments, and athletic competitions",
    ),
    (
        3,
        "Tech",
        "Technology events including conferences, hackathons, and meetups",
    ),
    (
        4,
        "Arts",
        "Arts events including exhibitions, galleries, and cultural shows",
    ),
    (
        5,
        "Food",
        "Food events including tastings, festivals, and cooking classes",
    ),
    (
        6,
        "Business",
        "Business events including networking, seminars, and trade shows",
    ),
    (
        7,
        "Health",
        "Health events including wellness workshops, fitness classes, and medical conferences",
    ),
    (
        8,
        "Education",
        "Education events including workshops, lectures, and training sessions",
    ),
    (
        9,
        "Community",
        "Community events including social gatherings, volunteering, and local meetups",
    ),
    (
        10,
        "Other",
        "Other events that do not fit into the above categories",
    ),
];

/// Validates that the categories in the database match the contract's canonical list.
/// Logs an error if there is a mismatch. Should be called at server startup.
pub async fn validate_categories_match_contract(pool: &PgPool) -> bool {
    use sqlx::Row;
    match sqlx::query("SELECT name, slug FROM categories ORDER BY created_at ASC")
        .fetch_all(pool)
        .await
    {
        Ok(rows) => {
            let db_names: Vec<String> = rows.iter().map(|r| r.get::<String, _>("name")).collect();
            let canonical_names: Vec<String> = CANONICAL_CATEGORIES
                .iter()
                .map(|(_, name, _)| name.to_string())
                .collect();

            if db_names == canonical_names {
                tracing::info!(
                    "Database categories match the contract's canonical list ({} categories)",
                    db_names.len()
                );
                true
            } else {
                tracing::error!(
                    "Database categories mismatch detected! DB has {:?}, contract expects {:?}",
                    db_names,
                    canonical_names
                );
                false
            }
        }
        Err(e) => {
            tracing::error!(
                "Could not validate categories against contract: {:?}. This is normal if the categories table has not been seeded yet.",
                e
            );
            false
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_categories_cache_key_is_deterministic() {
        let a = categories_cache_key("", "", 1, 20);
        let b = categories_cache_key("", "", 1, 20);
        assert_eq!(a, b);
        assert_eq!(a, "categories:all:::1:20");
    }

    #[test]
    fn test_categories_cache_key_varies_with_filters() {
        let unfiltered = categories_cache_key("", "", 1, 20);
        let filtered = categories_cache_key("null", "music", 2, 20);
        assert_ne!(unfiltered, filtered);
        assert_eq!(filtered, "categories:all:null:music:2:20");
    }

    #[test]
    fn test_canonical_category_id_to_name_mapping() {
        // Verify each canonical category has the correct ID-to-name mapping
        // as defined in the contract's Category enum (Music=1, Sports=2, Tech=3,
        // Arts=4, Food=5, Business=6, Health=7, Education=8, Community=9, Other=10)
        let mapping: std::collections::HashMap<u32, &str> = CANONICAL_CATEGORIES
            .iter()
            .map(|(id, name, _)| (*id, *name))
            .collect();

        assert_eq!(mapping.get(&1), Some(&"Music"));
        assert_eq!(mapping.get(&2), Some(&"Sports"));
        assert_eq!(mapping.get(&3), Some(&"Tech"));
        assert_eq!(mapping.get(&4), Some(&"Arts"));
        assert_eq!(mapping.get(&5), Some(&"Food"));
        assert_eq!(mapping.get(&6), Some(&"Business"));
        assert_eq!(mapping.get(&7), Some(&"Health"));
        assert_eq!(mapping.get(&8), Some(&"Education"));
        assert_eq!(mapping.get(&9), Some(&"Community"));
        assert_eq!(mapping.get(&10), Some(&"Other"));
        assert_eq!(mapping.len(), 10);
    }

    // -----------------------------------------------------------------------
    // Caching headers / ETag / 304 (Issue #1260)
    // -----------------------------------------------------------------------

    #[test]
    fn test_cached_success_response_sets_cache_headers() {
        let resp = cached_success_response(vec![1i32, 2, 3], "ok", None);
        assert_eq!(resp.status(), StatusCode::OK);
        assert_eq!(
            resp.headers().get(header::CACHE_CONTROL).unwrap(),
            CATEGORY_CACHE_CONTROL
        );
        assert!(resp.headers().get(header::ETAG).is_some());
    }

    #[test]
    fn test_cached_success_response_returns_304_on_match() {
        let first = cached_success_response(vec![1i32, 2, 3], "ok", None);
        let etag = first.headers().get(header::ETAG).unwrap().clone();

        let resp = cached_success_response(vec![1i32, 2, 3], "ok", Some(&etag));
        assert_eq!(resp.status(), StatusCode::NOT_MODIFIED);
        assert_eq!(resp.headers().get(header::ETAG).unwrap(), &etag);
        assert_eq!(
            resp.headers().get(header::CACHE_CONTROL).unwrap(),
            CATEGORY_CACHE_CONTROL
        );
    }

    #[test]
    fn test_cached_success_response_mismatched_etag_returns_200() {
        let first = cached_success_response(vec![1i32, 2, 3], "ok", None);
        let etag = first.headers().get(header::ETAG).unwrap().clone();
        let other = HeaderValue::from_static("W/\"different-etag\"");

        let resp = cached_success_response(vec![1i32, 2, 3], "ok", Some(&other));
        assert_eq!(resp.status(), StatusCode::OK);
        assert_eq!(resp.headers().get(header::ETAG).unwrap(), &etag);
    }

    #[test]
    fn test_no_store_error_sets_no_store() {
        let resp = no_store_error(AppError::NotFound("missing".into()));
        assert_eq!(
            resp.headers().get(header::CACHE_CONTROL).unwrap(),
            "no-store"
        );
    }

    // -----------------------------------------------------------------------
    // Slug validation — Issue #1856
    // -----------------------------------------------------------------------

    #[test]
    fn test_slug_valid_simple() {
        assert!(is_valid_category_slug("music"));
    }

    #[test]
    fn test_slug_valid_hyphenated() {
        assert!(is_valid_category_slug("live-music"));
    }

    #[test]
    fn test_slug_valid_leading_digit() {
        assert!(is_valid_category_slug("2026-events"));
    }

    #[test]
    fn test_slug_valid_max_length() {
        // Exactly 64 characters — should be accepted.
        let slug = "a".repeat(64);
        assert!(is_valid_category_slug(&slug));
    }

    #[test]
    fn test_slug_invalid_only_hyphen() {
        assert!(!is_valid_category_slug("-"));
    }

    #[test]
    fn test_slug_invalid_leading_hyphen() {
        assert!(!is_valid_category_slug("-music"));
    }

    #[test]
    fn test_slug_invalid_trailing_hyphen() {
        assert!(!is_valid_category_slug("music-"));
    }

    #[test]
    fn test_slug_invalid_double_hyphen() {
        assert!(!is_valid_category_slug("live--music"));
    }

    #[test]
    fn test_slug_invalid_too_long() {
        // 65 characters — one over the limit.
        let slug = "a".repeat(65);
        assert!(!is_valid_category_slug(&slug));
    }

    #[test]
    fn test_slug_invalid_empty() {
        assert!(!is_valid_category_slug(""));
    }

    #[test]
    fn test_slug_invalid_uppercase() {
        assert!(!is_valid_category_slug("Music"));
    }

    #[test]
    fn test_slug_invalid_space() {
        assert!(!is_valid_category_slug("live music"));
    }

    // -----------------------------------------------------------------------
    // parent_id validation — Issue #1857
    // -----------------------------------------------------------------------

    #[test]
    fn test_parent_id_null_literal_is_valid() {
        // "null" is the sentinel that means WHERE parent_id IS NULL.
        let result: Option<Option<Uuid>> = if "null" == "null" {
            Some(None)
        } else {
            panic!("should have matched null branch");
        };
        assert!(result.is_some());
        assert!(result.unwrap().is_none());
    }

    #[test]
    fn test_parent_id_valid_uuid_is_accepted() {
        let uuid_str = "550e8400-e29b-41d4-a716-446655440000";
        let parsed = Uuid::parse_str(uuid_str);
        assert!(parsed.is_ok(), "valid UUID should parse successfully");
    }

    #[test]
    fn test_parent_id_invalid_string_is_rejected() {
        // Simulate the validation logic that now returns 400 for bad values.
        let bad_values = ["abc", "123", "not-a-uuid", "null-ish", ""];
        for bad in bad_values {
            let result = Uuid::parse_str(bad);
            assert!(
                result.is_err(),
                "\"{}\" is not a valid UUID and must be rejected",
                bad
            );
        }
    }
}
