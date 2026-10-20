//! # Profile Integration Tests
//!
//! Exercises the profile endpoints with Redis cache invalidation:
//!
//! - `PUT /api/v1/profile` — create or update profile with field length validation
//! - `PATCH /api/v1/profile` — partial update with field length validation
//! - `GET /api/v1/profile/:address` — verify cache invalidation after updates
//!
//! ## Running
//! ```bash
//! cargo test --test profile_integration_tests
//! ```
//!
//! The `DATABASE_URL` and `REDIS_URL` environment variables must be set.
//! The schema must already be migrated (run `sqlx migrate run` first).

use axum::{
    body::Body,
    http::{Request, StatusCode},
    routing::{get, post},
    Router,
};
use chrono::Utc;
use serde_json::{json, Value};
use sqlx::{PgPool, Postgres};
use tower::ServiceExt;
use uuid::Uuid;

// Re-export the handlers under test.
use agora_server::handlers::profile::{
    delete_profile, get_my_profile, get_organizer_stats, get_profile_by_address,
    patch_profile, upsert_profile, ProfileState,
};

/// Spin up a minimal router wired to the real test database and Redis.
fn profile_router(pool: PgPool, redis: redis::Client) -> Router {
    let state = ProfileState { pool, redis };
    Router::new()
        .route("/api/v1/profile", post(upsert_profile))
        .route("/api/v1/profile", patch(patch_profile))
        .route("/api/v1/profile", delete(delete_profile))
        .route("/api/v1/profile", get(get_my_profile))
        .route("/api/v1/profile/:address", get(get_profile_by_address))
        .route("/api/v1/profile/:address/stats", get(get_organizer_stats))
        .with_state(state)
}

/// Create a test Redis client connected to the test Redis instance.
///
/// Skips gracefully if REDIS_URL is not set.
async fn test_redis() -> Option<redis::Client> {
    let Ok(redis_url) = std::env::var("REDIS_URL") else {
        eprintln!("skipping profile integration tests: REDIS_URL not set");
        return None;
    };

    match redis::Client::open(redis_url) {
        Ok(client) => Some(client),
        Err(e) => {
            eprintln!("skipping profile integration tests: failed to connect to REDIS_URL: {e}");
            None
        }
    }
}

/// Post a JSON body to `path` and return the response status and body.
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

/// Patch a JSON body to `path` and return the response status and body.
async fn patch_json(router: Router, path: &str, body: Value) -> (StatusCode, Value) {
    let req = Request::builder()
        .method("PATCH")
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

/// Get a JSON response from `path` with optional authorization header.
async fn get_json(router: Router, path: &str, auth_header: Option<&str>) -> (StatusCode, Value) {
    let mut req = Request::builder()
        .method("GET")
        .uri(path)
        .body(Body::empty())
        .unwrap();

    if let Some(auth) = auth_header {
        req.headers_mut().insert(
            "authorization",
            auth.parse().unwrap(),
        );
    }

    let response = router.oneshot(req).await.unwrap();
    let status = response.status();
    let bytes = axum::body::to_bytes(response.into_body(), usize::MAX)
        .await
        .unwrap();
    let json: Value = serde_json::from_slice(&bytes).unwrap_or(Value::Null);
    (status, json)
}

/// Test that PUT /api/v1/profile creates a profile with valid field lengths.
#[tokio::test]
async fn test_put_profile_creates_with_valid_field_lengths() {
    let Ok(database_url) = std::env::var("DATABASE_URL") else {
        eprintln!("skipping test_put_profile_creates_with_valid_field_lengths: DATABASE_URL not set");
        return;
    };

    let pool = PgPool::connect(&database_url)
        .await
        .expect("Failed to connect to test database");

    let redis = test_redis().await;
    let Some(redis_client) = redis else {
        return;
    };

    let wallet_address = "GPUTPROFILE001TESTWALLETADDRESSA";
    let display_name = "Test Organizer 001";
    let bio = "This is a test bio for organizer 001.".to_string();

    // Insert organizer first
    let organizer_id = Uuid::new_v4();
    sqlx::query(
        "INSERT INTO organizers (id, wallet_address, name, contact_email) VALUES ($1, $2, $3, $4)",
    )
    .bind(organizer_id)
    .bind(wallet_address)
    .bind(display_name)
    .bind("test+organizer@agora.test")
    .execute(&pool)
    .await
    .expect("failed to insert organizer");

    // Insert profile directly to avoid auth complexity
    let jwt = create_test_jwt(wallet_address);
    let router = profile_router(pool.clone(), redis_client);

    let (status, body) = post_json(
        router.clone(),
        "/api/v1/profile",
        json!({
            "displayName": display_name,
            "bio": bio,
            "avatarUrl": None::<String>,
            "socials": {}
        }),
    )
    .await;

    assert_eq!(status, StatusCode::OK, "PUT profile response: {body}");
    
    let address = body["data"]["address"].as_str().unwrap();
    assert_eq!(address, wallet_address);
    assert_eq!(body["data"]["displayName"].as_str().unwrap(), display_name);
    assert_eq!(body["data"]["bio"].as_str().unwrap(), bio);
}

/// Test that PUT /api/v1/profile enforces display_name <= 50 characters.
#[tokio::test]
async fn test_put_profile_rejects_display_name_over_50_chars() {
    let Ok(database_url) = std::env::var("DATABASE_URL") else {
        eprintln!("skipping test_put_profile_rejects_display_name_over_50_chars: DATABASE_URL not set");
        return;
    };

    let pool = PgPool::connect(&database_url)
        .await
        .expect("Failed to connect to test database");

    let redis = test_redis().await;
    let Some(redis_client) = redis else {
        return;
    };

    let wallet_address = "GPUTPROFILE002TESTWALLETADDRESSB";
    let long_name = "A".repeat(51);

    // Insert organizer first
    let organizer_id = Uuid::new_v4();
    sqlx::query(
        "INSERT INTO organizers (id, wallet_address, name, contact_email) VALUES ($1, $2, $3, $4)",
    )
    .bind(organizer_id)
    .bind(wallet_address)
    .bind("Test Organizer 002")
    .bind("test+organizer@agora.test")
    .execute(&pool)
    .await
    .expect("failed to insert organizer");

    let router = profile_router(pool.clone(), redis_client);

    let (status, body) = post_json(
        router,
        "/api/v1/profile",
        json!({
            "displayName": long_name,
            "bio": None::<String>,
            "avatarUrl": None::<String>,
            "socials": {}
        }),
    )
    .await;

    assert_eq!(status, StatusCode::BAD_REQUEST, "Expected 400 for long display_name");
    assert_eq!(body["code"].as_str().unwrap(), "VALIDATION_ERROR");
}

/// Test that PUT /api/v1/profile enforces bio <= 500 characters.
#[tokio::test]
async fn test_put_profile_rejects_bio_over_500_chars() {
    let Ok(database_url) = std::env::var("DATABASE_URL") else {
        eprintln!("skipping test_put_profile_rejects_bio_over_500_chars: DATABASE_URL not set");
        return;
    };

    let pool = PgPool::connect(&database_url)
        .await
        .expect("Failed to connect to test database");

    let redis = test_redis().await;
    let Some(redis_client) = redis else {
        return;
    };

    let wallet_address = "GPUTPROFILE003TESTWALLETADDRESSC";
    let long_bio = "B".repeat(501);

    // Insert organizer first
    let organizer_id = Uuid::new_v4();
    sqlx::query(
        "INSERT INTO organizers (id, wallet_address, name, contact_email) VALUES ($1, $2, $3, $4)",
    )
    .bind(organizer_id)
    .bind(wallet_address)
    .bind("Test Organizer 003")
    .bind("test+organizer@agora.test")
    .execute(&pool)
    .await
    .expect("failed to insert organizer");

    let router = profile_router(pool.clone(), redis_client);

    let (status, body) = post_json(
        router,
        json!({
            "displayName": "Valid Name",
            "bio": long_bio,
            "avatarUrl": None::<String>,
            "socials": {}
        }),
    )
    .await;

    assert_eq!(status, StatusCode::BAD_REQUEST, "Expected 400 for long bio");
    assert_eq!(body["code"].as_str().unwrap(), "VALIDATION_ERROR");
}

/// Test that PATCH /api/v1/profile updates bio length validation.
#[tokio::test]
async fn test_patch_profile_validates_bio_length() {
    let Ok(database_url) = std::env::var("DATABASE_URL") else {
        eprintln!("skipping test_patch_profile_validates_bio_length: DATABASE_URL not set");
        return;
    };

    let pool = PgPool::connect(&database_url)
        .await
        .expect("Failed to connect to test database");

    let redis = test_redis().await;
    let Some(redis_client) = redis else {
        return;
    };

    let wallet_address = "GPATCHPROFILE004TESTWALLETADDRESSD";
    let long_bio = "C".repeat(501);

    // Insert organizer first
    let organizer_id = Uuid::new_v4();
    sqlx::query(
        "INSERT INTO organizers (id, wallet_address, name, contact_email) VALUES ($1, $2, $3, $4)",
    )
    .bind(organizer_id)
    .bind(wallet_address)
    .bind("Test Organizer 004")
    .bind("test+organizer@agora.test")
    .execute(&pool)
    .await
    .expect("failed to insert organizer");

    // First create a profile
    let router = profile_router(pool.clone(), redis_client.clone());
    
    post_json(
        router.clone(),
        "/api/v1/profile",
        json!({
            "displayName": "Initial Name",
            "bio": None::<String>,
            "avatarUrl": None::<String>,
            "socials": {}
        }),
    )
    .await;

    // Now try to patch with bio over 500 chars
    let (status, body) = patch_json(
        router,
        "/api/v1/profile",
        json!({
            "bio": long_bio
        }),
    )
    .await;

    assert_eq!(status, StatusCode::BAD_REQUEST, "Expected 400 for long bio in PATCH");
    assert_eq!(body["code"].as_str().unwrap(), "VALIDATION_ERROR");
}

/// Test that Redis cache is invalidated after profile update.
#[tokio::test]
async fn test_redis_cache_invalidation_on_profile_update() {
    let Ok(database_url) = std::env::var("DATABASE_URL") else {
        eprintln!("skipping test_redis_cache_invalidation_on_profile_update: DATABASE_URL not set");
        return;
    };

    let pool = PgPool::connect(&database_url)
        .await
        .expect("Failed to connect to test database");

    let redis = test_redis().await;
    let Some(redis_client) = redis else {
        return;
    };

    let wallet_address = "GCACHEINVALID005TESTWALLETADDRESSE";

    // Insert organizer first
    let organizer_id = Uuid::new_v4();
    sqlx::query(
        "INSERT INTO organizers (id, wallet_address, name, contact_email) VALUES ($1, $2, $3, $4)",
    )
    .bind(organizer_id)
    .bind(wallet_address)
    .bind("Test Organizer 005")
    .bind("test+organizer@agora.test")
    .execute(&pool)
    .await
    .expect("failed to insert organizer");

    let router = profile_router(pool.clone(), redis_client.clone());

    // Create profile
    post_json(
        router.clone(),
        "/api/v1/profile",
        json!({
            "displayName": "Initial Name",
            "bio": None::<String>,
            "avatarUrl": None::<String>,
            "socials": {}
        }),
    )
    .await;

    // Cache key for profile
    let cache_key = format!("profile:{}", wallet_address);
    
    // Verify cache was set after GET (need to call GET first to populate cache)
    let (_status, _) = get_json(
        router.clone(),
        &format!("/api/v1/profile/{}", wallet_address),
        None,
    )
    .await;

    // Verify cache exists
    let mut redis_conn = redis_client.get_connection_manager().await.unwrap();
    let cached: Option<String> = redis_conn.get(&cache_key).await.unwrap();
    assert!(cached.is_some(), "Cache should be populated after GET");

    // Update profile
    let (status, body) = patch_json(
        router,
        "/api/v1/profile",
        json!({
            "displayName": "Updated Name"
        }),
    )
    .await;

    assert_eq!(status, StatusCode::OK, "PATCH response: {body}");

    // Verify cache was invalidated
    let mut redis_conn = redis_client.get_connection_manager().await.unwrap();
    let cached: Option<String> = redis_conn.get(&cache_key).await.unwrap();
    assert!(cached.is_none(), "Cache should be invalidated after PATCH");

    // Verify GET returns updated data
    let (status, body) = get_json(
        router,
        &format!("/api/v1/profile/{}", wallet_address),
        None,
    )
    .await;

    assert_eq!(status, StatusCode::OK);
    assert_eq!(body["data"]["displayName"].as_str().unwrap(), "Updated Name");
}

/// Test that bio exactly at 500 characters is accepted.
#[tokio::test]
async fn test_put_profile_accepts_bio_at_exact_500_chars() {
    let Ok(database_url) = std::env::var("DATABASE_URL") else {
        eprintln!("skipping test_put_profile_accepts_bio_at_exact_500_chars: DATABASE_URL not set");
        return;
    };

    let pool = PgPool::connect(&database_url)
        .await
        .expect("Failed to connect to test database");

    let redis = test_redis().await;
    let Some(redis_client) = redis else {
        return;
    };

    let wallet_address = "GBIO500CHARS006TESTWALLETADDRESSF";
    let bio_at_limit = "B".repeat(500);

    // Insert organizer first
    let organizer_id = Uuid::new_v4();
    sqlx::query(
        "INSERT INTO organizers (id, wallet_address, name, contact_email) VALUES ($1, $2, $3, $4)",
    )
    .bind(organizer_id)
    .bind(wallet_address)
    .bind("Test Organizer 006")
    .bind("test+organizer@agora.test")
    .execute(&pool)
    .await
    .expect("failed to insert organizer");

    let router = profile_router(pool.clone(), redis_client);

    let (status, body) = post_json(
        router,
        json!({
            "displayName": "Valid Name",
            "bio": bio_at_limit,
            "avatarUrl": None::<String>,
            "socials": {}
        }),
    )
    .await;

    assert_eq!(status, StatusCode::OK, "Expected 200 for bio at 500 chars");
    assert_eq!(body["data"]["bio"].as_str().unwrap().len(), 500);
}

/// Test that display_name exactly at 50 characters is accepted.
#[tokio::test]
async fn test_put_profile_accepts_display_name_at_exact_50_chars() {
    let Ok(database_url) = std::env::var("DATABASE_URL") else {
        eprintln!("skipping test_put_profile_accepts_display_name_at_exact_50_chars: DATABASE_URL not set");
        return;
    };

    let pool = PgPool::connect(&database_url)
        .await
        .expect("Failed to connect to test database");

    let redis = test_redis().await;
    let Some(redis_client) = redis else {
        return;
    };

    let wallet_address = "GNAME50CHARS007TESTWALLETADDRESSG";
    let name_at_limit = "A".repeat(50);

    // Insert organizer first
    let organizer_id = Uuid::new_v4();
    sqlx::query(
        "INSERT INTO organizers (id, wallet_address, name, contact_email) VALUES ($1, $2, $3, $4)",
    )
    .bind(organizer_id)
    .bind(wallet_address)
    .bind("Test Organizer 007")
    .bind("test+organizer@agora.test")
    .execute(&pool)
    .await
    .expect("failed to insert organizer");

    let router = profile_router(pool.clone(), redis_client);

    let (status, body) = post_json(
        router,
        json!({
            "displayName": name_at_limit,
            "bio": None::<String>,
            "avatarUrl": None::<String>,
            "socials": {}
        }),
    )
    .await;

    assert_eq!(status, StatusCode::OK, "Expected 200 for display_name at 50 chars");
    assert_eq!(body["data"]["displayName"].as_str().unwrap().len(), 50);
}

/// Test that cache invalidation happens on profile creation (PUT).
#[tokio::test]
async fn test_redis_cache_invalidation_on_profile_creation() {
    let Ok(database_url) = std::env::var("DATABASE_URL") else {
        eprintln!("skipping test_redis_cache_invalidation_on_profile_creation: DATABASE_URL not set");
        return;
    };

    let pool = PgPool::connect(&database_url)
        .await
        .expect("Failed to connect to test database");

    let redis = test_redis().await;
    let Some(redis_client) = redis else {
        return;
    };

    let wallet_address = "GCACHECREATE008TESTWALLETADDRESSH";

    // Insert organizer first
    let organizer_id = Uuid::new_v4();
    sqlx::query(
        "INSERT INTO organizers (id, wallet_address, name, contact_email) VALUES ($1, $2, $3, $4)",
    )
    .bind(organizer_id)
    .bind(wallet_address)
    .bind("Test Organizer 008")
    .bind("test+organizer@agora.test")
    .execute(&pool)
    .await
    .expect("failed to insert organizer");

    let router = profile_router(pool.clone(), redis_client.clone());

    // Cache key for profile
    let cache_key = format!("profile:{}", wallet_address);

    // First GET should populate cache
    let (_status, _) = get_json(
        router.clone(),
        &format!("/api/v1/profile/{}", wallet_address),
        None,
    )
    .await;

    // Verify cache exists
    let mut redis_conn = redis_client.get_connection_manager().await.unwrap();
    let cached: Option<String> = redis_conn.get(&cache_key).await.unwrap();
    assert!(cached.is_some(), "Cache should be populated after GET");

    // Create profile - should invalidate cache
    let (status, _) = post_json(
        router,
        "/api/v1/profile",
        json!({
            "displayName": "Created Name",
            "bio": None::<String>,
            "avatarUrl": None::<String>,
            "socials": {}
        }),
    )
    .await;

    assert_eq!(status, StatusCode::OK);

    // Verify cache was invalidated
    let mut redis_conn = redis_client.get_connection_manager().await.unwrap();
    let cached: Option<String> = redis_conn.get(&cache_key).await.unwrap();
    assert!(cached.is_none(), "Cache should be invalidated after PUT");

    // Clean up
    let _ = sqlx::query("DELETE FROM organizer_profiles WHERE address = $1")
        .bind(wallet_address)
        .execute(&pool)
        .await;
}

/// Test that stats cache is also invalidated on profile update.
#[tokio::test]
async fn test_stats_cache_invalidation_on_profile_update() {
    let Ok(database_url) = std::env::var("DATABASE_URL") else {
        eprintln!("skipping test_stats_cache_invalidation_on_profile_update: DATABASE_URL not set");
        return;
    };

    let pool = PgPool::connect(&database_url)
        .await
        .expect("Failed to connect to test database");

    let redis = test_redis().await;
    let Some(redis_client) = redis else {
        return;
    };

    let wallet_address = "GSTATSCACHEINV009TESTWALLETADDRESSI";

    // Insert organizer first
    let organizer_id = Uuid::new_v4();
    sqlx::query(
        "INSERT INTO organizers (id, wallet_address, name, contact_email) VALUES ($1, $2, $3, $4)",
    )
    .bind(organizer_id)
    .bind(wallet_address)
    .bind("Test Organizer 009")
    .bind("test+organizer@agora.test")
    .execute(&pool)
    .await
    .expect("failed to insert organizer");

    // Insert a test event for stats
    let event_id = Uuid::new_v4();
    sqlx::query(
        "INSERT INTO events (id, organizer_id, title, location, start_time, end_time) VALUES ($1, $2, $3, $4, $5, $6)",
    )
    .bind(event_id)
    .bind(organizer_id)
    .bind("Test Event")
    .bind("Test Location")
    .bind(Utc::now() + chrono::Duration::hours(2))
    .bind(Utc::now() + chrono::Duration::hours(4))
    .execute(&pool)
    .await
    .expect("failed to insert event");

    let router = profile_router(pool.clone(), redis_client.clone());

    // Create profile
    post_json(
        router.clone(),
        "/api/v1/profile",
        json!({
            "displayName": "Initial Name",
            "bio": None::<String>,
            "avatarUrl": None::<String>,
            "socials": {}
        }),
    )
    .await;

    // Cache key for stats
    let stats_cache_key = format!("organizer_stats:{}", wallet_address);

    // First GET stats should populate cache
    let (_status, _) = get_json(
        router.clone(),
        &format!("/api/v1/profile/{}/stats", wallet_address),
        None,
    )
    .await;

    // Verify stats cache exists
    let mut redis_conn = redis_client.get_connection_manager().await.unwrap();
    let cached: Option<String> = redis_conn.get(&stats_cache_key).await.unwrap();
    assert!(cached.is_some(), "Stats cache should be populated after GET");

    // Update profile - should invalidate both profile and stats cache
    let (status, _) = patch_json(
        router,
        "/api/v1/profile",
        json!({
            "displayName": "Updated Name"
        }),
    )
    .await;

    assert_eq!(status, StatusCode::OK);

    // Verify stats cache was invalidated
    let mut redis_conn = redis_client.get_connection_manager().await.unwrap();
    let cached: Option<String> = redis_conn.get(&stats_cache_key).await.unwrap();
    assert!(cached.is_none(), "Stats cache should be invalidated after PATCH");

    // Clean up
    let _ = sqlx::query("DELETE FROM events WHERE id = $1")
        .bind(event_id)
        .execute(&pool)
        .await;
    let _ = sqlx::query("DELETE FROM organizer_profiles WHERE address = $1")
        .bind(wallet_address)
        .execute(&pool)
        .await;
}

/// Helper to create a test JWT for authentication
fn create_test_jwt(wallet_address: &str) -> String {
    // Simple mock JWT format for testing - header.payload.signature
    // This is a simplified approach - in production you'd use proper JWT signing
    let header = base64::encode("eyJhbGciOiJIUzI1NiIsInR5cCI6IkpXVCJ9");
    let payload = base64::encode(format!(
        "{{\"sub\":\"{}\",\"exp\":{}}}",
        wallet_address,
        (Utc::now() + chrono::Duration::hours(1)).timestamp()
    ));
    let signature = base64::encode("mock_signature_for_testing");
    
    format!("{}.{}.{}", header, payload, signature)
}
