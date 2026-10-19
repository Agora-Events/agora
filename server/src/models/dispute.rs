use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use sqlx::FromRow;
use uuid::Uuid;

/// A governance dispute opened against a concluded event.
///
/// Maps to the `disputes` table in the database.
#[derive(Debug, Clone, Serialize, Deserialize, FromRow)]
pub struct Dispute {
    pub id: Uuid,
    pub event_id: Uuid,
    pub opened_by: Uuid,
    pub status: String,
    pub closes_at: DateTime<Utc>,
    pub resolved_at: Option<DateTime<Utc>>,
    pub ruling: Option<String>,
    pub total_eligible_tickets: i32,
    pub buyer_votes: i32,
    pub organizer_votes: i32,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

/// A single mediation vote cast by a user within a dispute.
///
/// Maps to the `mediation_votes` table in the database.
#[derive(Debug, Clone, Serialize, Deserialize, FromRow)]
pub struct MediationVote {
    pub id: Uuid,
    pub dispute_id: Uuid,
    pub voter_id: Uuid,
    pub vote: String,
    pub voted_at: DateTime<Utc>,
}
