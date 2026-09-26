//! # Auth Integration Tests
//!
//! Exercises the complete two-step authentication flow end-to-end against a
//! real PostgreSQL database:
//!
//! 1. `POST /api/v1/auth/nonce` — server inserts a nonce and returns it.
//! 2. `POST /api/v1/auth/verify` — client signs the nonce with a real Ed25519
//!    keypair; server verifies the signature and returns a JWT.
//!
//! The test asserts that the returned JWT contains the correct `sub` claim
//! (the Stellar wallet address).
//!
//! ## Running
//! ```bash
//! cargo test --test auth_integration
//! ```
//!
//! The `DATABASE_URL` and `JWT_SECRET` environment variables must be set.
//! The schema must already be migrated (run `sqlx migrate run` first).

use axum::{
    body::Body,
    http::{Request, StatusCode},
    routing::post,
    Router,
};
use ed25519_dalek::{Signer, SigningKey};
use rand::rngs::OsRng;
use serde_json::Value;
use sqlx::PgPool;
use tower::ServiceExt;

// Re-export the handlers under test.
use agora_server::handlers::auth::{request_nonce, verify_jwt, verify_signature};

/// Spin up a minimal router wired to the real test database.
fn auth_router(pool: PgPool) -> Router {
    Router::new()
        .route("/api/v1/auth/nonce", post(request_nonce))
        .route("/api/v1/auth/verify", post(verify_signature))
        .with_state(pool)
}

/// Post a JSON body to `path` and return the response body as a parsed
/// [`serde_json::Value`].
async fn post_json(router: Router, path: &str, body: Value) -> (StatusCode, Value) {
    let req = Request::builder()
        .method("POST")
        .uri(path)
        .header("content-type", "application/json")
        .body(Body::from(serde_json::to_vec(&body).unwrap()))
        .unwrap();

    let response = router.oneshot(req).await.unwrap();
    let status = response.status();
    let bytes = axum::body::to_bytes(response.into_body(), usize::MAX)
        .await
        .unwrap();
    let json: Value = serde_json::from_slice(&bytes).unwrap_or(Value::Null);
    (status, json)
}

/// Full two-step auth flow integration test.
///
/// Acceptance criteria verified:
/// - `request_nonce` and `verify_signature` are exercised against a real DB.
/// - A real Ed25519 keypair is generated and used to sign the nonce.
/// - The returned JWT contains the correct `sub` claim.
#[tokio::test]
async fn test_full_auth_flow_returns_jwt_with_correct_sub() {
    // -----------------------------------------------------------------------
    // Setup
    // -----------------------------------------------------------------------
    let database_url = std::env::var("DATABASE_URL")
        .expect("DATABASE_URL must be set for integration tests");

    // JWT_SECRET is read inside `jwt_secret()` via env::var; ensure it is set.
    std::env::set_var("JWT_SECRET", "test-secret-do-not-use-in-production");

    let pool = PgPool::connect(&database_url)
        .await
        .expect("Failed to connect to test database");

    // -----------------------------------------------------------------------
    // Generate a real Ed25519 keypair.
    // -----------------------------------------------------------------------
    let mut csprng = OsRng;
    let signing_key = SigningKey::generate(&mut csprng);
    let verifying_key = signing_key.verifying_key();

    // Derive a Stellar-format address from the public key bytes.
    let pk_bytes: [u8; 32] = verifying_key.to_bytes();
    let strkey_pk = stellar_strkey::ed25519::PublicKey(pk_bytes);
    let address = stellar_strkey::Strkey::PublicKeyEd25519(strkey_pk).to_string();

    let router = auth_router(pool.clone());

    // -----------------------------------------------------------------------
    // Step 1: POST /api/v1/auth/nonce
    // -----------------------------------------------------------------------
    let (nonce_status, nonce_body) = post_json(
        auth_router(pool.clone()),
        "/api/v1/auth/nonce",
        serde_json::json!({ "address": address }),
    )
    .await;

    assert_eq!(nonce_status, StatusCode::OK, "nonce response: {nonce_body}");
    let nonce = nonce_body["data"]["nonce"]
        .as_str()
        .expect("response should contain data.nonce")
        .to_string();
    assert!(!nonce.is_empty(), "nonce must not be empty");

    // -----------------------------------------------------------------------
    // Step 2: Sign the nonce and POST /api/v1/auth/verify
    // -----------------------------------------------------------------------
    // The handler verifies `signing_key.sign(nonce.as_bytes())`.
    let signature = signing_key.sign(nonce.as_bytes());
    let sig_hex = hex::encode(signature.to_bytes());
    let pk_hex = hex::encode(pk_bytes);

    let (verify_status, verify_body) = post_json(
        router,
        "/api/v1/auth/verify",
        serde_json::json!({
            "address":    address,
            "nonce":      nonce,
            "signature":  sig_hex,
            "public_key": pk_hex,
        }),
    )
    .await;

    assert_eq!(
        verify_status,
        StatusCode::OK,
        "verify response: {verify_body}"
    );

    let token = verify_body["data"]["token"]
        .as_str()
        .expect("response should contain data.token")
        .to_string();
    assert!(!token.is_empty(), "JWT must not be empty");

    // -----------------------------------------------------------------------
    // Step 3: Decode the JWT and assert the `sub` claim matches the address.
    // -----------------------------------------------------------------------
    let claims = verify_jwt(&token).expect("returned JWT should be valid");
    assert_eq!(
        claims.sub, address,
        "JWT sub claim must match the wallet address"
    );

    // -----------------------------------------------------------------------
    // Cleanup: delete the nonce row so the test is idempotent.
    // -----------------------------------------------------------------------
    let _ = sqlx::query("DELETE FROM jwt_nonces WHERE nonce = $1")
        .bind(&nonce)
        .execute(&pool)
        .await;
}

/// Verifies that the verify endpoint rejects a replayed (already-used) nonce.
#[tokio::test]
async fn test_replay_nonce_is_rejected() {
    let database_url = std::env::var("DATABASE_URL")
        .expect("DATABASE_URL must be set for integration tests");
    std::env::set_var("JWT_SECRET", "test-secret-do-not-use-in-production");

    let pool = PgPool::connect(&database_url)
        .await
        .expect("Failed to connect to test database");

    let mut csprng = OsRng;
    let signing_key = SigningKey::generate(&mut csprng);
    let verifying_key = signing_key.verifying_key();
    let pk_bytes: [u8; 32] = verifying_key.to_bytes();
    let strkey_pk = stellar_strkey::ed25519::PublicKey(pk_bytes);
    let address = stellar_strkey::Strkey::PublicKeyEd25519(strkey_pk).to_string();
    let sig_hex = hex::encode(signing_key.sign(b"placeholder").to_bytes());
    let pk_hex = hex::encode(pk_bytes);

    // Step 1: Get a nonce.
    let (_, nonce_body) = post_json(
        auth_router(pool.clone()),
        "/api/v1/auth/nonce",
        serde_json::json!({ "address": address }),
    )
    .await;
    let nonce = nonce_body["data"]["nonce"]
        .as_str()
        .expect("nonce")
        .to_string();

    // Sign the actual nonce.
    let signature = signing_key.sign(nonce.as_bytes());
    let sig_hex = hex::encode(signature.to_bytes());

    let verify_payload = serde_json::json!({
        "address":    address,
        "nonce":      nonce,
        "signature":  sig_hex,
        "public_key": pk_hex,
    });

    // Step 2: First verify — should succeed.
    let (first_status, _) = post_json(
        auth_router(pool.clone()),
        "/api/v1/auth/verify",
        verify_payload.clone(),
    )
    .await;
    assert_eq!(first_status, StatusCode::OK, "first verify should succeed");

    // Step 3: Replay the same nonce — must be rejected.
    let (replay_status, replay_body) = post_json(
        auth_router(pool.clone()),
        "/api/v1/auth/verify",
        verify_payload,
    )
    .await;
    assert_eq!(
        replay_status,
        StatusCode::UNAUTHORIZED,
        "replay should be rejected; got: {replay_body}"
    );

    // Cleanup.
    let _ = sqlx::query("DELETE FROM jwt_nonces WHERE nonce = $1")
        .bind(&nonce)
        .execute(&pool)
        .await;

    // suppress unused warning for first sig_hex assignment
    let _ = sig_hex;
}
