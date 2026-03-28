use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use sqlx::FromRow;
use uuid::Uuid;

/// Represents a registered user of the Agora platform.
///
/// Users are the attendees who browse events, purchase tickets, and attend events.
/// They are distinct from [`super::organizer::Organizer`]s, who create and manage events.
///
/// Maps to the `users` table in the database.
#[allow(dead_code)]
#[derive(Debug, Clone, Serialize, Deserialize, FromRow)]
pub struct User {
    /// Unique identifier for the user (UUID v4).
    pub id: Uuid,
    /// Display name of the user.
    pub name: String,
    /// Unique email address used for identification and communication.
    pub email: String,
    /// Timestamp when the user account was created.
    pub created_at: DateTime<Utc>,
    /// Timestamp of the last update to this record. Managed by a DB trigger.
    pub updated_at: DateTime<Utc>,
}
