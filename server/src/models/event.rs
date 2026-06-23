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
    /// Whether the event is flagged for moderation.
    pub is_flagged: bool,
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
        };
        assert!(event.average_rating().is_none());
    }
}
