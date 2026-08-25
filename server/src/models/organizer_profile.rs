use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use serde_json::Value;
use sqlx::FromRow;

/// Represents an organizer's public-facing brand profile.
///
/// Keyed by the organizer's Stellar wallet `address` (the primary key).
/// This is separate from the internal [`super::organizer::Organizer`] record
/// and holds display metadata that organizers manage themselves.
///
/// Maps to the `organizer_profiles` table in the database.
#[derive(Debug, Clone, Serialize, Deserialize, FromRow)]
pub struct OrganizerProfile {
    /// Stellar wallet address — primary key.
    pub address: String,
    /// Public display name (max 50 chars).
    pub display_name: String,
    /// Optional biography (max 500 chars).
    pub bio: Option<String>,
    /// Optional URL to the organizer's avatar image.
    pub avatar_url: Option<String>,
    /// JSON object of social links (e.g. `{"twitter": "...", "website": "..."}`).
    pub socials: Value,
    /// Timestamp when the profile was first created.
    pub created_at: DateTime<Utc>,
    /// Timestamp of the last update. Managed by a DB trigger.
    pub updated_at: DateTime<Utc>,
}

/// Payload accepted by `PUT /api/v1/profile`.
#[derive(Debug, Deserialize)]
pub struct UpsertProfileRequest {
    /// Public display name (max 50 chars).
    pub display_name: String,
    /// Optional biography (max 500 chars).
    pub bio: Option<String>,
    /// Optional avatar URL.
    pub avatar_url: Option<String>,
    /// Social links object.
    #[serde(default)]
    pub socials: Value,
}
