//! # JWT Authentication Handler
//!
//! Implements a challenge-response authentication flow using Stellar wallet signing.
//!
//! ## Flow
//! 1. `POST /api/v1/auth/nonce` — client sends their wallet address, server returns a random nonce.
//! 2. `POST /api/v1/auth/verify` — client signs the nonce with their Stellar keypair and sends
//!    the signature + public key. Server verifies the signature and issues a 24-hour JWT.
//!
//! ## JWT Claims
//! ```json
//! { "sub": "<stellar_address>", "exp": <unix_ts>, "iat": <unix_ts> }
//! ```
//!
//! Protected handlers extract `user_address` from the token via [`extract_auth`].

use axum::{extract::State, response::IntoResponse, response::Response, Json};
use chrono::{Duration, Utc};
use ed25519_dalek::{Signature, VerifyingKey};
use jsonwebtoken::{decode, encode, Algorithm, DecodingKey, EncodingKey, Header, Validation};
use rand::RngCore;
use serde::{Deserialize, Serialize};
use sqlx::PgPool;
use std::env;

use crate::utils::error::AppError;
use crate::utils::response::{empty_success, success};
use stellar_strkey::Strkey;

// ---------------------------------------------------------------------------
// JWT helpers
// ---------------------------------------------------------------------------

/// JWT claims payload.
#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct Claims {
    /// Stellar wallet address of the authenticated organizer.
    pub sub: String,
    /// Issued-at timestamp (Unix seconds).
    pub iat: i64,
    /// Expiry timestamp (Unix seconds).
    pub exp: i64,
}

pub fn jwt_secret() -> String {
    env::var("JWT_SECRET")
        .expect("JWT_SECRET environment variable is missing. It is required for signing JWTs.")
}

/// Encode a JWT for the given Stellar address with a 24-hour expiry.
pub fn issue_jwt(address: &str) -> Result<String, AppError> {
    let now = Utc::now();
    let claims = Claims {
        sub: address.to_string(),
        iat: now.timestamp(),
        exp: (now + Duration::hours(24)).timestamp(),
    };
    encode(
        &Header::default(),
        &claims,
        &EncodingKey::from_secret(jwt_secret().as_bytes()),
    )
    .map_err(|e| AppError::InternalServerError(format!("Failed to issue JWT: {e}")))
}

/// Decode and validate a JWT, returning the claims on success.
pub fn verify_jwt(token: &str) -> Result<Claims, AppError> {
    let mut validation = Validation::new(Algorithm::HS256);
    validation.validate_exp = true;
    decode::<Claims>(
        token,
        &DecodingKey::from_secret(jwt_secret().as_bytes()),
        &validation,
    )
    .map(|data| data.claims)
    .map_err(|e| AppError::AuthError(format!("Invalid or expired token: {e}")))
}

/// Extract the authenticated wallet address from the `Authorization: Bearer <token>` header.
///
/// Returns `AppError::AuthError` if the header is missing, malformed, or the token is invalid.
pub fn extract_auth(headers: &axum::http::HeaderMap) -> Result<String, AppError> {
    let header = headers
        .get(axum::http::header::AUTHORIZATION)
        .and_then(|v| v.to_str().ok())
        .ok_or_else(|| AppError::AuthError("Missing Authorization header".to_string()))?;

    let token = header.strip_prefix("Bearer ").ok_or_else(|| {
        AppError::AuthError("Authorization header must use Bearer scheme".to_string())
    })?;

    let claims = verify_jwt(token)?;
    Ok(claims.sub)
}

// ---------------------------------------------------------------------------
// Request / response types
// ---------------------------------------------------------------------------

#[derive(Debug, Deserialize)]
pub struct NonceRequest {
    /// Stellar wallet address (G… or C… format).
    pub address: String,
}

#[derive(Debug, Serialize)]
pub struct NonceResponse {
    pub nonce: String,
}

#[derive(Debug, Deserialize)]
pub struct VerifyRequest {
    /// Stellar wallet address that signed the nonce.
    pub address: String,
    /// The nonce that was previously issued for this address.
    pub nonce: String,
    /// Hex-encoded Ed25519 signature of the nonce bytes.
    pub signature: String,
    /// Hex-encoded 32-byte Ed25519 public key corresponding to the address.
    pub public_key: String,
}

#[derive(Debug, Serialize)]
pub struct TokenResponse {
    pub token: String,
}

// ---------------------------------------------------------------------------
// Handlers
// ---------------------------------------------------------------------------

/// `POST /api/v1/auth/nonce`
///
/// Issues a random nonce for the given wallet address. The nonce expires in
/// 5 minutes and is stored in the `jwt_nonces` table.
pub async fn request_nonce(
    State(pool): State<PgPool>,
    Json(payload): Json<NonceRequest>,
) -> Response {
    if payload.address.is_empty() {
        return AppError::ValidationError("address is required".to_string()).into_response();
    }

    // Validate that the address is a valid Stellar public key
    if validate_stellar_address(&payload.address).is_err() {
        return AppError::ValidationError("address must be a valid Stellar public key".to_string())
            .into_response();
    }

    // Generate a 32-byte random nonce encoded as hex
    let mut bytes = [0u8; 32];
    rand::thread_rng().fill_bytes(&mut bytes);
    let nonce = hex::encode(bytes);

    match sqlx::query("INSERT INTO jwt_nonces (nonce, address) VALUES ($1, $2)")
        .bind(&nonce)
        .bind(&payload.address)
        .execute(&pool)
        .await
    {
        Ok(_) => success(NonceResponse { nonce }, "Nonce issued").into_response(),
        Err(e) => {
            tracing::error!("Failed to store nonce: {:?}", e);
            AppError::DatabaseError(e).into_response()
        }
    }
}

/// Validates that a string is a valid Stellar public key address.
///
/// A valid Stellar address:
/// - Starts with 'G'
/// - Is 56 characters long
/// - Contains only valid base32 characters
fn validate_stellar_address(address: &str) -> Result<(), String> {
    // Check basic format
    if !address.starts_with('G') || address.len() != 56 {
        return Err("Address must start with G and be 56 characters long".to_string());
    }

    // Try to decode as Stellar strkey
    match Strkey::from_string(address) {
        Ok(Strkey::PublicKeyEd25519(_)) => Ok(()),
        _ => Err("Invalid Stellar public key format".to_string()),
    }
}

/// `POST /api/v1/auth/verify`
///
/// Verifies the Ed25519 signature of the nonce and issues a 24-hour JWT.
///
/// The client must sign the raw nonce bytes (not a hash) with the private key
/// corresponding to the provided public key.
pub async fn verify_signature(
    State(pool): State<PgPool>,
    Json(payload): Json<VerifyRequest>,
) -> Response {
    if payload.address.is_empty()
        || payload.nonce.is_empty()
        || payload.signature.is_empty()
        || payload.public_key.is_empty()
    {
        return AppError::ValidationError(
            "address, nonce, signature, and public_key are all required".to_string(),
        )
        .into_response();
    }

    // 1. Look up the nonce — must exist, be unused, and not expired
    struct NonceRow {
        id: uuid::Uuid,
        used: bool,
        expires_at: chrono::DateTime<Utc>,
    }

    let row = match sqlx::query_as::<_, (uuid::Uuid, bool, chrono::DateTime<Utc>)>(
        "SELECT id, used, expires_at FROM jwt_nonces WHERE nonce = $1 AND address = $2",
    )
    .bind(&payload.nonce)
    .bind(&payload.address)
    .fetch_optional(&pool)
    .await
    {
        Ok(Some(r)) => NonceRow {
            id: r.0,
            used: r.1,
            expires_at: r.2,
        },
        Ok(None) => {
            return AppError::AuthError("Nonce not found or address mismatch".to_string())
                .into_response();
        }
        Err(e) => {
            tracing::error!("Failed to fetch nonce: {:?}", e);
            return AppError::DatabaseError(e).into_response();
        }
    };

    if row.used {
        return AppError::AuthError("Nonce has already been used".to_string()).into_response();
    }

    if row.expires_at < Utc::now() {
        return AppError::AuthError("Nonce has expired".to_string()).into_response();
    }

    // 2. Verify the Ed25519 signature
    let sig_bytes = match hex::decode(&payload.signature) {
        Ok(b) => b,
        Err(_) => {
            return AppError::ValidationError("signature must be valid hex".to_string())
                .into_response();
        }
    };

    let pk_bytes = match hex::decode(&payload.public_key) {
        Ok(b) => b,
        Err(_) => {
            return AppError::ValidationError("public_key must be valid hex".to_string())
                .into_response();
        }
    };

    let pk_array: [u8; 32] = match pk_bytes.as_slice().try_into() {
        Ok(arr) => arr,
        Err(_) => {
            return AppError::ValidationError("public_key must be 32 bytes".to_string())
                .into_response();
        }
    };

    let strkey_pk = stellar_strkey::ed25519::PublicKey(pk_array);
    let strkey = stellar_strkey::Strkey::PublicKeyEd25519(strkey_pk);
    if strkey.to_string().as_str() != payload.address {
        return AppError::AuthError("Public key does not match address".to_string())
            .into_response();
    }

    let verifying_key = match pk_bytes
        .as_slice()
        .try_into()
        .ok()
        .and_then(|b: [u8; 32]| VerifyingKey::from_bytes(&b).ok())
    {
        Some(k) => k,
        None => {
            return AppError::ValidationError(
                "public_key must be a 32-byte Ed25519 key".to_string(),
            )
            .into_response();
        }
    };

    let signature = match sig_bytes
        .as_slice()
        .try_into()
        .ok()
        .map(|b: [u8; 64]| Signature::from_bytes(&b))
    {
        Some(s) => s,
        None => {
            return AppError::ValidationError("signature must be 64 bytes".to_string())
                .into_response();
        }
    };

    use ed25519_dalek::Verifier;
    if verifying_key
        .verify(payload.nonce.as_bytes(), &signature)
        .is_err()
    {
        return AppError::AuthError("Signature verification failed".to_string()).into_response();
    }

    // 3. Mark nonce as used (single-use)
    if let Err(e) = sqlx::query("UPDATE jwt_nonces SET used = TRUE WHERE id = $1")
        .bind(row.id)
        .execute(&pool)
        .await
    {
        tracing::error!("Failed to mark nonce as used: {:?}", e);
        return AppError::DatabaseError(e).into_response();
    }

    // 4. Issue JWT
    let token = match issue_jwt(&payload.address) {
        Ok(t) => t,
        Err(e) => return e.into_response(),
    };

    let mut response = success(
        TokenResponse {
            token: token.clone(),
        },
        "Authentication successful",
    )
    .into_response();

    // Generate CSRF token
    let mut csrf_bytes = [0u8; 32];
    rand::thread_rng().fill_bytes(&mut csrf_bytes);
    let csrf_token = hex::encode(csrf_bytes);

    let cookie = format!("XSRF-TOKEN={}; Path=/; HttpOnly; SameSite=Lax", csrf_token);
    response
        .headers_mut()
        .insert(axum::http::header::SET_COOKIE, cookie.parse().unwrap());

    response
}

/// `POST /api/v1/auth/logout`
///
/// Stateless logout — instructs the client to discard the token.
/// Since JWTs are stateless, server-side invalidation is not performed here.
pub async fn logout() -> Response {
    empty_success("Logged out successfully").into_response()
}

// ---------------------------------------------------------------------------
// Tests
// ---------------------------------------------------------------------------

#[cfg(test)]
mod tests {
    use super::*;

    /// `jwt_secret()` reads `JWT_SECRET` from the process environment and
    /// panics if it's unset — correct for a real server, but tests need to
    /// set it themselves since nothing else in `cargo test --lib` does.
    fn ensure_test_jwt_secret() {
        std::env::set_var("JWT_SECRET", "test-secret-for-unit-tests-only-32b");
    }

    #[test]
    fn test_issue_and_verify_jwt() {
        ensure_test_jwt_secret();
        let address = "GABC123XYZ";
        let token = issue_jwt(address).expect("should issue JWT");
        let claims = verify_jwt(&token).expect("should verify JWT");
        assert_eq!(claims.sub, address);
    }

    #[test]
    fn test_verify_invalid_jwt() {
        ensure_test_jwt_secret();
        let result = verify_jwt("not.a.valid.token");
        assert!(result.is_err());
    }

    #[test]
    fn test_extract_auth_missing_header() {
        let headers = axum::http::HeaderMap::new();
        let result = extract_auth(&headers);
        assert!(result.is_err());
    }

    #[test]
    fn test_extract_auth_valid_token() {
        ensure_test_jwt_secret();
        let address = "GTEST456";
        let token = issue_jwt(address).unwrap();
        let mut headers = axum::http::HeaderMap::new();
        headers.insert(
            axum::http::header::AUTHORIZATION,
            format!("Bearer {token}").parse().unwrap(),
        );
        let extracted = extract_auth(&headers).unwrap();
        assert_eq!(extracted, address);
    }

    #[test]
    fn test_extract_auth_wrong_scheme() {
        let mut headers = axum::http::HeaderMap::new();
        headers.insert(
            axum::http::header::AUTHORIZATION,
            "Basic sometoken".parse().unwrap(),
        );
        let result = extract_auth(&headers);
        assert!(result.is_err());
    }

    #[test]
    fn test_validate_stellar_address_valid() {
        // Compute a real, correctly-checksummed Stellar public key rather
        // than hand-typing one — a single wrong character silently produces
        // an invalid strkey (wrong CRC16), which is exactly what this test
        // is supposed to catch, not accidentally exercise.
        let strkey_pk = Strkey::PublicKeyEd25519(stellar_strkey::ed25519::PublicKey([0u8; 32]));
        // `Strkey` has an inherent `to_string()` returning a `heapless::String`
        // (shadowing the std `ToString` blanket impl) — go through `format!`
        // to force a real `std::String` via the `Display` impl instead.
        let address = format!("{}", strkey_pk);
        assert!(validate_stellar_address(&address).is_ok());
    }

    #[test]
    fn test_validate_stellar_address_invalid_not_starting_with_g() {
        let address = "CAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAYMY";
        let result = validate_stellar_address(address);
        assert!(result.is_err());
    }

    #[test]
    fn test_validate_stellar_address_wrong_length() {
        let address = "GABC123";
        let result = validate_stellar_address(address);
        assert!(result.is_err());
    }

    #[test]
    fn test_validate_stellar_address_completely_invalid() {
        let address = "not-an-address";
        let result = validate_stellar_address(address);
        assert!(result.is_err());
    }

    #[test]
    fn test_validate_stellar_address_empty() {
        let address = "";
        let result = validate_stellar_address(address);
        assert!(result.is_err());
    }
}
