//! # Category Handlers
//!
//! This module provides HTTP handlers for category-related operations.

use axum::{
    extract::{Query, State},
    response::IntoResponse,
    response::Response,
};
use serde::Deserialize;
use sqlx::PgPool;
use std::time::Duration;
use uuid::Uuid;

use crate::cache::RedisCache;
use crate::models::category::Category;
use crate::utils::error::AppError;
use crate::utils::pagination::{PaginatedResponse, PaginationParams};
use crate::utils::response::success;

/// TTL for cached category listings. Categories are effectively static, so a
/// long TTL (1 hour) sharply reduces database load under traffic (Issue #583).
const CATEGORIES_CACHE_TTL: Duration = Duration::from_secs(3600);

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
    Query(pagination): Query<PaginationParams>,
    Query(filters): Query<CategoryFilters>,
) -> Response {
    let validated_pagination = pagination.validate();

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
            return success(cached, "Categories retrieved successfully (cached)").into_response();
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

    // Handle parent_id filter (including "null" for root categories)
    let parent_filter = if let Some(ref parent_str) = filters.parent_id {
        if parent_str == "null" {
            Some(None) // Filter for NULL parent_id
        } else if let Ok(uuid) = Uuid::parse_str(parent_str) {
            Some(Some(uuid)) // Filter for specific parent_id
        } else {
            None // Invalid UUID, ignore filter
        }
    } else {
        None // No filter
    };

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
            return AppError::DatabaseError(e).into_response();
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
            return AppError::DatabaseError(e).into_response();
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

    success(response, "Categories retrieved successfully").into_response()
}

/// Get a single category by ID
///
/// # Endpoint
/// GET `/api/v1/categories/:id`
pub async fn get_category(
    State(pool): State<PgPool>,
    axum::extract::Path(category_id): axum::extract::Path<Uuid>,
) -> Response {
    let category = match sqlx::query_as::<_, Category>("SELECT * FROM categories WHERE id = $1")
        .bind(category_id)
        .fetch_optional(&pool)
        .await
    {
        Ok(Some(category)) => category,
        Ok(None) => {
            return AppError::NotFound(format!("Category with id '{}' not found", category_id))
                .into_response();
        }
        Err(e) => {
            tracing::error!("Failed to fetch category: {:?}", e);
            return AppError::DatabaseError(e).into_response();
        }
    };

    success(category, "Category retrieved successfully").into_response()
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
}
