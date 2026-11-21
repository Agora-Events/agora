use axum::{
    body::Body,
    http::{Method, Request, StatusCode},
    routing::{delete, get, post},
    Router,
};
use serde_json::{json, Value};
use sqlx::PgPool;
use tower::ServiceExt;
use uuid::Uuid;

use agora_server::handlers::auth::issue_jwt;
use agora_server::handlers::webhooks::{
    create_webhook, delete_webhook, list_webhook_failures, list_webhooks, retry_webhook_failure,
    test_webhook, WebhookState,
};

fn webhook_router(pool: PgPool) -> Router {
    Router::new()
        .route("/webhooks", post(create_webhook).get(list_webhooks))
        .route("/webhooks/failures", get(list_webhook_failures))
        .route("/webhooks/failures/:id/retry", post(retry_webhook_failure))
        .route("/webhooks/:id", delete(delete_webhook))
        .route("/webhooks/:id/test", post(test_webhook))
        .with_state(WebhookState { pool })
}

async fn request(
    router: Router,
    method: Method,
    uri: &str,
    body: Option<Value>,
    token: Option<&str>,
) -> (StatusCode, Value) {
    let mut builder = Request::builder().method(method).uri(uri);
    let bytes = match body {
        Some(body) => {
            builder = builder.header("content-type", "application/json");
            serde_json::to_vec(&body).unwrap()
        }
        None => Vec::new(),
    };
    if let Some(token) = token {
        builder = builder.header("authorization", format!("Bearer {token}"));
    }

    let response = router
        .oneshot(builder.body(Body::from(bytes)).unwrap())
        .await
        .unwrap();
    let status = response.status();
    let bytes = axum::body::to_bytes(response.into_body(), usize::MAX)
        .await
        .unwrap();
    let body = serde_json::from_slice(&bytes).unwrap_or(Value::Null);
    (status, body)
}

async fn test_pool(test_name: &str) -> Option<PgPool> {
    let Ok(database_url) = std::env::var("DATABASE_URL") else {
        eprintln!("skipping {test_name}: DATABASE_URL not set");
        return None;
    };
    PgPool::connect(&database_url).await.ok()
}

async fn seed_organizer(pool: &PgPool, wallet: &str) -> Uuid {
    let id = Uuid::new_v4();
    sqlx::query(
        "INSERT INTO organizers (id, name, contact_email, wallet_address) VALUES ($1, $2, $3, $4)",
    )
    .bind(id)
    .bind(format!("Webhook Test {id}"))
    .bind(format!("{id}@example.test"))
    .bind(wallet)
    .execute(pool)
    .await
    .expect("seed organizer");
    id
}

async fn seed_endpoint(pool: &PgPool, organizer_id: Uuid) -> Uuid {
    let id = Uuid::new_v4();
    sqlx::query(
        r#"
        INSERT INTO webhook_endpoints (id, organizer_id, url, secret, events)
        VALUES ($1, $2, 'https://example.test/webhook', 'test-secret', ARRAY['PaymentProcessed'])
        "#,
    )
    .bind(id)
    .bind(organizer_id)
    .execute(pool)
    .await
    .expect("seed endpoint");
    id
}

async fn seed_failure(pool: &PgPool, endpoint_id: Uuid) -> Uuid {
    let id = Uuid::new_v4();
    sqlx::query(
        r#"
        INSERT INTO webhook_delivery_failures (id, endpoint_id, event, payload)
        VALUES ($1, $2, 'PaymentProcessed', '{}'::jsonb)
        "#,
    )
    .bind(id)
    .bind(endpoint_id)
    .execute(pool)
    .await
    .expect("seed failure");
    id
}

#[tokio::test]
async fn webhook_routes_require_auth_and_scope_cross_organizer_access() {
    let Some(pool) =
        test_pool("webhook_routes_require_auth_and_scope_cross_organizer_access").await
    else {
        return;
    };
    std::env::set_var("JWT_SECRET", "test-secret-do-not-use-in-production");

    let owner_wallet = format!("owner-{}", Uuid::new_v4());
    let other_wallet = format!("other-{}", Uuid::new_v4());
    let owner_id = seed_organizer(&pool, &owner_wallet).await;
    let other_id = seed_organizer(&pool, &other_wallet).await;
    let other_endpoint = seed_endpoint(&pool, other_id).await;
    let other_failure = seed_failure(&pool, other_endpoint).await;
    let owner_token = issue_jwt(&owner_wallet).expect("issue owner JWT");
    let router = webhook_router(pool.clone());

    let unauthenticated = [
        (Method::GET, "/webhooks".to_string(), None),
        (
            Method::POST,
            "/webhooks".to_string(),
            Some(json!({
                "url": "https://example.test/hook",
                "secret": "secret",
                "events": ["PaymentProcessed"]
            })),
        ),
        (Method::DELETE, format!("/webhooks/{other_endpoint}"), None),
        (
            Method::POST,
            format!("/webhooks/{other_endpoint}/test"),
            None,
        ),
        (Method::GET, "/webhooks/failures".to_string(), None),
        (
            Method::POST,
            format!("/webhooks/failures/{other_failure}/retry"),
            None,
        ),
    ];
    for (method, uri, body) in unauthenticated {
        let (status, _) = request(router.clone(), method, &uri, body, None).await;
        assert_eq!(status, StatusCode::UNAUTHORIZED);
    }

    let (status, created) = request(
        router.clone(),
        Method::POST,
        "/webhooks",
        Some(json!({
            "url": "https://example.test/owned-hook",
            "secret": "owned-secret",
            "events": ["PaymentProcessed"]
        })),
        Some(&owner_token),
    )
    .await;
    assert_eq!(status, StatusCode::OK);
    assert_eq!(
        created["data"]["endpoint"]["organizer_id"],
        owner_id.to_string()
    );

    for (method, uri) in [
        (Method::DELETE, format!("/webhooks/{other_endpoint}")),
        (Method::POST, format!("/webhooks/{other_endpoint}/test")),
        (
            Method::GET,
            format!("/webhooks/failures?endpoint_id={other_endpoint}"),
        ),
        (
            Method::POST,
            format!("/webhooks/failures/{other_failure}/retry"),
        ),
    ] {
        let (status, _) = request(router.clone(), method, &uri, None, Some(&owner_token)).await;
        assert_eq!(status, StatusCode::NOT_FOUND);
    }

    let is_active: bool =
        sqlx::query_scalar("SELECT is_active FROM webhook_endpoints WHERE id = $1")
            .bind(other_endpoint)
            .fetch_one(&pool)
            .await
            .unwrap();
    assert!(is_active);

    let _ = sqlx::query("DELETE FROM organizers WHERE id = $1")
        .bind(owner_id)
        .execute(&pool)
        .await;
    let _ = sqlx::query("DELETE FROM organizers WHERE id = $1")
        .bind(other_id)
        .execute(&pool)
        .await;
}
