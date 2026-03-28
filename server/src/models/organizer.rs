use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use sqlx::FromRow;
use uuid::Uuid;

/// Represents an event organizer on the Agora platform.
///
/// Organizers are entities (individuals or organizations) that create and manage
/// [`super::event::Event`]s. An organizer can own multiple events; deleting an
/// organizer cascades to all their events.
///
/// Maps to the `organizers` table in the database.
#[allow(dead_code)]
#[derive(Debug, Clone, Serialize, Deserialize, FromRow)]
pub struct Organizer {
    /// Unique identifier for the organizer (UUID v4).
    pub id: Uuid,
    /// Public-facing name of the organizer (e.g., company or individual name).
    pub name: String,
    /// Optional bio or description shown on the organizer's profile page.
    pub description: Option<String>,
    /// Primary contact email for the organizer, used for platform communications.
    pub contact_email: String,
    /// Timestamp when the organizer account was created.
    pub created_at: DateTime<Utc>,
    /// Timestamp of the last update to this record. Managed by a DB trigger.
    pub updated_at: DateTime<Utc>,
}
