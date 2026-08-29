//! Developer API keys for organisers (Issue #1340)

use axum::{
    extract::{Path, State},
    http::HeaderMap,
    response::{IntoResponse, Response},
    Json,
};
use rand::{distributions::Alphanumeric, Rng};
use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};
use sqlx::PgPool;
use uuid::Uuid;

use crate::handlers::auth::extract_auth;
use crate::utils::error::AppError;
use crate::utils::response::success;

/// Application state for API key handlers.
#[derive(Clone)]
pub struct ApiKeysState {
    pub pool: PgPool,
}

#[derive(Debug, Deserialize)]
pub struct CreateApiKeyRequest {
    pub name: String,
}

#[derive(Debug, Serialize, Clone, sqlx::FromRow)]
pub struct ApiKeyRow {
    pub id: Uuid,
    pub organizer_id: String,
    pub key_hash: String,
    pub prefix: String,
    pub name: String,
    pub created_at: chrono::DateTime<chrono::Utc>,
    pub last_used_at: Option<chrono::DateTime<chrono::Utc>>,
    pub is_active: bool,
}

#[derive(Debug, Serialize)]
pub struct ApiKeyListItem {
    pub id: Uuid,
    pub prefix: String,
    pub name: String,
    pub created_at: chrono::DateTime<chrono::Utc>,
    pub last_used_at: Option<chrono::DateTime<chrono::Utc>>,
    pub is_active: bool,
}

#[derive(Debug, Serialize)]
pub struct CreateApiKeyResponse {
    pub id: Uuid,
    pub key: String,
    pub prefix: String,
    pub name: String,
    pub created_at: chrono::DateTime<chrono::Utc>,
}

fn api_key_prefix() -> String {
    format!("{}{}", "sk_", "live_")
}

/// Generate a key of format prefix + [32 random alphanumeric chars]
pub fn generate_api_key() -> String {
    let rand_part: String = rand::thread_rng()
        .sample_iter(&Alphanumeric)
        .take(32)
        .map(char::from)
        .collect();
    format!("{}{}", api_key_prefix(), rand_part)
}

pub fn hash_api_key(key: &str) -> String {
    let mut hasher = Sha256::new();
    hasher.update(key.as_bytes());
    hex::encode(hasher.finalize())
}

pub fn prefix_of(key: &str) -> String {
    key.chars().take(8).collect()
}

/// POST /api/v1/settings/api-keys
pub async fn create_api_key(
    State(state): State<ApiKeysState>,
    headers: HeaderMap,
    Json(payload): Json<CreateApiKeyRequest>,
) -> Response {
    let address = match extract_auth(&headers) {
        Ok(a) => a,
        Err(e) => return e.into_response(),
    };

    if payload.name.trim().is_empty() {
        return AppError::ValidationError("name is required".to_string()).into_response();
    }
    if payload.name.len() > 100 {
        return AppError::ValidationError("name must not exceed 100 characters".to_string())
            .into_response();
    }

    let plaintext = generate_api_key();
    let key_hash = hash_api_key(&plaintext);
    let prefix = prefix_of(&plaintext);

    let row = match sqlx::query_as::<_, ApiKeyRow>(
        r#"INSERT INTO developer_api_keys (organizer_id, key_hash, prefix, name)
           VALUES ($1, $2, $3, $4) RETURNING *"#,
    )
    .bind(&address)
    .bind(&key_hash)
    .bind(&prefix)
    .bind(payload.name.trim())
    .fetch_one(&state.pool)
    .await
    {
        Ok(r) => r,
        Err(e) => {
            tracing::error!("Failed to create api key: {:?}", e);
            return AppError::DatabaseError(e).into_response();
        }
    };

    let resp = CreateApiKeyResponse {
        id: row.id,
        key: plaintext,
        prefix: row.prefix,
        name: row.name,
        created_at: row.created_at,
    };
    success(resp, "API key created").into_response()
}

/// GET /api/v1/settings/api-keys
pub async fn list_api_keys(
    State(state): State<ApiKeysState>,
    headers: HeaderMap,
) -> Response {
    let address = match extract_auth(&headers) {
        Ok(a) => a,
        Err(e) => return e.into_response(),
    };

    let rows = match sqlx::query_as::<_, ApiKeyRow>(
        r#"SELECT * FROM developer_api_keys WHERE organizer_id = $1 AND is_active = TRUE ORDER BY created_at DESC"#,
    )
    .bind(&address)
    .fetch_all(&state.pool)
    .await
    {
        Ok(r) => r,
        Err(e) => {
            tracing::error!("Failed to list api keys: {:?}", e);
            return AppError::DatabaseError(e).into_response();
        }
    };

    let items: Vec<ApiKeyListItem> = rows
        .into_iter()
        .map(|r| ApiKeyListItem {
            id: r.id,
            prefix: r.prefix,
            name: r.name,
            created_at: r.created_at,
            last_used_at: r.last_used_at,
            is_active: r.is_active,
        })
        .collect();

    success(items, "API keys retrieved").into_response()
}

/// DELETE /api/v1/settings/api-keys/:id
pub async fn revoke_api_key(
    State(state): State<ApiKeysState>,
    headers: HeaderMap,
    Path(id): Path<Uuid>,
) -> Response {
    let address = match extract_auth(&headers) {
        Ok(a) => a,
        Err(e) => return e.into_response(),
    };

    // Soft-revoke: set is_active = false and ensure it belongs to requester
    let result = match sqlx::query(
        r#"UPDATE developer_api_keys SET is_active = FALSE WHERE id = $1 AND organizer_id = $2 AND is_active = TRUE"#,
    )
    .bind(id)
    .bind(&address)
    .execute(&state.pool)
    .await
    {
        Ok(r) => r,
        Err(e) => {
            tracing::error!("Failed to revoke api key: {:?}", e);
            return AppError::DatabaseError(e).into_response();
        }
    };

    if result.rows_affected() == 0 {
        return AppError::NotFound(format!("API key {id} not found")).into_response();
    }

    success(serde_json::json!({}), "API key revoked").into_response()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_generate_api_key_format() {
        let prefix = api_key_prefix();
        let key = generate_api_key();
        assert!(key.starts_with(&prefix));
        assert_eq!(key.len(), prefix.len() + 32);
        let suffix = &key[prefix.len()..];
        assert!(suffix.chars().all(|c| c.is_alphanumeric()));
    }

    #[test]
    fn test_hash_is_deterministic_and_hex() {
        let key = format!("{}abc123", api_key_prefix());
        let h1 = hash_api_key(&key);
        let h2 = hash_api_key(&key);
        assert_eq!(h1, h2);
        assert_eq!(h1.len(), 64); // sha256 hex
        assert!(h1.chars().all(|c| c.is_ascii_hexdigit()));
    }

    #[test]
    fn test_prefix_is_first_8_chars() {
        let key = format!("{}ABCDEFGHIJ", api_key_prefix());
        let prefix = api_key_prefix();
        assert_eq!(prefix_of(&key), prefix);
        assert_eq!(prefix_of(&format!("{}1234567890", prefix)).len(), 8);
    }

    #[test]
    fn test_generate_unique_keys() {
        let k1 = generate_api_key();
        let k2 = generate_api_key();
        assert_ne!(k1, k2);
        assert_ne!(hash_api_key(&k1), hash_api_key(&k2));
    }
}
