use agora_server::handlers::{
    auth::issue_jwt,
    qr_payload::{
        add_event_scanner, delete_qr_payload, mark_qr_used, remove_event_scanner,
        EventScannerRequest,
    },
};
use axum::{
    extract::{Path, State},
    http::{HeaderMap, StatusCode},
    Json,
};
use chrono::{Duration, Utc};
use sqlx::PgPool;
use uuid::Uuid;

struct Fixture {
    pool: PgPool,
    organizer_id: Uuid,
    event_id: Uuid,
    ticket_id: Uuid,
    organizer_wallet: String,
    scanner_wallet: String,
    owner_wallet: String,
    other_wallet: String,
}

async fn fixture() -> Fixture {
    let database_url =
        std::env::var("DATABASE_URL").expect("DATABASE_URL must be set for QR authorization tests");
    let pool = PgPool::connect(&database_url)
        .await
        .expect("connect to the QR authorization test database");

    std::env::set_var("JWT_SECRET", "qr-authorization-test-secret");

    let organizer_id = Uuid::new_v4();
    let event_id = Uuid::new_v4();
    let ticket_id = Uuid::new_v4();
    let organizer_wallet = format!("organizer-{organizer_id}");
    let scanner_wallet = format!("scanner-{event_id}");
    let owner_wallet = format!("owner-{ticket_id}");
    let other_wallet = format!("other-{event_id}");

    sqlx::query(
        "INSERT INTO organizers (id, name, contact_email, wallet_address) VALUES ($1, $2, $3, $4)",
    )
    .bind(organizer_id)
    .bind("QR authorization integration test")
    .bind(format!("{organizer_id}@agora.test"))
    .bind(&organizer_wallet)
    .execute(&pool)
    .await
    .expect("insert test organizer");

    sqlx::query(
        "INSERT INTO events (id, organizer_id, title, location, start_time) VALUES ($1, $2, $3, $4, $5)",
    )
    .bind(event_id)
    .bind(organizer_id)
    .bind("QR authorization integration test")
    .bind("Test venue")
    .bind(Utc::now() + Duration::hours(2))
    .execute(&pool)
    .await
    .expect("insert test event");

    sqlx::query(
        "INSERT INTO tickets (id, event_id, status, owner_wallet, buyer_wallet) VALUES ($1, $2, $3, $4, $5)",
    )
    .bind(ticket_id)
    .bind(event_id)
    .bind("active")
    .bind(format!("buyer-{ticket_id}"))
    .bind(&owner_wallet)
    .execute(&pool)
    .await
    .expect("insert test ticket");

    Fixture {
        pool,
        organizer_id,
        event_id,
        ticket_id,
        organizer_wallet,
        scanner_wallet,
        owner_wallet,
        other_wallet,
    }
}

fn auth_headers(wallet: &str) -> HeaderMap {
    let token = issue_jwt(wallet).expect("issue test JWT");
    let mut headers = HeaderMap::new();
    headers.insert(
        axum::http::header::AUTHORIZATION,
        format!("Bearer {token}").parse().unwrap(),
    );
    headers
}

async fn create_qr(fixture: &Fixture) -> Uuid {
    let qr_id = Uuid::new_v4();
    sqlx::query(
        r#"
        INSERT INTO qr_payloads
            (id, qr_type, payload_data, signature, public_key, expires_at, event_id, ticket_id)
        VALUES ($1, 'ticket', $2, 'test-signature', 'test-public-key', $3, $4, $5)
        "#,
    )
    .bind(qr_id.to_string())
    .bind(serde_json::json!({"ticket_id": fixture.ticket_id}))
    .bind(Utc::now() + Duration::hours(1))
    .bind(fixture.event_id)
    .bind(fixture.ticket_id)
    .execute(&fixture.pool)
    .await
    .expect("insert test QR payload");
    qr_id
}

async fn cleanup(fixture: Fixture) {
    sqlx::query("DELETE FROM organizers WHERE id = $1")
        .bind(fixture.organizer_id)
        .execute(&fixture.pool)
        .await
        .expect("remove QR authorization test data");
    fixture.pool.close().await;
}

#[tokio::test]
async fn mark_qr_used_requires_an_event_organizer_or_assigned_scanner() {
    let fixture = fixture().await;
    let qr_id = create_qr(&fixture).await;

    let response = mark_qr_used(
        State(fixture.pool.clone()),
        auth_headers(&fixture.other_wallet),
        Path(qr_id.to_string()),
    )
    .await;
    assert_eq!(response.status(), StatusCode::FORBIDDEN);

    let is_used: bool = sqlx::query_scalar("SELECT is_used FROM qr_payloads WHERE id = $1")
        .bind(qr_id.to_string())
        .fetch_one(&fixture.pool)
        .await
        .expect("read unchanged QR payload");
    assert!(
        !is_used,
        "unauthorized requests must not use the QR payload"
    );

    let response = add_event_scanner(
        State(fixture.pool.clone()),
        auth_headers(&fixture.other_wallet),
        Path(fixture.event_id),
        Json(EventScannerRequest {
            wallet_address: fixture.scanner_wallet.clone(),
        }),
    )
    .await;
    assert_eq!(response.status(), StatusCode::FORBIDDEN);

    let response = add_event_scanner(
        State(fixture.pool.clone()),
        auth_headers(&fixture.organizer_wallet),
        Path(fixture.event_id),
        Json(EventScannerRequest {
            wallet_address: fixture.scanner_wallet.clone(),
        }),
    )
    .await;
    assert_eq!(response.status(), StatusCode::NO_CONTENT);

    let response = mark_qr_used(
        State(fixture.pool.clone()),
        auth_headers(&fixture.scanner_wallet),
        Path(qr_id.to_string()),
    )
    .await;
    assert_eq!(response.status(), StatusCode::OK);

    let revoked_qr_id = create_qr(&fixture).await;
    let response = remove_event_scanner(
        State(fixture.pool.clone()),
        auth_headers(&fixture.organizer_wallet),
        Path((fixture.event_id, fixture.scanner_wallet.clone())),
    )
    .await;
    assert_eq!(response.status(), StatusCode::NO_CONTENT);

    let response = mark_qr_used(
        State(fixture.pool.clone()),
        auth_headers(&fixture.scanner_wallet),
        Path(revoked_qr_id.to_string()),
    )
    .await;
    assert_eq!(response.status(), StatusCode::FORBIDDEN);

    let organizer_qr_id = create_qr(&fixture).await;
    let response = mark_qr_used(
        State(fixture.pool.clone()),
        auth_headers(&fixture.organizer_wallet),
        Path(organizer_qr_id.to_string()),
    )
    .await;
    assert_eq!(response.status(), StatusCode::OK);

    cleanup(fixture).await;
}

#[tokio::test]
async fn delete_qr_payload_requires_ticket_ownership_or_event_organizer() {
    let fixture = fixture().await;
    let qr_id = create_qr(&fixture).await;

    let response = delete_qr_payload(
        State(fixture.pool.clone()),
        auth_headers(&fixture.other_wallet),
        Path(qr_id),
    )
    .await;
    assert_eq!(response.status(), StatusCode::FORBIDDEN);

    let still_exists: bool =
        sqlx::query_scalar("SELECT EXISTS(SELECT 1 FROM qr_payloads WHERE id = $1)")
            .bind(qr_id.to_string())
            .fetch_one(&fixture.pool)
            .await
            .expect("check QR payload after rejected delete");
    assert!(
        still_exists,
        "unauthorized requests must not delete the QR payload"
    );

    let response = delete_qr_payload(
        State(fixture.pool.clone()),
        auth_headers(&format!("buyer-{}", fixture.ticket_id)),
        Path(qr_id),
    )
    .await;
    assert_eq!(response.status(), StatusCode::FORBIDDEN);

    let response = delete_qr_payload(
        State(fixture.pool.clone()),
        auth_headers(&fixture.owner_wallet),
        Path(qr_id),
    )
    .await;
    assert_eq!(response.status(), StatusCode::NO_CONTENT);

    let organizer_qr_id = create_qr(&fixture).await;
    let response = delete_qr_payload(
        State(fixture.pool.clone()),
        auth_headers(&fixture.organizer_wallet),
        Path(organizer_qr_id),
    )
    .await;
    assert_eq!(response.status(), StatusCode::NO_CONTENT);

    cleanup(fixture).await;
}
