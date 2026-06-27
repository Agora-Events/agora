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
}
