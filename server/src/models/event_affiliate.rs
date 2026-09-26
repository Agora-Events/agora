use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use sqlx::FromRow;
use uuid::Uuid;

/// An affiliate registration: one wallet promoting one event (Issue #1151).
///
/// The referral code is globally unique rather than unique-per-event, so a
/// code arriving on a checkout link resolves on its own without the event id
/// having to be carried alongside it.
///
/// Uniqueness of `(event_id, wallet_address)` is enforced by the database, so
/// two concurrent registration requests for the same wallet cannot both
/// succeed.
///
/// Maps to the `event_affiliates` table.
#[allow(dead_code)]
#[derive(Debug, Clone, Serialize, Deserialize, FromRow)]
pub struct EventAffiliate {
    /// Unique identifier for this registration (UUID v4).
    pub id: Uuid,
    /// The event being promoted.
    pub event_id: Uuid,
    /// Wallet address of the affiliate, used to attribute and pay out.
    pub wallet_address: String,
    /// Globally unique code identifying this affiliate on referral links.
    pub referral_code: String,
    /// When the registration was created.
    pub created_at: DateTime<Utc>,
    /// Last update. Managed by a database trigger.
    pub updated_at: DateTime<Utc>,
}
