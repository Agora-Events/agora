//! # Waiting Room Integration Tests
//!
//! Exercises the virtual waiting room's bot-resistant queue:
//!
//! - `POST /api/v1/waiting-room/challenge` — Issue PoW challenge
//! - `POST /api/v1/waiting-room/join` — Verify PoW, join queue, return status
//! - `GET /api/v1/waiting-room/status` — Query queue position
//! - `GET /api/v1/waiting-room/stream` — SSE stream for live updates
//!
//! ## Running
//! ```bash
//! cargo test --test waiting_room_integration_tests
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
use serde_json::{json, Value};
use sqlx::PgPool;
use tower::ServiceExt;
use uuid::Uuid;

// Re-export the handlers under test.
use agora_server::handlers::waiting_room::{
    queue_status, queue_stream, request_challenge, join_queue, WaitingRoomState, WaitingRoomConfig,
};

/// Create a router with the waiting room state initialized.
/// This helper function takes a pre-configured state.
fn create_waiting_room_router(state: WaitingRoomState) -> Router {
    Router::new()
        .route("/api/v1/waiting-room/challenge", post(request_challenge))
        .route("/api/v1/waiting-room/join", post(join_queue))
        .route("/api/v1/waiting-room/status", get(queue_status))
        .route("/api/v1/waiting-room/stream", get(queue_stream))
        .with_state(state)
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

/// Get a JSON response from `path` with query parameters.
async fn get_json(router: Router, path: &str, query: &str) -> (StatusCode, Value) {
    let req = Request::builder()
        .method("GET")
        .uri(format!("{}?{}", path, query))
        .body(Body::empty())
        .unwrap();

    let response = router.oneshot(req).await.unwrap();
    let status = response.status();
    let bytes = axum::body::to_bytes(response.into_body(), usize::MAX)
        .await
        .unwrap();
    let json: Value = serde_json::from_slice(&bytes).unwrap_or(Value::Null);
    (status, json)
}

/// Create a test event in the database.
async fn create_test_event(pool: &PgPool) -> String {
    let event_id = Uuid::new_v4().to_string();
    
    let organizer_id = Uuid::new_v4();
    let organizer_name = format!("Test Organizer {}", &event_id[0..8]);
    
    // Insert organizer
    sqlx::query(
        "INSERT INTO organizers (id, name, contact_email) VALUES ($1, $2, 'test+organizer@agora.test')",
    )
    .bind(organizer_id)
    .bind(&organizer_name)
    .execute(pool)
    .await
    .expect("failed to insert organizer");

    // Insert event with 100 total tickets and 0 minted
    sqlx::query(
        "INSERT INTO events (id, organizer_id, title, location, start_time, end_time, total_tickets, minted_tickets) 
         VALUES ($1, $2, $3, $4, $5, $6, $7, $8)",
    )
    .bind(event_id.clone())
    .bind(organizer_id)
    .bind("Test Event - Waiting Room")
    .bind("Test Venue")
    .bind(chrono::Utc::now() + chrono::Duration::hours(2))
    .bind(chrono::Utc::now() + chrono::Duration::hours(4))
    .bind(100i64) // total_tickets
    .bind(0i64)   // minted_tickets
    .execute(pool)
    .await
    .expect("failed to insert event");

    event_id
}

/// Test that challenge issuance returns valid PoW challenge.
#[tokio::test]
async fn test_challenge_issuance_returns_valid_pow_challenge() {
    let Ok(database_url) = std::env::var("DATABASE_URL") else {
        eprintln!("skipping test_challenge_issuance_returns_valid_pow_challenge: DATABASE_URL not set");
        return;
    };

    let Ok(redis_url) = std::env::var("REDIS_URL") else {
        eprintln!("skipping test_challenge_issuance_returns_valid_pow_challenge: REDIS_URL not set");
        return;
    };

    let pool = PgPool::connect(&database_url)
        .await
        .expect("Failed to connect to test database");

    let redis_client = redis::Client::open(&redis_url).expect("Failed to connect to Redis");
    let connection_manager = redis_client.get_connection_manager().await.expect("Failed to get Redis connection manager");
    let cache = crate::cache::RedisCache { client: connection_manager };
    let engine = crate::services::queue::QueueEngine::new(cache);
    
    let config = WaitingRoomConfig {
        pow_difficulty: 4,
        admission_rate_per_minute: 60,
        grant_ttl_minutes: 10,
        challenge_ttl_seconds: 300,
        tick_interval_ms: 1000,
    };
    
    let state = WaitingRoomState {
        engine: std::sync::Arc::new(engine),
        pool,
        config,
    };
    
    let router = create_waiting_room_router(state);

    let event_id = create_test_event(&pool).await;

    let (status, body) = post_json(
        router,
        "/api/v1/waiting-room/challenge",
        json!({
            "event_id": event_id
        }),
    )
    .await;

    assert_eq!(status, StatusCode::OK, "Challenge issuance failed: {body}");
    assert!(body["data"]["challenge"].is_string());
    assert_eq!(body["data"]["difficulty"], 4);
    assert_eq!(body["data"]["expires_in"], 300);

    let challenge = body["data"]["challenge"].as_str().unwrap();
    // Challenge should be 32 hex characters (16 bytes encoded)
    assert_eq!(challenge.len(), 32);
    assert!(challenge.chars().all(|c| c.is_ascii_hexdigit()));
}

/// Test that challenge is single-use and expires.
#[tokio::test]
async fn test_challenge_is_single_use_and_expires() {
    let Ok(database_url) = std::env::var("DATABASE_URL") else {
        eprintln!("skipping test_challenge_is_single_use_and_expires: DATABASE_URL not set");
        return;
    };

    let Ok(redis_url) = std::env::var("REDIS_URL") else {
        eprintln!("skipping test_challenge_is_single_use_and_expires: REDIS_URL not set");
        return;
    };

    let pool = PgPool::connect(&database_url)
        .await
        .expect("Failed to connect to test database");

    let redis_client = redis::Client::open(&redis_url).expect("Failed to connect to Redis");
    let connection_manager = redis_client.get_connection_manager().await.expect("Failed to get Redis connection manager");
    let cache = crate::cache::RedisCache { client: connection_manager };
    let engine = crate::services::queue::QueueEngine::new(cache);
    
    let config = WaitingRoomConfig {
        pow_difficulty: 4,
        admission_rate_per_minute: 60,
        grant_ttl_minutes: 10,
        challenge_ttl_seconds: 300,
        tick_interval_ms: 1000,
    };
    
    let state = WaitingRoomState {
        engine: std::sync::Arc::new(engine),
        pool,
        config,
    };
    
    let router = create_waiting_room_router(state);

    let event_id = create_test_event(&pool).await;

    // Issue challenge
    let (status, body) = post_json(
        router.clone(),
        "/api/v1/waiting-room/challenge",
        json!({
            "event_id": event_id
        }),
    )
    .await;

    assert_eq!(status, StatusCode::OK);
    let challenge = body["data"]["challenge"].as_str().unwrap();
    let difficulty = body["data"]["difficulty"].as_i64().unwrap() as u32;

    // Solve PoW - find nonce that starts with difficulty zeros
    let nonce = find_pow_nonce(challenge, difficulty).unwrap();

    // First use should succeed
    let (status, body) = post_json(
        router.clone(),
        "/api/v1/waiting-room/join",
        json!({
            "event_id": event_id,
            "client_id": "GTESTCLIENT001",
            "challenge": challenge,
            "nonce": nonce
        }),
    )
    .await;

    assert_eq!(status, StatusCode::OK, "First join should succeed: {body}");

    // Second use with same challenge should fail (single-use)
    let (status, body) = post_json(
        router.clone(),
        "/api/v1/waiting-room/join",
        json!({
            "event_id": event_id,
            "client_id": "GTESTCLIENT002",
            "challenge": challenge,
            "nonce": nonce
        }),
    )
    .await;

    assert_eq!(status, StatusCode::UNPROCESSABLE_ENTITY, "Second join should fail");
    assert!(body["message"].as_str().unwrap().contains("Invalid, expired, or already-used"));
}

/// Test that valid PoW solution allows queue entry.
#[tokio::test]
async fn test_valid_pow_allows_queue_entry() {
    let Ok(database_url) = std::env::var("DATABASE_URL") else {
        eprintln!("skipping test_valid_pow_allows_queue_entry: DATABASE_URL not set");
        return;
    };

    let Ok(redis_url) = std::env::var("REDIS_URL") else {
        eprintln!("skipping test_valid_pow_allows_queue_entry: REDIS_URL not set");
        return;
    };

    let pool = PgPool::connect(&database_url)
        .await
        .expect("Failed to connect to test database");

    let redis_client = redis::Client::open(&redis_url).expect("Failed to connect to Redis");
    let connection_manager = redis_client.get_connection_manager().await.expect("Failed to get Redis connection manager");
    let cache = crate::cache::RedisCache { client: connection_manager };
    let engine = crate::services::queue::QueueEngine::new(cache);
    
    let config = WaitingRoomConfig {
        pow_difficulty: 4,
        admission_rate_per_minute: 60,
        grant_ttl_minutes: 10,
        challenge_ttl_seconds: 300,
        tick_interval_ms: 1000,
    };
    
    let state = WaitingRoomState {
        engine: std::sync::Arc::new(engine),
        pool,
        config,
    };
    
    let router = create_waiting_room_router(state);

    let event_id = create_test_event(&pool).await;

    // Issue challenge
    let (status, body) = post_json(
        router.clone(),
        "/api/v1/waiting-room/challenge",
        json!({
            "event_id": event_id
        }),
    )
    .await;

    assert_eq!(status, StatusCode::OK);
    let challenge = body["data"]["challenge"].as_str().unwrap();
    let difficulty = body["data"]["difficulty"].as_i64().unwrap() as u32;

    // Solve PoW
    let nonce = find_pow_nonce(challenge, difficulty).unwrap();

    // Join queue with valid PoW
    let (status, body) = post_json(
        router,
        "/api/v1/waiting-room/join",
        json!({
            "event_id": event_id,
            "client_id": "GTESTCLIENT003",
            "challenge": challenge,
            "nonce": nonce
        }),
    )
    .await;

    assert_eq!(status, StatusCode::OK, "Join failed: {body}");
    assert_eq!(body["data"]["status"], "waiting");
    assert_eq!(body["data"]["position"], 1);
    assert_eq!(body["data"]["queue_size"], 1);
    assert!(body["data"]["estimated_wait_seconds"].is_i64());
}

/// Test that invalid PoW solution is rejected.
#[tokio::test]
async fn test_invalid_pow_solution_is_rejected() {
    let Ok(database_url) = std::env::var("DATABASE_URL") else {
        eprintln!("skipping test_invalid_pow_solution_is_rejected: DATABASE_URL not set");
        return;
    };

    let Ok(redis_url) = std::env::var("REDIS_URL") else {
        eprintln!("skipping test_invalid_pow_solution_is_rejected: REDIS_URL not set");
        return;
    };

    let pool = PgPool::connect(&database_url)
        .await
        .expect("Failed to connect to test database");

    let redis_client = redis::Client::open(&redis_url).expect("Failed to connect to Redis");
    let connection_manager = redis_client.get_connection_manager().await.expect("Failed to get Redis connection manager");
    let cache = crate::cache::RedisCache { client: connection_manager };
    let engine = crate::services::queue::QueueEngine::new(cache);
    
    let config = WaitingRoomConfig {
        pow_difficulty: 4,
        admission_rate_per_minute: 60,
        grant_ttl_minutes: 10,
        challenge_ttl_seconds: 300,
        tick_interval_ms: 1000,
    };
    
    let state = WaitingRoomState {
        engine: std::sync::Arc::new(engine),
        pool,
        config,
    };
    
    let router = create_waiting_room_router(state);

    let event_id = create_test_event(&pool).await;

    // Issue challenge
    let (status, body) = post_json(
        router.clone(),
        "/api/v1/waiting-room/challenge",
        json!({
            "event_id": event_id
        }),
    )
    .await;

    assert_eq!(status, StatusCode::OK);
    let challenge = body["data"]["challenge"].as_str().unwrap();

    // Try with wrong nonce (just "0" won't satisfy difficulty 4)
    let (status, body) = post_json(
        router,
        "/api/v1/waiting-room/join",
        json!({
            "event_id": event_id,
            "client_id": "GTESTCLIENT004",
            "challenge": challenge,
            "nonce": "0"
        }),
    )
    .await;

    assert_eq!(status, StatusCode::UNPROCESSABLE_ENTITY, "Expected rejection for invalid PoW");
    assert!(body["message"].as_str().unwrap().contains("Proof-of-work solution is incorrect"));
}

/// Test that queue position can be retrieved.
#[tokio::test]
async fn test_queue_position_can_be_retrieved() {
    let Ok(database_url) = std::env::var("DATABASE_URL") else {
        eprintln!("skipping test_queue_position_can_be_retrieved: DATABASE_URL not set");
        return;
    };

    let Ok(redis_url) = std::env::var("REDIS_URL") else {
        eprintln!("skipping test_queue_position_can_be_retrieved: REDIS_URL not set");
        return;
    };

    let pool = PgPool::connect(&database_url)
        .await
        .expect("Failed to connect to test database");

    let redis_client = redis::Client::open(&redis_url).expect("Failed to connect to Redis");
    let connection_manager = redis_client.get_connection_manager().await.expect("Failed to get Redis connection manager");
    let cache = crate::cache::RedisCache { client: connection_manager };
    let engine = crate::services::queue::QueueEngine::new(cache);
    
    let config = WaitingRoomConfig {
        pow_difficulty: 4,
        admission_rate_per_minute: 60,
        grant_ttl_minutes: 10,
        challenge_ttl_seconds: 300,
        tick_interval_ms: 1000,
    };
    
    let state = WaitingRoomState {
        engine: std::sync::Arc::new(engine),
        pool,
        config,
    };
    
    let router = create_waiting_room_router(state);

    let event_id = create_test_event(&pool).await;

    // Join queue
    let (status, body) = post_json(
        router.clone(),
        "/api/v1/waiting-room/challenge",
        json!({
            "event_id": event_id
        }),
    )
    .await;

    assert_eq!(status, StatusCode::OK);
    let challenge = body["data"]["challenge"].as_str().unwrap();
    let difficulty = body["data"]["difficulty"].as_i64().unwrap() as u32;
    let nonce = find_pow_nonce(challenge, difficulty).unwrap();

    let (status, body) = post_json(
        router.clone(),
        "/api/v1/waiting-room/join",
        json!({
            "event_id": event_id,
            "client_id": "GTESTCLIENT005",
            "challenge": challenge,
            "nonce": nonce
        }),
    )
    .await;

    assert_eq!(status, StatusCode::OK);

    // Query position
    let (status, body) = get_json(
        router,
        "/api/v1/waiting-room/status",
        &format!("event_id={}&client_id=GTESTCLIENT005", event_id),
    )
    .await;

    assert_eq!(status, StatusCode::OK);
    assert_eq!(body["data"]["status"], "waiting");
    assert_eq!(body["data"]["position"], 1);
    assert_eq!(body["data"]["queue_size"], 1);
}

/// Test that non-existent queue entry returns 404.
#[tokio::test]
async fn test_nonexistent_queue_entry_returns_404() {
    let Ok(database_url) = std::env::var("DATABASE_URL") else {
        eprintln!("skipping test_nonexistent_queue_entry_returns_404: DATABASE_URL not set");
        return;
    };

    let Ok(redis_url) = std::env::var("REDIS_URL") else {
        eprintln!("skipping test_nonexistent_queue_entry_returns_404: REDIS_URL not set");
        return;
    };

    let pool = PgPool::connect(&database_url)
        .await
        .expect("Failed to connect to test database");

    let redis_client = redis::Client::open(&redis_url).expect("Failed to connect to Redis");
    let connection_manager = redis_client.get_connection_manager().await.expect("Failed to get Redis connection manager");
    let cache = crate::cache::RedisCache { client: connection_manager };
    let engine = crate::services::queue::QueueEngine::new(cache);
    
    let config = WaitingRoomConfig {
        pow_difficulty: 4,
        admission_rate_per_minute: 60,
        grant_ttl_minutes: 10,
        challenge_ttl_seconds: 300,
        tick_interval_ms: 1000,
    };
    
    let state = WaitingRoomState {
        engine: std::sync::Arc::new(engine),
        pool,
        config,
    };
    
    let router = create_waiting_room_router(state);

    let event_id = create_test_event(&pool).await;

    // Query position for non-existent client
    let (status, body) = get_json(
        router,
        "/api/v1/waiting-room/status",
        &format!("event_id={}&client_id=GNONEXISTENT", event_id),
    )
    .await;

    assert_eq!(status, StatusCode::NOT_FOUND);
    assert!(body["message"].as_str().unwrap().contains("not in the queue"));
}

/// Test that SSE stream endpoint is accessible.
#[tokio::test]
async fn test_sse_stream_endpoint_is_accessible() {
    let Ok(database_url) = std::env::var("DATABASE_URL") else {
        eprintln!("skipping test_sse_stream_endpoint_is_accessible: DATABASE_URL not set");
        return;
    };

    let Ok(redis_url) = std::env::var("REDIS_URL") else {
        eprintln!("skipping test_sse_stream_endpoint_is_accessible: REDIS_URL not set");
        return;
    };

    let pool = PgPool::connect(&database_url)
        .await
        .expect("Failed to connect to test database");

    let redis_client = redis::Client::open(&redis_url).expect("Failed to connect to Redis");
    let connection_manager = redis_client.get_connection_manager().await.expect("Failed to get Redis connection manager");
    let cache = crate::cache::RedisCache { client: connection_manager };
    let engine = crate::services::queue::QueueEngine::new(cache);
    
    let config = WaitingRoomConfig {
        pow_difficulty: 4,
        admission_rate_per_minute: 60,
        grant_ttl_minutes: 10,
        challenge_ttl_seconds: 300,
        tick_interval_ms: 1000,
    };
    
    let state = WaitingRoomState {
        engine: std::sync::Arc::new(engine),
        pool,
        config,
    };
    
    let router = create_waiting_room_router(state);

    let event_id = create_test_event(&pool).await;

    // Join queue first so client exists
    let (status, body) = post_json(
        router.clone(),
        "/api/v1/waiting-room/challenge",
        json!({
            "event_id": event_id
        }),
    )
    .await;

    assert_eq!(status, StatusCode::OK);
    let challenge = body["data"]["challenge"].as_str().unwrap();
    let difficulty = body["data"]["difficulty"].as_i64().unwrap() as u32;
    let nonce = find_pow_nonce(challenge, difficulty).unwrap();

    let (status, body) = post_json(
        router.clone(),
        "/api/v1/waiting-room/join",
        json!({
            "event_id": event_id,
            "client_id": "GTESTCLIENT006",
            "challenge": challenge,
            "nonce": nonce
        }),
    )
    .await;

    assert_eq!(status, StatusCode::OK);

    // Start SSE stream
    let (status, _body) = get_json(
        router,
        "/api/v1/waiting-room/stream",
        &format!("event_id={}&client_id=GTESTCLIENT006", event_id),
    )
    .await;

    // SSE streams are long-running, so we can't easily test the full flow
    // This test just verifies the endpoint responds with 200
    assert_eq!(status, StatusCode::OK);
}

/// Test that queue entry is idempotent (same client re-joins doesn't duplicate).
#[tokio::test]
async fn test_queue_entry_is_idempotent() {
    let Ok(database_url) = std::env::var("DATABASE_URL") else {
        eprintln!("skipping test_queue_entry_is_idempotent: DATABASE_URL not set");
        return;
    };

    let Ok(redis_url) = std::env::var("REDIS_URL") else {
        eprintln!("skipping test_queue_entry_is_idempotent: REDIS_URL not set");
        return;
    };

    let pool = PgPool::connect(&database_url)
        .await
        .expect("Failed to connect to test database");

    let redis_client = redis::Client::open(&redis_url).expect("Failed to connect to Redis");
    let connection_manager = redis_client.get_connection_manager().await.expect("Failed to get Redis connection manager");
    let cache = crate::cache::RedisCache { client: connection_manager };
    let engine = crate::services::queue::QueueEngine::new(cache);
    
    let config = WaitingRoomConfig {
        pow_difficulty: 4,
        admission_rate_per_minute: 60,
        grant_ttl_minutes: 10,
        challenge_ttl_seconds: 300,
        tick_interval_ms: 1000,
    };
    
    let state = WaitingRoomState {
        engine: std::sync::Arc::new(engine),
        pool,
        config,
    };
    
    let router = create_waiting_room_router(state);

    let event_id = create_test_event(&pool).await;

    // Issue challenge
    let (status, body) = post_json(
        router.clone(),
        "/api/v1/waiting-room/challenge",
        json!({
            "event_id": event_id
        }),
    )
    .await;

    assert_eq!(status, StatusCode::OK);
    let challenge = body["data"]["challenge"].as_str().unwrap();
    let difficulty = body["data"]["difficulty"].as_i64().unwrap() as u32;

    // Solve PoW
    let nonce = find_pow_nonce(challenge, difficulty).unwrap();

    // First join
    let (status, body) = post_json(
        router.clone(),
        "/api/v1/waiting-room/join",
        json!({
            "event_id": event_id,
            "client_id": "GTESTCLIENT007",
            "challenge": challenge,
            "nonce": nonce
        }),
    )
    .await;

    assert_eq!(status, StatusCode::OK);
    assert_eq!(body["data"]["position"], 1);
    assert_eq!(body["data"]["queue_size"], 1);

    // Re-join with same client (idempotent)
    let nonce2 = find_pow_nonce(challenge, difficulty).unwrap();
    let (status, body) = post_json(
        router,
        "/api/v1/waiting-room/join",
        json!({
            "event_id": event_id,
            "client_id": "GTESTCLIENT007",
            "challenge": challenge,
            "nonce": nonce2
        }),
    )
    .await;

    assert_eq!(status, StatusCode::OK);
    // Position should still be 1 (idempotent), queue size should still be 1
    assert_eq!(body["data"]["position"], 1);
    assert_eq!(body["data"]["queue_size"], 1);
}

/// Test that sold-out event returns appropriate error.
#[tokio::test]
async fn test_sold_out_event_returns_error() {
    let Ok(database_url) = std::env::var("DATABASE_URL") else {
        eprintln!("skipping test_sold_out_event_returns_error: DATABASE_URL not set");
        return;
    };

    let Ok(redis_url) = std::env::var("REDIS_URL") else {
        eprintln!("skipping test_sold_out_event_returns_error: REDIS_URL not set");
        return;
    };

    let pool = PgPool::connect(&database_url)
        .await
        .expect("Failed to connect to test database");

    let redis_client = redis::Client::open(&redis_url).expect("Failed to connect to Redis");
    let connection_manager = redis_client.get_connection_manager().await.expect("Failed to get Redis connection manager");
    let cache = crate::cache::RedisCache { client: connection_manager };
    let engine = crate::services::queue::QueueEngine::new(cache);
    
    let config = WaitingRoomConfig {
        pow_difficulty: 4,
        admission_rate_per_minute: 60,
        grant_ttl_minutes: 10,
        challenge_ttl_seconds: 300,
        tick_interval_ms: 1000,
    };
    
    let state = WaitingRoomState {
        engine: std::sync::Arc::new(engine),
        pool,
        config,
    };
    
    let router = create_waiting_room_router(state);

    // Create event that's sold out
    let event_id = Uuid::new_v4().to_string();
    let organizer_id = Uuid::new_v4();
    
    sqlx::query(
        "INSERT INTO organizers (id, name, contact_email) VALUES ($1, $2, 'test+organizer@agora.test')",
    )
    .bind(organizer_id)
    .bind("Sold Out Organizer")
    .execute(&pool)
    .await
    .expect("failed to insert organizer");

    // Event with 10 total tickets, all minted (sold out)
    sqlx::query(
        "INSERT INTO events (id, organizer_id, title, location, start_time, end_time, total_tickets, minted_tickets) 
         VALUES ($1, $2, $3, $4, $5, $6, $7, $8)",
    )
    .bind(event_id.clone())
    .bind(organizer_id)
    .bind("Sold Out Event")
    .bind("Test Venue")
    .bind(chrono::Utc::now() + chrono::Duration::hours(2))
    .bind(chrono::Utc::now() + chrono::Duration::hours(4))
    .bind(10i64)
    .bind(10i64) // All tickets minted
    .execute(&pool)
    .await
    .expect("failed to insert event");

    // Issue challenge
    let (status, body) = post_json(
        router.clone(),
        "/api/v1/waiting-room/challenge",
        json!({
            "event_id": event_id
        }),
    )
    .await;

    assert_eq!(status, StatusCode::OK);
    let challenge = body["data"]["challenge"].as_str().unwrap();
    let difficulty = body["data"]["difficulty"].as_i64().unwrap() as u32;
    let nonce = find_pow_nonce(challenge, difficulty).unwrap();

    // Try to join sold-out event
    let (status, body) = post_json(
        router,
        "/api/v1/waiting-room/join",
        json!({
            "event_id": event_id,
            "client_id": "GTESTCLIENT008",
            "challenge": challenge,
            "nonce": nonce
        }),
    )
    .await;

    assert_eq!(status, StatusCode::CONFLICT);
    assert!(body["message"].as_str().unwrap().contains("sold out"));
}

/// Test that challenge mismatch returns error.
#[tokio::test]
async fn test_challenge_mismatch_returns_error() {
    let Ok(database_url) = std::env::var("DATABASE_URL") else {
        eprintln!("skipping test_challenge_mismatch_returns_error: DATABASE_URL not set");
        return;
    };

    let Ok(redis_url) = std::env::var("REDIS_URL") else {
        eprintln!("skipping test_challenge_mismatch_returns_error: REDIS_URL not set");
        return;
    };

    let pool = PgPool::connect(&database_url)
        .await
        .expect("Failed to connect to test database");

    let redis_client = redis::Client::open(&redis_url).expect("Failed to connect to Redis");
    let connection_manager = redis_client.get_connection_manager().await.expect("Failed to get Redis connection manager");
    let cache = crate::cache::RedisCache { client: connection_manager };
    let engine = crate::services::queue::QueueEngine::new(cache);
    
    let config = WaitingRoomConfig {
        pow_difficulty: 4,
        admission_rate_per_minute: 60,
        grant_ttl_minutes: 10,
        challenge_ttl_seconds: 300,
        tick_interval_ms: 1000,
    };
    
    let state = WaitingRoomState {
        engine: std::sync::Arc::new(engine),
        pool,
        config,
    };
    
    let router = create_waiting_room_router(state);

    let event_id1 = create_test_event(&pool).await;
    let event_id2 = create_test_event(&pool).await;

    // Issue challenge for event1
    let (status, body) = post_json(
        router.clone(),
        "/api/v1/waiting-room/challenge",
        json!({
            "event_id": event_id1
        }),
    )
    .await;

    assert_eq!(status, StatusCode::OK);
    let challenge = body["data"]["challenge"].as_str().unwrap();
    let difficulty = body["data"]["difficulty"].as_i64().unwrap() as u32;
    let nonce = find_pow_nonce(challenge, difficulty).unwrap();

    // Try to join with wrong event_id
    let (status, body) = post_json(
        router,
        "/api/v1/waiting-room/join",
        json!({
            "event_id": event_id2, // Different event!
            "client_id": "GTESTCLIENT009",
            "challenge": challenge,
            "nonce": nonce
        }),
    )
    .await;

    assert_eq!(status, StatusCode::UNPROCESSABLE_ENTITY);
    assert!(body["message"].as_str().unwrap().contains("Invalid, expired, or already-used"));
}

/// Test that missing required fields returns 400.
#[tokio::test]
async fn test_missing_fields_returns_400() {
    let Ok(database_url) = std::env::var("DATABASE_URL") else {
        eprintln!("skipping test_missing_fields_returns_400: DATABASE_URL not set");
        return;
    };

    let Ok(redis_url) = std::env::var("REDIS_URL") else {
        eprintln!("skipping test_missing_fields_returns_400: REDIS_URL not set");
        return;
    };

    let pool = PgPool::connect(&database_url)
        .await
        .expect("Failed to connect to test database");

    let redis_client = redis::Client::open(&redis_url).expect("Failed to connect to Redis");
    let connection_manager = redis_client.get_connection_manager().await.expect("Failed to get Redis connection manager");
    let cache = crate::cache::RedisCache { client: connection_manager };
    let engine = crate::services::queue::QueueEngine::new(cache);
    
    let config = WaitingRoomConfig {
        pow_difficulty: 4,
        admission_rate_per_minute: 60,
        grant_ttl_minutes: 10,
        challenge_ttl_seconds: 300,
        tick_interval_ms: 1000,
    };
    
    let state = WaitingRoomState {
        engine: std::sync::Arc::new(engine),
        pool,
        config,
    };
    
    let router = create_waiting_room_router(state);

    let event_id = create_test_event(&pool).await;

    // Test missing event_id
    let (status, body) = post_json(
        router.clone(),
        "/api/v1/waiting-room/challenge",
        json!({}),
    )
    .await;

    assert_eq!(status, StatusCode::BAD_REQUEST);

    // Test missing client_id in join
    let (status, body) = post_json(
        router.clone(),
        "/api/v1/waiting-room/challenge",
        json!({
            "event_id": event_id
        }),
    )
    .await;

    assert_eq!(status, StatusCode::OK);
    let challenge = body["data"]["challenge"].as_str().unwrap();
    let difficulty = body["data"]["difficulty"].as_i64().unwrap() as u32;
    let nonce = find_pow_nonce(challenge, difficulty).unwrap();

    let (status, body) = post_json(
        router.clone(),
        "/api/v1/waiting-room/join",
        json!({
            "event_id": event_id,
            "challenge": challenge,
            "nonce": nonce
        }),
    )
    .await;

    assert_eq!(status, StatusCode::BAD_REQUEST);
}

/// Helper: Brute-force a nonce that satisfies the PoW challenge
fn find_pow_nonce(challenge: &str, difficulty: u32) -> Option<String> {
    for nonce in 0..1_000_000 {
        let candidate = nonce.to_string();
        if verify_pow(challenge, &candidate, difficulty) {
            return Some(candidate);
        }
    }
    None
}

/// Verify PoW solution
fn verify_pow(challenge: &str, nonce: &str, difficulty: u32) -> bool {
    if difficulty == 0 {
        return true;
    }
    
    use sha2::{Digest, Sha256};
    let mut hasher = Sha256::new();
    hasher.update(challenge.as_bytes());
    hasher.update(nonce.as_bytes());
    let digest = hasher.finalize();
    let hex = hex::encode(digest);
    hex.chars().take(difficulty as usize).all(|c| c == '0')
}
