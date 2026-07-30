use chrono::{DateTime, Utc};
use rust_decimal::Decimal;
use serde::{Deserialize, Serialize};
use sqlx::FromRow;
use uuid::Uuid;

/// Represents a pricing tier for an event (e.g., "General Admission", "VIP").
///
/// Each [`super::event::Event`] can have multiple tiers with different prices and
/// capacities. `available_quantity` is decremented as [`Ticket`]s are issued and
/// incremented on cancellation. Deleting an event cascades to all its tiers.
///
/// Maps to the `ticket_tiers` table in the database.
#[allow(dead_code)]
#[derive(Debug, Clone, Serialize, Deserialize, FromRow)]
pub struct TicketTier {
    /// Unique identifier for this ticket tier (UUID v4).
    pub id: Uuid,
    /// Foreign key referencing the [`super::event::Event`] this tier belongs to.
    pub event_id: Uuid,
    /// Display name of the tier (e.g., `"General Admission"`, `"VIP"`).
    pub name: String,
    /// Optional description of what this tier includes or its restrictions.
    pub description: Option<String>,
    /// Price per ticket in this tier, stored with up to 2 decimal places.
    pub price: Decimal,
    /// Maximum number of tickets that can ever be sold for this tier.
    pub total_quantity: i32,
    /// Current number of tickets still available for purchase.
    /// Starts equal to `total_quantity` and decreases as tickets are issued.
    pub available_quantity: i32,
    /// Timestamp when this tier was created.
    pub created_at: DateTime<Utc>,
    /// Timestamp of the last update to this record. Managed by a DB trigger.
    pub updated_at: DateTime<Utc>,
}

/// Represents a single ticket issued to a user for a specific ticket tier.
///
/// A ticket is the join between a [`super::user::User`] and a [`TicketTier`].
/// Its lifecycle is tracked via the `status` field. Each ticket may have an
/// associated [`super::transaction::Transaction`] recording the payment.
/// Deleting a user or tier cascades to their tickets.
///
/// Maps to the `tickets` table in the database.
#[allow(dead_code)]
#[derive(Debug, Clone, Serialize, Deserialize, FromRow)]
pub struct Ticket {
    /// Unique identifier for this ticket (UUID v4).
    pub id: Uuid,
    /// Foreign key referencing the [`super::user::User`] who owns this ticket.
    /// `None` for tickets synced directly from on-chain events before user matching.
    pub user_id: Option<Uuid>,
    /// Foreign key referencing the [`TicketTier`] this ticket was purchased under.
    /// `None` for tickets synced directly from on-chain events before tier matching.
    pub ticket_tier_id: Option<Uuid>,
    /// Current lifecycle status of the ticket.
    ///
    /// Known values (enforced at the application layer):
    /// - `"active"` — issued and valid for entry
    /// - `"used"` — scanned/checked in at the event
    /// - `"cancelled"` — refunded or voided; no longer valid
    pub status: String,
    /// Optional QR code payload used for event check-in scanning.
    /// `None` until the ticket is fully confirmed and a code is generated.
    pub qr_code: Option<String>,
    /// On-chain Stellar transaction / ticket ID from the smart contract.
    pub stellar_id: Option<String>,
    /// Foreign key to the event (denormalized for on-chain synced tickets).
    pub event_id: Option<Uuid>,
    /// Stellar wallet address of the buyer (who paid).
    pub buyer_wallet: Option<String>,
    /// Stellar wallet address of the owner (who holds the ticket; may differ from buyer).
    pub owner_wallet: Option<String>,
    /// Timestamp when this ticket was issued.
    pub created_at: DateTime<Utc>,
    /// Timestamp of the last update to this record. Managed by a DB trigger.
    pub updated_at: DateTime<Utc>,
}
