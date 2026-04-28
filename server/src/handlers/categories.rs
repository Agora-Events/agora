//! # Category Handlers
//!
//! This module provides HTTP handlers for category-related operations.

use axum::{extract::{Query, State}, response::IntoResponse, response::Response};
use serde::Deserialize;
use sqlx::PgPool;
use uuid::Uuid;

use crate::models::category::Category;
use crate::utils::error::AppError;
use crate::utils::pagination::{PaginatedResponse, PaginationParams};
use crate::utils::response::success;

/// Query parameters for filtering categories
#[derive(Debug, Deserialize)]
pub struct CategoryFilters {
    /// Filter by parent category ID (use "null" for root categories)
    pub parent_id: Option<String>,
    
    /// Search in name and description
    pub search: Option<String>,
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
    State(pool): State<PgPool>,
    Query(pagination): Query<PaginationParams>,
    Query(filters): Query<CategoryFilters>,
) -> Response {
    let validated_pagination = pagination.validate();
    
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
    
    if parent_filter.is_some() {
        param_count += 1;
        if parent_filter.as_ref().unwrap().is_none() {
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
    
    let total = match count_query_builder.fetch_one(&pool).await {
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
    
    let items = match items_query_builder.fetch_all(&pool).await {
        Ok(categories) => categories,
        Err(e) => {
            tracing::error!("Failed to fetch categories: {:?}", e);
            return AppError::DatabaseError(e).into_response();
        }
    };
    
    let response = PaginatedResponse::new(items, validated_pagination, total);
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
    let category = match sqlx::query_as::<_, Category>(
        "SELECT * FROM categories WHERE id = $1"
    )
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
