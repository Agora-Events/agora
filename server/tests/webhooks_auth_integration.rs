mod common;

use agora_server::{
    handlers::auth::issue_jwt,
    handlers::webhooks::{
        create_webhook, delete_webhook, list_webhook_failures, list_webhooks,
        retry_webhook_failure, test_webhook, WebhookState,
    },
};
use axum::{
    body::Body,
    http::{Method, Request, StatusCode},
    routing::{delete, get, post},
    Router,
};
use common::{cleanup_organizer, test_pool};
use serde_json::{json, Value};
use sqlx::PgPool;
use tower::ServiceExt;
use uuid::Uuid;

fn webhook_router(pool: PgPool) -> Router {
    Router::new()
        .route("/webhooks", post(create_webhook).get(list_webhooks))
        .route("/webhooks/failures", get(list_webhook_failures))
        .route("/webhooks/failures/:id/retry", post(retry_webhook_failure))
        .route("/webhooks/:id", delete(delete_webhook))
        .route("/webhooks/:id/test", post(test_webhook))
        .with_state(WebhookState { pool })
}

async fn send_request(
    router: Router,
    method: Method,
    uri: &str,
    body: Option<Value>,
    token: Option<&str>,
) -> (StatusCode, Value) {
    let mut builder = Request::builder().method(method).uri(uri);
    if let Some(token) = token {
        builder = builder.header("Authorization", format!("Bearer {token}"));
    }

    let request_body = match body {
        Some(body) => {
            builder = builder.header("Content-Type", "application/json");
            serde_json::to_vec(&body).unwrap()
        }
        None => Vec::new(),
    };
    let response = router
        .oneshot(builder.body(Body::from(request_body)).unwrap())
        .await
        .unwrap();
    let status = response.status();
    let bytes = axum::body::to_bytes(response.into_body(), usize::MAX)
        .await
        .unwrap();
    let body = serde_json::from_slice(&bytes).unwrap_or(Value::Null);
    (status, body)
}

async fn seed_organizer(pool: &PgPool, name: &str, wallet_address: &str) -> Uuid {
    let id = Uuid::new_v4();
    sqlx::query(
        "INSERT INTO organizers (id, name, contact_email, wallet_address) VALUES ($1, $2, $3, $4)",
    )
    .bind(id)
    .bind(name)
    .bind(format!("{id}@example.test"))
    .bind(wallet_address)
    .execute(pool)
    .await
    .expect("seed webhook test organizer");
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
    .expect("seed webhook endpoint");
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
    .expect("seed webhook delivery failure");
    id
}

#[tokio::test]
async fn webhook_management_is_authenticated_and_organizer_scoped() {
    let Some(pool) = test_pool("webhook_management_is_authenticated_and_organizer_scoped").await
    else {
        return;
    };
    std::env::set_var("JWT_SECRET", "test-secret-do-not-use-in-production");

    let owner_wallet = format!("webhook-owner-{}", Uuid::new_v4());
    let other_wallet = format!("webhook-other-{}", Uuid::new_v4());
    let owner_id = seed_organizer(&pool, "Webhook owner", &owner_wallet).await;
    let other_id = seed_organizer(&pool, "Other webhook owner", &other_wallet).await;
    let owner_endpoint = seed_endpoint(&pool, owner_id).await;
    let other_endpoint = seed_endpoint(&pool, other_id).await;
    let owner_failure = seed_failure(&pool, owner_endpoint).await;
    let other_failure = seed_failure(&pool, other_endpoint).await;
    let owner_token = issue_jwt(&owner_wallet).expect("issue owner JWT");
    let router = webhook_router(pool.clone());
    let create_body = json!({
        "organizer_id": other_id,
        "url": "https://example.test/new-webhook",
        "secret": "new-secret",
        "events": ["PaymentProcessed"]
    });

    for (method, uri, body) in vec![
        (Method::GET, "/webhooks".to_string(), None),
        (
            Method::POST,
            "/webhooks".to_string(),
            Some(create_body.clone()),
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
    ] {
        let (status, _) = send_request(router.clone(), method.clone(), &uri, body, None).await;
        assert_eq!(
            status,
            StatusCode::UNAUTHORIZED,
            "unauthenticated {method} {uri}"
        );
    }

    let (status, created) = send_request(
        router.clone(),
        Method::POST,
        "/webhooks",
        Some(create_body),
        Some(&owner_token),
    )
    .await;
    assert_eq!(status, StatusCode::OK);
    assert_eq!(
        created["data"]["endpoint"]["organizer_id"],
        owner_id.to_string(),
        "the organizer must come from the JWT, not the request body"
    );

    let (status, listed) = send_request(
        router.clone(),
        Method::GET,
        &format!("/webhooks?organizer_id={other_id}"),
        None,
        Some(&owner_token),
    )
    .await;
    assert_eq!(status, StatusCode::OK);
    let listed_endpoints = listed["data"].as_array().unwrap();
    assert!(listed_endpoints
        .iter()
        .any(|endpoint| endpoint["id"] == owner_endpoint.to_string()));
    assert!(listed_endpoints
        .iter()
        .all(|endpoint| endpoint["organizer_id"] == owner_id.to_string()));

    let (status, _) = send_request(
        router.clone(),
        Method::DELETE,
        &format!("/webhooks/{other_endpoint}"),
        None,
        Some(&owner_token),
    )
    .await;
    assert_eq!(status, StatusCode::NOT_FOUND);
    let is_active: bool =
        sqlx::query_scalar("SELECT is_active FROM webhook_endpoints WHERE id = $1")
            .bind(other_endpoint)
            .fetch_one(&pool)
            .await
            .unwrap();
    assert!(
        is_active,
        "cross-organizer delete must not modify the endpoint"
    );

    for (method, uri) in [
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
        let (status, _) = send_request(
            router.clone(),
            method.clone(),
            &uri,
            None,
            Some(&owner_token),
        )
        .await;
        assert_eq!(
            status,
            StatusCode::NOT_FOUND,
            "cross-organizer {method} {uri}"
        );
    }

    let (status, failures) = send_request(
        router.clone(),
        Method::GET,
        "/webhooks/failures",
        None,
        Some(&owner_token),
    )
    .await;
    assert_eq!(status, StatusCode::OK);
    assert!(failures["data"]["items"]
        .as_array()
        .unwrap()
        .iter()
        .all(|failure| { failure["endpoint_id"] == owner_endpoint.to_string() }));
    assert_eq!(
        failures["data"]["items"][0]["id"],
        owner_failure.to_string()
    );

    cleanup_organizer(&pool, owner_id).await;
    cleanup_organizer(&pool, other_id).await;
}

#[tokio::test]
async fn webhook_failure_pagination_follows_cursor_to_second_page() {
    let Some(pool) = test_pool("webhook_failure_pagination_follows_cursor_to_second_page").await
    else {
        return;
    };
    std::env::set_var("JWT_SECRET", "test-secret-do-not-use-in-production");

    let wallet = format!("webhook-pagination-{}", Uuid::new_v4());
    let organizer_id = seed_organizer(&pool, "Webhook pagination owner", &wallet).await;
    let endpoint_id = seed_endpoint(&pool, organizer_id).await;
    let first_id = seed_failure(&pool, endpoint_id).await;
    let second_id = seed_failure(&pool, endpoint_id).await;
    let third_id = seed_failure(&pool, endpoint_id).await;
    let token = issue_jwt(&wallet).expect("issue pagination JWT");
    let router = webhook_router(pool.clone());

    let (status, first_page) = send_request(
        router.clone(),
        Method::GET,
        &format!("/webhooks/failures?endpoint_id={endpoint_id}&limit=2"),
        None,
        Some(&token),
    )
    .await;
    assert_eq!(status, StatusCode::OK);
    assert_eq!(first_page["data"]["items"].as_array().unwrap().len(), 2);
    let next_cursor = first_page["data"]["pagination"]["next_cursor"]
        .as_str()
        .expect("first page should have a next cursor")
        .to_string();

    let (status, second_page) = send_request(
        router,
        Method::GET,
        &format!("/webhooks/failures?endpoint_id={endpoint_id}&limit=2&cursor={next_cursor}"),
        None,
        Some(&token),
    )
    .await;
    assert_eq!(status, StatusCode::OK);
    let second_items = second_page["data"]["items"].as_array().unwrap();
    assert_eq!(second_items.len(), 1);
    assert!(second_items.iter().any(|failure| {
        let id = failure["id"].as_str().unwrap();
        id == first_id.to_string() || id == second_id.to_string() || id == third_id.to_string()
    }));
    assert_eq!(
        second_page["data"]["pagination"]["next_cursor"],
        serde_json::Value::Null
    );

    cleanup_organizer(&pool, organizer_id).await;
}
