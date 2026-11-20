use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use sqlx::FromRow;
use utoipa::ToSchema;
use url::Url;
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
    #[serde(skip_serializing)]
    pub secret: String,
    pub events: Vec<String>,
    pub is_active: bool,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

/// A view of WebhookEndpoint without the secret, for safe listing/response serialization.
#[derive(Debug, Clone, Serialize, Deserialize, FromRow, ToSchema)]
pub struct WebhookEndpointView {
    pub id: Uuid,
    pub organizer_id: Uuid,
    pub url: String,
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
/// Also validates the URL to prevent SSRF (Server-Side Request Forgery) by
/// rejecting private, loopback, link-local, and other reserved IP addresses.
pub fn validate_create_webhook(req: &CreateWebhookRequest) -> Result<Vec<String>, String> {
    let url_str = req.url.trim();
    if url_str.is_empty() || !url_str.starts_with("https://") || url_str.len() > MAX_WEBHOOK_URL_LEN {
        return Err("url must be a valid HTTPS URL".to_string());
    }

    let parsed = match Url::parse(url_str) {
        Ok(u) => u,
        Err(_) => return Err("url must be a valid HTTPS URL".to_string()),
    };

    // Require a host (reject data: URIs, mailto:, etc. without proper host)
    let host = parsed.host();
    if host.is_none() {
        return Err("url must contain a valid host".to_string());
    }
    let host = host.unwrap();

    // Reject localhost
    if host == "localhost" || host == "local" {
        return Err("url must not use localhost".to_string());
    }

    // Reject IP-literal hosts that are in protected ranges
    // Parse as IP address to check ranges
    if let Ok(ip) = std::net::IpAddr::from_str(host) {
        // IPv4 loopback: 127.0.0.0/8
        // IPv6 loopback: ::1
        if ip.is_loopback() {
            return Err("url must not use loopback address".to_string());
        }

        // IPv4 private: 10.0.0.0/8, 172.16.0.0/12, 192.168.0.0/16
        // IPv6 unique-local: fc00::/8 (but we check fd00::/8 specifically)
        if ip.is_private() || ip.is_ftos() || ip.is_link_local() {
            return Err("url must not use private or link-local address".to_string());
        }

        // IPv4 link-local: 169.254.0.0/16
        // IPv6 link-local: fe80::/10
        if ip.is_multicast() {
            return Err("url must not use multicast address".to_string());
        }

        // IPv4 unspecified: 0.0.0.0
        // IPv6 unspecified: ::
        if ip.is_unspecified() {
            return Err("url must not use unspecified address".to_string());
        }
    } else {
        // It's a hostname - check if it resolves to a private/loopback address
        // We'll do basic hostname checks; full DNS resolution guarded in dispatcher
        if host == "localhost" {
            return Err("url must not use localhost".to_string());
        }
    }

    // Validate secret
    if req.secret.trim().is_empty() || req.secret.len() > MAX_WEBHOOK_SECRET_LEN {
        return Err(format!(
            "secret must be between 1 and {MAX_WEBHOOK_SECRET_LEN} characters"
        ));
    }

    // Validate events
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

/// Unit tests for `validate_create_webhook` SSRF protections (Issue #1801).
#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn validation_accepts_public_ip() {
        let req = CreateWebhookRequest {
            organizer_id: Uuid::new_v4(),
            url: "https://example.com/webhook".to_string(),
            secret: "s3cret".to_string(),
            events: vec!["PaymentProcessed".to_string()],
        };
        assert!(validate_create_webhook(&req).is_ok());
    }

    #[test]
    fn validation_rejects_loopback_ipv4() {
        let req = CreateWebhookRequest {
            organizer_id: Uuid::new_v4(),
            url: "https://127.0.0.1/hook".to_string(),
            secret: "s3cret".to_string(),
            events: vec!["PaymentProcessed".to_string()],
        };
        assert!(validate_create_webhook(&req).is_err());
    }

    #[test]
    fn validation_rejects_loopback_ipv6() {
        let req = CreateWebhookRequest {
            organizer_id: Uuid::new_v4(),
            url: "https://[::1]/hook".to_string(),
            secret: "s3cret".to_string(),
            events: vec!["PaymentProcessed".to_string()],
        };
        assert!(validate_create_webhook(&req).is_err());
    }

    #[test]
    fn validation_rejects_private_ipv4_10() {
        let req = CreateWebhookRequest {
            organizer_id: Uuid::new_v4(),
            url: "https://10.0.0.5/hook".to_string(),
            secret: "s3cret".to_string(),
            events: vec!["PaymentProcessed".to_string()],
        };
        assert!(validate_create_webhook(&req).is_err());
    }

    #[test]
    fn validation_rejects_private_ipv4_172() {
        let req = CreateWebhookRequest {
            organizer_id: Uuid::new_v4(),
            url: "https://172.16.0.1/hook".to_string(),
            secret: "s3cret".to_string(),
            events: vec!["PaymentProcessed".to_string()],
        };
        assert!(validate_create_webhook(&req).is_err());
    }

    #[test]
    fn validation_rejects_private_ipv4_192() {
        let req = CreateWebhookRequest {
            organizer_id: Uuid::new_v4(),
            url: "https://192.168.1.1/hook".to_string(),
            secret: "s3cret".to_string(),
            events: vec!["PaymentProcessed".to_string()],
        };
        assert!(validate_create_webhook(&req).is_err());
    }

    #[test]
    fn validation_rejects_link_local_ipv4() {
        let req = CreateWebhookRequest {
            organizer_id: Uuid::new_v4(),
            url: "https://169.254.169.254/hook".to_string(),
            secret: "s3cret".to_string(),
            events: vec!["PaymentProcessed".to_string()],
        };
        assert!(validate_create_webhook(&req).is_err());
    }

    #[test]
    fn validation_rejects_localhost_hostname() {
        let req = CreateWebhookRequest {
            organizer_id: Uuid::new_v4(),
            url: "https://localhost:6379/hook".to_string(),
            secret: "s3cret".to_string(),
            events: vec!["PaymentProcessed".to_string()],
        };
        assert!(validate_create_webhook(&req).is_err());
    }

    #[test]
    fn validation_accepts_normal_hostname() {
        let req = CreateWebhookRequest {
            organizer_id: Uuid::new_v4(),
            url: "https://api.example.com/hook".to_string(),
            secret: "s3cret".to_string(),
            events: vec!["PaymentProcessed".to_string()],
        };
        assert!(validate_create_webhook(&req).is_ok());
    }
}

/// A permanently failed webhook delivery preserved in the Dead-Letter Queue (DLQ).
#[derive(Debug, Clone, Serialize, Deserialize, FromRow, ToSchema)]
pub struct WebhookDeliveryFailure {
    pub id: Uuid,
    pub endpoint_id: Uuid,
    pub event: String,
    pub payload: serde_json::Value,
    pub attempts: i32,
    pub last_status_code: Option<i32>,
    pub last_error: Option<String>,
    pub failed_at: DateTime<Utc>,
    pub retried_at: Option<DateTime<Utc>>,
    pub retry_status: Option<String>,
}

