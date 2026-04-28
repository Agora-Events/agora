use chrono::{DateTime, Utc};
use rust_decimal::Decimal;
use serde::{Deserialize, Serialize};
use sqlx::FromRow;
use uuid::Uuid;

/// Represents a payment transaction associated with a ticket purchase.
///
/// Each [`super::ticket::Ticket`] has at most one transaction recording the
/// payment details. For on-chain payments, `stellar_transaction_hash` links
/// the record to the Stellar blockchain. Deleting a ticket cascades to its
/// transaction.
///
/// Maps to the `transactions` table in the database.
#[allow(dead_code)]
#[derive(Debug, Clone, Serialize, Deserialize, FromRow)]
pub struct Transaction {
    /// Unique identifier for this transaction (UUID v4).
    pub id: Uuid,
    /// Foreign key referencing the [`super::ticket::Ticket`] this payment is for.
    pub ticket_id: Uuid,
    /// Amount charged, stored with up to 2 decimal places.
    pub amount: Decimal,
    /// ISO 4217 currency code for the transaction (e.g., `"USD"`, `"USDC"`).
    /// Defaults to `"USD"` in the database.
    pub currency: String,
    /// Current processing status of the transaction.
    ///
    /// Known values (enforced at the application layer):
    /// - `"pending"` — payment initiated but not yet confirmed
    /// - `"completed"` — payment successfully settled
    /// - `"failed"` — payment was declined or encountered an error
    pub status: String,
    /// Optional Stellar blockchain transaction hash for on-chain payments.
    /// `None` for off-chain or fiat transactions.
    pub stellar_transaction_hash: Option<String>,
    /// Timestamp when this transaction record was created.
    pub created_at: DateTime<Utc>,
    /// Timestamp of the last update to this record. Managed by a DB trigger.
    pub updated_at: DateTime<Utc>,
}
