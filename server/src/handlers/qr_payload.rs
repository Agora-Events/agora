//! # QR Payload Handler
//!
//! This module provides endpoints for generating and verifying cryptographically
//! signed QR code payloads. The signatures ensure payload integrity and authenticity.
//!
//! ## Endpoints
//! - POST `/api/v1/qr/generate` - Generate a signed QR payload
//! - POST `/api/v1/qr/verify` - Verify a signed QR payload
//!
//! ## Cryptography
//! Uses Ed25519 digital signatures for payload signing and verification.

use axum::{extract::Path, extract::Query, extract::State, response::IntoResponse, response::Response, Json};
use base64::{engine::general_purpose, Engine as _};
use chrono::{DateTime, Duration, Utc};
use ed25519_dalek::{Signature, Signer, SigningKey, Verifier, VerifyingKey};
use rand::rngs::OsRng;
use serde::{Deserialize, Serialize};
use sqlx::PgPool;
use uuid::Uuid;

use crate::utils::error::AppError;
use crate::utils::pagination::{PaginatedResponse, PaginationParams};
use crate::utils::response::success;

/// Request body for generating a signed QR payload
#[derive(Debug, Deserialize)]
pub struct GenerateQrRequest {
    /// Type of QR code (e.g., "ticket", "payment", "access")
    pub qr_type: String,
    /// Associated data (e.g., event_id, ticket_id, amount)
    pub data: serde_json::Value,
    /// Optional expiration time in seconds (default: 3600)
    pub expires_in_seconds: Option<i64>,
}

/// Response containing the signed QR payload
#[derive(Debug, Serialize)]
pub struct GenerateQrResponse {
    /// Unique identifier for this QR code
    pub qr_id: String,
    /// The payload data
    pub payload: QrPayload,
    /// Base64-encoded signature
    pub signature: String,
    /// Public key for verification (hex-encoded)
    pub public_key: String,
    /// Expiration timestamp
    pub expires_at: DateTime<Utc>,
}

/// The QR payload structure that gets signed
#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct QrPayload {
    /// Unique identifier
    pub id: String,
    /// Type of QR code
    pub qr_type: String,
    /// Associated data
    pub data: serde_json::Value,
    /// Creation timestamp
    pub created_at: DateTime<Utc>,
    /// Expiration timestamp
    pub expires_at: DateTime<Utc>,
    /// Nonce for uniqueness
    pub nonce: String,
}

/// Request body for verifying a signed QR payload
#[derive(Debug, Deserialize)]
pub struct VerifyQrRequest {
    /// The QR payload to verify
    pub payload: QrPayload,
    /// Base64-encoded signature
    pub signature: String,
    /// Public key for verification (hex-encoded)
    pub public_key: String,
}

/// Response for QR verification
#[derive(Debug, Serialize)]
pub struct VerifyQrResponse {
    /// Whether the signature is valid
    pub valid: bool,
    /// Whether the payload has expired
    pub expired: bool,
    /// The verified payload (if valid)
    pub payload: Option<QrPayload>,
    /// Verification message
    pub message: String,
}

/// Generate a cryptographically signed QR payload
///
/// # Endpoint
/// POST `/api/v1/qr/generate`
///
/// # Request Body
/// ```json
/// {
///   "qr_type": "ticket",
///   "data": {
///     "event_id": "123",
///     "ticket_id": "456",
///     "seat": "A12"
///   },
///   "expires_in_seconds": 3600
/// }
/// ```
///
/// # Response
/// Returns a signed QR payload with signature and public key for verification
pub async fn generate_qr_payload(
    State(pool): State<PgPool>,
    Json(request): Json<GenerateQrRequest>,
) -> Response {
    // Validate QR type
    if request.qr_type.is_empty() {
        return AppError::ValidationError("qr_type cannot be empty".to_string()).into_response();
    }

    // Generate unique ID and nonce
    let qr_id = Uuid::new_v4().to_string();
    let nonce = Uuid::new_v4().to_string();

    // Calculate expiration
    let created_at = Utc::now();
    let expires_in = request.expires_in_seconds.unwrap_or(3600);
    let expires_at = created_at + Duration::seconds(expires_in);

    // Create payload
    let payload = QrPayload {
        id: qr_id.clone(),
        qr_type: request.qr_type.clone(),
        data: request.data.clone(),
        created_at,
        expires_at,
        nonce,
    };

    // Serialize payload for signing
    let payload_json = match serde_json::to_string(&payload) {
        Ok(json) => json,
        Err(e) => {
            return AppError::InternalServerError(format!("Failed to serialize payload: {}", e))
                .into_response();
        }
    };

    // Generate Ed25519 keypair
    let mut csprng = OsRng;
    let signing_key = SigningKey::generate(&mut csprng);
    let verifying_key = signing_key.verifying_key();

    // Sign the payload
    let signature = signing_key.sign(payload_json.as_bytes());
    let signature_base64 = general_purpose::STANDARD.encode(signature.to_bytes());
    let public_key_hex = hex::encode(verifying_key.to_bytes());

    // Store in database
    let result = sqlx::query(
        r#"
        INSERT INTO qr_payloads (
            id, qr_type, payload_data, signature, public_key, 
            created_at, expires_at, is_used
        )
        VALUES ($1, $2, $3, $4, $5, $6, $7, $8)
        "#,
    )
    .bind(&qr_id)
    .bind(&request.qr_type)
    .bind(&request.data)
    .bind(&signature_base64)
    .bind(&public_key_hex)
    .bind(created_at)
    .bind(expires_at)
    .bind(false)
    .execute(&pool)
    .await;

    if let Err(e) = result {
        tracing::error!("Failed to store QR payload: {:?}", e);
        return AppError::DatabaseError(e).into_response();
    }

    let response = GenerateQrResponse {
        qr_id,
        payload,
        signature: signature_base64,
        public_key: public_key_hex,
        expires_at,
    };

    success(response, "QR payload generated successfully").into_response()
}

/// Verify a cryptographically signed QR payload
///
/// # Endpoint
/// POST `/api/v1/qr/verify`
///
/// # Request Body
/// ```json
/// {
///   "payload": { ... },
///   "signature": "base64_signature",
///   "public_key": "hex_public_key"
/// }
/// ```
///
/// # Response
/// Returns verification status and payload details
pub async fn verify_qr_payload(
    State(pool): State<PgPool>,
    Json(request): Json<VerifyQrRequest>,
) -> Response {
    // Serialize payload for verification
    let payload_json = match serde_json::to_string(&request.payload) {
        Ok(json) => json,
        Err(e) => {
            return AppError::ValidationError(format!("Invalid payload format: {}", e))
                .into_response();
        }
    };

    // Decode signature from base64
    let signature_bytes = match general_purpose::STANDARD.decode(&request.signature) {
        Ok(bytes) => bytes,
        Err(e) => {
            return AppError::ValidationError(format!("Invalid signature encoding: {}", e))
                .into_response();
        }
    };

    let signature = match Signature::from_slice(&signature_bytes) {
        Ok(sig) => sig,
        Err(e) => {
            return AppError::ValidationError(format!("Invalid signature format: {}", e))
                .into_response();
        }
    };

    // Decode public key from hex
    let public_key_bytes = match hex::decode(&request.public_key) {
        Ok(bytes) => bytes,
        Err(e) => {
            return AppError::ValidationError(format!("Invalid public key encoding: {}", e))
                .into_response();
        }
    };

    let public_key_array: [u8; 32] = match public_key_bytes.try_into() {
        Ok(arr) => arr,
        Err(_) => {
            return AppError::ValidationError("Public key must be 32 bytes".to_string())
                .into_response();
        }
    };

    let verifying_key = match VerifyingKey::from_bytes(&public_key_array) {
        Ok(key) => key,
        Err(e) => {
            return AppError::ValidationError(format!("Invalid public key: {}", e)).into_response();
        }
    };

    // Verify signature
    let is_valid = verifying_key
        .verify(payload_json.as_bytes(), &signature)
        .is_ok();

    // Check expiration
    let is_expired = request.payload.expires_at < Utc::now();

    // Check if already used (if valid)
    let mut is_used = false;
    if is_valid {
        if let Ok(Some((used,))) = sqlx::query_as::<_, (bool,)>(
            r#"
            SELECT is_used FROM qr_payloads WHERE id = $1
            "#,
        )
        .bind(&request.payload.id)
        .fetch_optional(&pool)
        .await
        {
            is_used = used;
        }
    }

    let message = if !is_valid {
        "Invalid signature".to_string()
    } else if is_expired {
        "Payload has expired".to_string()
    } else if is_used {
        "QR code has already been used".to_string()
    } else {
        "Payload is valid and ready to use".to_string()
    };

    let response = VerifyQrResponse {
        valid: is_valid && !is_expired && !is_used,
        expired: is_expired,
        payload: if is_valid {
            Some(request.payload)
        } else {
            None
        },
        message,
    };

    success(response, "Verification complete").into_response()
}

/// Mark a QR payload as used
///
/// # Endpoint
/// POST `/api/v1/qr/mark-used/:id`
///
/// # Response
/// Returns success if the QR code was marked as used
pub async fn mark_qr_used(
    State(pool): State<PgPool>,
    axum::extract::Path(qr_id): axum::extract::Path<String>,
) -> Response {
    // Check if QR exists and is not already used
    let record = match sqlx::query_as::<_, (bool, DateTime<Utc>)>(
        r#"
        SELECT is_used, expires_at FROM qr_payloads WHERE id = $1
        "#,
    )
    .bind(&qr_id)
    .fetch_optional(&pool)
    .await
    {
        Ok(Some((is_used, expires_at))) => (is_used, expires_at),
        Ok(None) => {
            return AppError::NotFound(format!("QR payload with id '{}' not found", qr_id))
                .into_response();
        }
        Err(e) => {
            return AppError::DatabaseError(e).into_response();
        }
    };

    if record.0 {
        return AppError::ValidationError("QR code has already been used".to_string())
            .into_response();
    }

    if record.1 < Utc::now() {
        return AppError::ValidationError("QR code has expired".to_string()).into_response();
    }

    // Mark as used
    if let Err(e) = sqlx::query(
        r#"
        UPDATE qr_payloads SET is_used = true, used_at = $1 WHERE id = $2
        "#,
    )
    .bind(Utc::now())
    .bind(&qr_id)
    .execute(&pool)
    .await
    {
        return AppError::DatabaseError(e).into_response();
    }

    crate::utils::response::empty_success("QR code marked as used").into_response()
}

/// Query parameters for filtering QR payloads
#[derive(Debug, Deserialize)]
pub struct QrPayloadFilters {
    /// Filter by QR type
    pub qr_type: Option<String>,

    /// Filter by usage status
    pub is_used: Option<bool>,

    /// Filter expired QR codes
    pub expired: Option<bool>,
}

/// List QR payloads with pagination and filters
///
/// # Endpoint
/// GET `/api/v1/qr/list`
///
/// # Query Parameters
/// - `page` (optional): Page number (default: 1)
/// - `page_size` (optional): Items per page (default: 20, max: 100)
/// - `qr_type` (optional): Filter by QR type
/// - `is_used` (optional): Filter by usage status
/// - `expired` (optional): Filter expired QR codes
///
/// # Response
/// Returns a paginated list of QR payloads
pub async fn list_qr_payloads(
    State(pool): State<PgPool>,
    Query(pagination): Query<PaginationParams>,
    Query(filters): Query<QrPayloadFilters>,
) -> Response {
    let validated_pagination = pagination.validate();

    // Build WHERE clause
    let mut where_clauses = Vec::new();
    let mut param_count = 0;

    if filters.qr_type.is_some() {
        param_count += 1;
        where_clauses.push(format!("qr_type = ${}", param_count));
    }

    if filters.is_used.is_some() {
        param_count += 1;
        where_clauses.push(format!("is_used = ${}", param_count));
    }

    if let Some(expired) = filters.expired {
        if expired {
            where_clauses.push("expires_at < NOW()".to_string());
        } else {
            where_clauses.push("expires_at >= NOW()".to_string());
        }
    }

    let where_clause = if where_clauses.is_empty() {
        String::new()
    } else {
        format!("WHERE {}", where_clauses.join(" AND "))
    };

    // Count total
    let count_query = format!("SELECT COUNT(*) FROM qr_payloads {}", where_clause);
    let mut count_query_builder = sqlx::query_scalar::<_, i64>(&count_query);

    if let Some(ref qr_type) = filters.qr_type {
        count_query_builder = count_query_builder.bind(qr_type);
    }
    if let Some(is_used) = filters.is_used {
        count_query_builder = count_query_builder.bind(is_used);
    }

    let total = match count_query_builder.fetch_one(&pool).await {
        Ok(count) => count,
        Err(e) => {
            tracing::error!("Failed to count QR payloads: {:?}", e);
            return AppError::DatabaseError(e).into_response();
        }
    };

    // Fetch items
    let items_query = format!(
        "SELECT id, qr_type, payload_data, created_at, expires_at, is_used, used_at FROM qr_payloads {} ORDER BY created_at DESC LIMIT ${} OFFSET ${}",
        where_clause,
        param_count + 1,
        param_count + 2
    );

    let mut items_query_builder = sqlx::query_as::<_, QrPayloadListItem>(&items_query);

    if let Some(ref qr_type) = filters.qr_type {
        items_query_builder = items_query_builder.bind(qr_type);
    }
    if let Some(is_used) = filters.is_used {
        items_query_builder = items_query_builder.bind(is_used);
    }

    items_query_builder = items_query_builder
        .bind(validated_pagination.limit())
        .bind(validated_pagination.offset());

    let items = match items_query_builder.fetch_all(&pool).await {
        Ok(payloads) => payloads,
        Err(e) => {
            tracing::error!("Failed to fetch QR payloads: {:?}", e);
            return AppError::DatabaseError(e).into_response();
        }
    };

    let response = PaginatedResponse::new(items, validated_pagination, total);
    success(response, "QR payloads retrieved successfully").into_response()
}

/// QR payload list item (without sensitive signature/key data)
#[derive(Debug, Serialize, sqlx::FromRow)]
pub struct QrPayloadListItem {
    pub id: String,
    pub qr_type: String,
    pub payload_data: serde_json::Value,
    pub created_at: DateTime<Utc>,
    pub expires_at: DateTime<Utc>,
    pub is_used: bool,
    pub used_at: Option<DateTime<Utc>>,
}

/// DELETE `/api/v1/qr/:id`
///
/// Deletes the QR payload with the given ID.
pub async fn delete_qr_payload(
    State(pool): State<PgPool>,
    axum::extract::Path(qr_id): axum::extract::Path<Uuid>,
) -> Response {
    let id_str = qr_id.to_string();
    match sqlx::query_scalar::<_, String>("DELETE FROM qr_payloads WHERE id = $1 RETURNING id")
        .bind(&id_str)
        .fetch_optional(&pool)
        .await
    {
        Ok(Some(_)) => (axum::http::StatusCode::NO_CONTENT).into_response(),
        Ok(None) => {
            AppError::NotFound(format!("QR payload with id '{}' not found", qr_id)).into_response()
        }
        Err(e) => {
            tracing::error!("Failed to delete QR payload: {:?}", e);
            AppError::DatabaseError(e).into_response()
        }
    }
}

/// Metadata returned per QR code in the per-event listing.
/// Raw signature and public key are intentionally excluded.
#[derive(Debug, Serialize, sqlx::FromRow)]
pub struct EventQrCodeItem {
    pub id: String,
    pub event_id: Option<Uuid>,
    pub ticket_id: Option<Uuid>,
    /// Alias for `is_used` to match the issue spec's `used` field name.
    #[sqlx(rename = "is_used")]
    pub used: bool,
    pub created_at: DateTime<Utc>,
}

/// List QR codes for a specific event (paginated).
///
/// # Endpoint
/// GET `/api/v1/events/:id/qr-codes`
///
/// Returns only metadata (id, event_id, ticket_id, used, created_at).
/// The raw QR payload, signature, and public key are never exposed.
pub async fn list_event_qr_codes(
    State(pool): State<PgPool>,
    Path(event_id): Path<Uuid>,
    Query(pagination): Query<PaginationParams>,
) -> Response {
    // Verify the event exists first.
    let event_exists = match sqlx::query_scalar::<_, bool>(
        "SELECT EXISTS(SELECT 1 FROM events WHERE id = $1)",
    )
    .bind(event_id)
    .fetch_one(&pool)
    .await
    {
        Ok(exists) => exists,
        Err(e) => {
            tracing::error!("Failed to check event existence: {:?}", e);
            return AppError::DatabaseError(e).into_response();
        }
    };

    if !event_exists {
        return AppError::NotFound(format!("Event '{}' not found", event_id)).into_response();
    }

    let validated = pagination.validate();

    let total = match sqlx::query_scalar::<_, i64>(
        "SELECT COUNT(*) FROM qr_payloads WHERE event_id = $1",
    )
    .bind(event_id)
    .fetch_one(&pool)
    .await
    {
        Ok(count) => count,
        Err(e) => {
            tracing::error!("Failed to count QR codes for event: {:?}", e);
            return AppError::DatabaseError(e).into_response();
        }
    };

    let items = match sqlx::query_as::<_, EventQrCodeItem>(
        r"
        SELECT id, event_id, ticket_id, is_used, created_at
        FROM qr_payloads
        WHERE event_id = $1
        ORDER BY created_at DESC
        LIMIT $2 OFFSET $3
        ",
    )
    .bind(event_id)
    .bind(validated.limit())
    .bind(validated.offset())
    .fetch_all(&pool)
    .await
    {
        Ok(rows) => rows,
        Err(e) => {
            tracing::error!("Failed to fetch QR codes for event: {:?}", e);
            return AppError::DatabaseError(e).into_response();
        }
    };

    let response = PaginatedResponse::new(items, validated, total);
    success(response, "QR codes retrieved successfully").into_response()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_qr_payload_serialization() {
        let payload = QrPayload {
            id: "test-id".to_string(),
            qr_type: "ticket".to_string(),
            data: serde_json::json!({"event_id": "123"}),
            created_at: Utc::now(),
            expires_at: Utc::now() + Duration::hours(1),
            nonce: "test-nonce".to_string(),
        };

        let json = serde_json::to_string(&payload).unwrap();
        let deserialized: QrPayload = serde_json::from_str(&json).unwrap();

        assert_eq!(payload.id, deserialized.id);
        assert_eq!(payload.qr_type, deserialized.qr_type);
    }

    #[test]
    fn test_signature_verification() {
        let mut csprng = OsRng;
        let signing_key = SigningKey::generate(&mut csprng);
        let verifying_key = signing_key.verifying_key();

        let message = b"test message";
        let signature = signing_key.sign(message);

        assert!(verifying_key.verify(message, &signature).is_ok());
    }

    #[test]
    fn test_delete_qr_payload_id_parsing() {
        let test_uuid = Uuid::new_v4();
        let uuid_str = test_uuid.to_string();
        let parsed = Uuid::parse_str(&uuid_str).unwrap();
        assert_eq!(test_uuid, parsed);
    }
}
