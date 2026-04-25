use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use sqlx::FromRow;
use uuid::Uuid;

/// Represents a single administrative action recorded in the audit trail.
///
/// Every request that hits an `/api/v1/admin/*` route is persisted here,
/// giving operators a tamper-evident log of who did what and when.
///
/// Maps to the `audit_logs` table.
#[derive(Debug, Clone, Serialize, Deserialize, FromRow)]
pub struct AuditLog {
    pub id: Uuid,
    /// The admin user who performed the action. `None` for system/unauthenticated calls.
    pub actor_id: Option<Uuid>,
    /// Dot-separated action name, e.g. `"admin.event.delete"`.
    pub action: String,
    /// The type of resource affected, e.g. `"event"`.
    pub resource_type: Option<String>,
    /// The ID of the affected resource.
    pub resource_id: Option<String>,
    pub request_path: String,
    pub request_method: String,
    /// Captured request body as JSON (may be `None` for GET/DELETE).
    pub request_body: Option<serde_json::Value>,
    pub ip_address: Option<String>,
    /// HTTP status code returned to the caller.
    pub status_code: Option<i32>,
    pub created_at: DateTime<Utc>,
}
