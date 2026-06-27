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
#[derive(Debug, Clone, Deserialize, FromRow)]
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
    /// Whether the event is flagged for moderation.
    pub is_flagged: bool,
    /// Whether this event has been marked as featured by an admin.
    pub is_featured: bool,
    /// Accumulated total of all star ratings for this event.
    pub sum_of_ratings: i64,
    /// Total number of ratings submitted for this event.
    pub count_of_ratings: i32,
    /// Timestamp when this event record was created.
    pub created_at: DateTime<Utc>,
    /// Timestamp of the last update to this record. Managed by a DB trigger.
    pub updated_at: DateTime<Utc>,
    /// Optional HTTPS URL for the event's banner/cover image.
    pub image_url: Option<String>,
    /// True when the event has no paid ticket tiers.
    /// Populated after fetch via `populate_is_free`; never read from DB.
    #[sqlx(skip)]
    pub is_free: bool,
    /// Number of tickets minted for this event. Loaded from DB for sorting; omitted from API.
    #[serde(skip)]
    pub minted_tickets: i64,
}

impl Event {
    /// Returns the average star rating for the event if any ratings exist.
    pub fn average_rating(&self) -> Option<f32> {
        if self.count_of_ratings == 0 {
            None
        } else {
            Some(self.sum_of_ratings as f32 / self.count_of_ratings as f32)
        }
    }
}

/// Custom serialization that adds the computed `average_rating` field alongside
/// the raw columns, so clients don't have to derive it from
/// `sum_of_ratings` / `count_of_ratings` (Issue #584). It is `null` when the
/// event has no ratings.
impl Serialize for Event {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        use serde::ser::SerializeStruct;

        let mut state = serializer.serialize_struct("Event", 15)?;
        state.serialize_field("id", &self.id)?;
        state.serialize_field("organizer_id", &self.organizer_id)?;
        state.serialize_field("title", &self.title)?;
        state.serialize_field("description", &self.description)?;
        state.serialize_field("location", &self.location)?;
        state.serialize_field("start_time", &self.start_time)?;
        state.serialize_field("end_time", &self.end_time)?;
        state.serialize_field("is_flagged", &self.is_flagged)?;
        state.serialize_field("is_featured", &self.is_featured)?;
        state.serialize_field("sum_of_ratings", &self.sum_of_ratings)?;
        state.serialize_field("count_of_ratings", &self.count_of_ratings)?;
        state.serialize_field("created_at", &self.created_at)?;
        state.serialize_field("updated_at", &self.updated_at)?;
        state.serialize_field("image_url", &self.image_url)?;
        state.serialize_field("is_free", &self.is_free)?;
        state.serialize_field("average_rating", &self.average_rating())?;
        state.end()
    }
}

/// Populate the `is_free` field on a batch of events with a single query.
///
/// Events that have at least one ticket tier with `price > 0` are considered
/// paid; all others are free.  A Redis fallback is intentionally not used here
/// because the source-of-truth is always the ticket_tiers table.
pub async fn populate_is_free(events: &mut [Event], pool: &sqlx::PgPool) {
    if events.is_empty() {
        return;
    }

    let ids: Vec<Uuid> = events.iter().map(|e| e.id).collect();

    // Fetch only the IDs of events that have at least one paid tier.
    let paid_ids: Vec<Uuid> = match sqlx::query_scalar::<_, Uuid>(
        "SELECT DISTINCT event_id FROM ticket_tiers WHERE event_id = ANY($1) AND price > 0",
    )
    .bind(&ids)
    .fetch_all(pool)
    .await
    {
        Ok(rows) => rows,
        Err(e) => {
            tracing::warn!("populate_is_free: ticket_tiers query failed: {:?}", e);
            return;
        }
    };

    for event in events.iter_mut() {
        event.is_free = !paid_ids.contains(&event.id);
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_is_free_defaults_false() {
        // When sqlx skips the field, the default is false.
        let event = Event {
            id: Uuid::new_v4(),
            organizer_id: Uuid::new_v4(),
            title: "Test".into(),
            description: None,
            location: "Lagos".into(),
            start_time: DateTime::default(),
            end_time: None,
            is_flagged: false,
            sum_of_ratings: 0,
            count_of_ratings: 0,
            created_at: DateTime::default(),
            updated_at: DateTime::default(),
            image_url: None,
            is_free: false,
            minted_tickets: 0,
        };
        assert!(!event.is_free);
    }

    #[test]
    fn test_is_free_serializes() {
        let mut event = Event {
            id: Uuid::new_v4(),
            organizer_id: Uuid::new_v4(),
            title: "Free Concert".into(),
            description: None,
            location: "Abuja".into(),
            start_time: DateTime::default(),
            end_time: None,
            is_flagged: false,
            sum_of_ratings: 0,
            count_of_ratings: 0,
            created_at: DateTime::default(),
            updated_at: DateTime::default(),
            image_url: None,
            is_free: false,
            minted_tickets: 0,
        };
        event.is_free = true;
        let json = serde_json::to_value(&event).unwrap();
        assert_eq!(json["is_free"], true);
    }

    #[test]
    fn test_average_rating_none_when_no_ratings() {
        let event = Event {
            id: Uuid::new_v4(),
            organizer_id: Uuid::new_v4(),
            title: "T".into(),
            description: None,
            location: "L".into(),
            start_time: DateTime::default(),
            end_time: None,
            is_flagged: false,
            sum_of_ratings: 0,
            count_of_ratings: 0,
            created_at: DateTime::default(),
            updated_at: DateTime::default(),
            image_url: None,
            is_free: false,
            minted_tickets: 0,
        };
        assert!(event.average_rating().is_none());
    }

    #[test]
    fn test_average_rating_serialized_when_ratings_exist() {
        let event = Event {
            id: Uuid::new_v4(),
            organizer_id: Uuid::new_v4(),
            title: "Rated".into(),
            description: None,
            location: "L".into(),
            start_time: DateTime::default(),
            end_time: None,
            is_flagged: false,
            sum_of_ratings: 45,
            count_of_ratings: 10,
            created_at: DateTime::default(),
            updated_at: DateTime::default(),
            image_url: None,
            is_free: false,
            minted_tickets: 0,
        };
        let json = serde_json::to_value(&event).unwrap();
        assert_eq!(json["average_rating"], 4.5);
    }

    #[test]
    fn test_created_at_and_updated_at_serialized() {
        use chrono::TimeZone;
        let created = Utc.with_ymd_and_hms(2026, 5, 1, 10, 0, 0).unwrap();
        let updated = Utc.with_ymd_and_hms(2026, 5, 20, 14, 30, 0).unwrap();
        let event = Event {
            id: Uuid::new_v4(),
            organizer_id: Uuid::new_v4(),
            title: "T".into(),
            description: None,
            location: "L".into(),
            start_time: created,
            end_time: None,
            is_flagged: false,
            sum_of_ratings: 0,
            count_of_ratings: 0,
            created_at: created,
            updated_at: updated,
            image_url: None,
            is_free: false,
            minted_tickets: 0,
        };
        let json = serde_json::to_value(&event).unwrap();
        assert!(!json["created_at"].is_null(), "created_at must be present");
        assert!(!json["updated_at"].is_null(), "updated_at must be present");
    }

    #[test]
    fn test_average_rating_serialized_null_when_no_ratings() {
        let event = Event {
            id: Uuid::new_v4(),
            organizer_id: Uuid::new_v4(),
            title: "Unrated".into(),
            description: None,
            location: "L".into(),
            start_time: DateTime::default(),
            end_time: None,
            is_flagged: false,
            sum_of_ratings: 0,
            count_of_ratings: 0,
            created_at: DateTime::default(),
            updated_at: DateTime::default(),
            image_url: None,
            is_free: false,
            minted_tickets: 0,
        };
        let json = serde_json::to_value(&event).unwrap();
        assert!(json["average_rating"].is_null());
    }
}
