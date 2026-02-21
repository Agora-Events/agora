use chrono::{DateTime, Utc};
use rust_decimal::Decimal;
use serde::{Deserialize, Serialize};
use sqlx::FromRow;
use uuid::Uuid;

#[allow(dead_code)]
#[derive(Debug, Clone, Serialize, Deserialize, FromRow)]
pub struct Refund {
    pub id: Uuid,
    pub ticket_id: Uuid,
    pub amount: Decimal,
    pub reason: String,
    pub status: String,       // pending, approved, rejected, completed
    pub initiated_by: String, // 'guest' or 'organizer'
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RefundRequest {
    pub ticket_id: Uuid,
    pub reason: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RefundResponse {
    pub id: Uuid,
    pub ticket_id: Uuid,
    pub amount: Decimal,
    pub status: String,
    pub message: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EventCancellationRequest {
    pub event_id: Uuid,
    pub reason: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EventCancellationResponse {
    pub event_id: Uuid,
    pub refunds_processed: i32,
    pub total_refunded: Decimal,
    pub message: String,
}
