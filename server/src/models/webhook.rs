use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use sqlx::FromRow;
use utoipa::ToSchema;
use uuid::Uuid;

/// Webhook events the dispatcher can emit (Issue #1339).
pub const WEBHOOK_EVENT_PAYMENT_PROCESSED: &str = "PaymentProcessed";
pub const WEBHOOK_EVENT_EVENT_CREATED: &str = "EventCreated";
pub const WEBHOOK_EVENT_TICKET_SCANNED: &str = "TicketScanned";

/// All supported webhook event names.
pub fn supported_webhook_events() -> [&'static str; 3] {
    [
        WEBHOOK_EVENT_PAYMENT_PROCESSED,
        WEBHOOK_EVENT_EVENT_CREATED,
        WEBHOOK_EVENT_TICKET_SCANNED,
    ]
}

/// An organiser-registered HTTPS endpoint that receives signed webhook POSTs.
#[derive(Debug, Clone, Serialize, Deserialize, FromRow, ToSchema)]
pub struct WebhookEndpoint {
    pub id: Uuid,
    pub organizer_id: Uuid,
    pub url: String,
    pub secret: String,
    pub events: Vec<String>,
    pub is_active: bool,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

/// Request body for `POST /api/v1/webhooks`.
#[derive(Debug, Clone, Deserialize, ToSchema)]
pub struct CreateWebhookRequest {
    pub organizer_id: Uuid,
    /// HTTPS URL that will receive signed POST payloads.
    pub url: String,
    /// Shared secret used to generate the `X-Agora-Signature` HMAC.
    pub secret: String,
    /// Subscribed event names, e.g. `["PaymentProcessed", "EventCreated"]`.
    #[serde(default)]
    pub events: Vec<String>,
}

/// One delivery attempt (or failure) recorded for an endpoint.
#[derive(Debug, Clone, Serialize, Deserialize, FromRow, ToSchema)]
pub struct WebhookDeliveryLog {
    pub id: Uuid,
    pub endpoint_id: Uuid,
    pub event: String,
    pub payload: serde_json::Value,
    pub attempt: i32,
    pub status_code: Option<i32>,
    pub error: Option<String>,
    pub delivered_at: Option<DateTime<Utc>>,
    pub created_at: DateTime<Utc>,
}

/// Default max registered endpoints per organiser.
pub const MAX_WEBHOOK_ENDPOINTS_PER_ORGANIZER: usize = 5;

/// Maximum URL length.
pub const MAX_WEBHOOK_URL_LEN: usize = 2048;

/// Maximum secret length.
pub const MAX_WEBHOOK_SECRET_LEN: usize = 512;

/// Validates a create-webhook request, deduplicating and normalising events.
pub fn validate_create_webhook(req: &CreateWebhookRequest) -> Result<Vec<String>, String> {
    let url = req.url.trim();
    if url.is_empty() || !url.starts_with("https://") || url.len() > MAX_WEBHOOK_URL_LEN {
        return Err("url must be a valid HTTPS URL".to_string());
    }
    if req.secret.trim().is_empty() || req.secret.len() > MAX_WEBHOOK_SECRET_LEN {
        return Err(format!(
            "secret must be between 1 and {MAX_WEBHOOK_SECRET_LEN} characters"
        ));
    }
    if req.events.is_empty() {
        return Err("at least one event must be subscribed".to_string());
    }

    let supported = supported_webhook_events();
    let mut seen = std::collections::HashSet::new();
    let mut events = Vec::new();
    for raw in &req.events {
        let event = raw.trim();
        if event.is_empty() {
            continue;
        }
        if !supported.contains(&event) {
            return Err(format!("unsupported webhook event '{event}'"));
        }
        if seen.insert(event.to_string()) {
            events.push(event.to_string());
        }
    }
    if events.is_empty() {
        return Err("at least one valid event must be subscribed".to_string());
    }
    Ok(events)
}
