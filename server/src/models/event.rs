use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use sqlx::FromRow;
use uuid::Uuid;

/// Represents a ticketed event created by an organizer.
///
/// An event belongs to exactly one [`super::organizer::Organizer`] and can have
/// multiple [`super::ticket::TicketTier`]s defining pricing and capacity.
/// Deleting an organizer cascades to all their events.
///
/// Maps to the `events` table in the database.
#[allow(dead_code)]
#[derive(Debug, Clone, Serialize, Deserialize, FromRow)]
pub struct Event {
    /// Unique identifier for the event (UUID v4).
    pub id: Uuid,
    /// Foreign key referencing the [`super::organizer::Organizer`] who owns this event.
    pub organizer_id: Uuid,
    /// Short, public-facing title of the event.
    pub title: String,
    /// Optional detailed description of the event (agenda, speakers, etc.).
    pub description: Option<String>,
    /// Physical or virtual location where the event takes place.
    pub location: String,
    /// Scheduled start time of the event (UTC).
    pub start_time: DateTime<Utc>,
    /// Optional scheduled end time of the event (UTC). `None` if open-ended.
    pub end_time: Option<DateTime<Utc>>,
    /// Timestamp when this event record was created.
    pub created_at: DateTime<Utc>,
    /// Timestamp of the last update to this record. Managed by a DB trigger.
    pub updated_at: DateTime<Utc>,
}
