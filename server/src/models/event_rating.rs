use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use sqlx::FromRow;
use uuid::Uuid;

/// Represents a rating or review left by an attendee for an event.
#[derive(Debug, Clone, Serialize, Deserialize, FromRow)]
pub struct EventRating {
    pub id: Uuid,
    pub event_id: Uuid,
    pub ticket_id: Uuid,
    pub rating: i16,
    pub review: Option<String>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}
