use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use sqlx::FromRow;
use uuid::Uuid;

/// Represents an event category in the taxonomy tree.
///
/// Categories are hierarchical — a category may have a `parent_id` pointing
/// to another category, enabling nested taxonomies (e.g. Music > Jazz).
///
/// Maps to the `categories` table.
#[derive(Debug, Clone, Serialize, Deserialize, FromRow)]
pub struct Category {
    pub id: Uuid,
    pub name: String,
    pub slug: String,
    pub description: Option<String>,
    /// Optional parent category for hierarchical taxonomies.
    pub parent_id: Option<Uuid>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

/// Represents the many-to-many link between an event and a category.
///
/// Maps to the `event_categories` join table.
#[derive(Debug, Clone, Serialize, Deserialize, FromRow)]
pub struct EventCategory {
    pub event_id: Uuid,
    pub category_id: Uuid,
}
