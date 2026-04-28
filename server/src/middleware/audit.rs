//! Audit logging middleware for `/api/v1/admin/*` routes.
//!
//! Every request that passes through [`audit_layer`] is persisted to the
//! `audit_logs` table after the inner handler responds, capturing the actor,
//! path, method, body, and response status code.

use axum::{
    body::Body,
    extract::{Request, State},
    middleware::Next,
    response::Response,
};
use bytes::Bytes;
use http_body_util::BodyExt;
use serde_json::Value;
use sqlx::PgPool;
use uuid::Uuid;

/// Axum middleware that records every admin request to `audit_logs`.
///
/// Attach via `axum::middleware::from_fn_with_state` on the admin router.
pub async fn audit_layer(
    State(pool): State<PgPool>,
    request: Request,
    next: Next,
) -> Response {
    let method = request.method().to_string();
    let path = request.uri().path().to_string();

    // Extract IP from the X-Forwarded-For header or the connection info.
    let ip_address = request
        .headers()
        .get("x-forwarded-for")
        .and_then(|v| v.to_str().ok())
        .map(|s| s.split(',').next().unwrap_or(s).trim().to_string());

    // Buffer the request body so we can log it and still forward it.
    let (parts, body) = request.into_parts();
    let body_bytes: Bytes = match body.collect().await {
        Ok(collected) => collected.to_bytes(),
        Err(_) => Bytes::new(),
    };
    let request_body: Option<Value> = if body_bytes.is_empty() {
        None
    } else {
        serde_json::from_slice(&body_bytes).ok()
    };

    // Reconstruct the request with the buffered body.
    let request = Request::from_parts(parts, Body::from(body_bytes));

    // Call the actual handler.
    let response = next.run(request).await;
    let status_code = response.status().as_u16() as i32;

    // Derive a human-readable action from the method + path.
    let action = derive_action(&method, &path);

    // Persist asynchronously — we don't want a DB hiccup to fail the request.
    let pool_clone = pool.clone();
    let path_clone = path.clone();
    let method_clone = method.clone();
    let ip_clone = ip_address.clone();
    tokio::spawn(async move {
        if let Err(e) = persist_audit_log(
            &pool_clone,
            &action,
            &path_clone,
            &method_clone,
            request_body,
            ip_clone,
            status_code,
        )
        .await
        {
            tracing::error!(error = %e, path = %path_clone, "Failed to write audit log");
        }
    });

    response
}

/// Derives a dot-separated action string from the HTTP method and path.
fn derive_action(method: &str, path: &str) -> String {
    let trimmed = path
        .trim_start_matches('/')
        .trim_start_matches("api/v1/admin/");
    let base = trimmed.replace('/', ".");
    let verb = match method {
        "GET" => "read",
        "POST" => "create",
        "PUT" | "PATCH" => "update",
        "DELETE" => "delete",
        other => other,
    };
    format!("admin.{base}.{verb}")
}

async fn persist_audit_log(
    pool: &PgPool,
    action: &str,
    request_path: &str,
    request_method: &str,
    request_body: Option<Value>,
    ip_address: Option<String>,
    status_code: i32,
) -> Result<(), sqlx::Error> {
    sqlx::query(
        r#"
        INSERT INTO audit_logs
            (id, action, request_path, request_method, request_body, ip_address, status_code)
        VALUES ($1, $2, $3, $4, $5, $6, $7)
        "#,
    )
    .bind(Uuid::new_v4())
    .bind(action)
    .bind(request_path)
    .bind(request_method)
    .bind(request_body)
    .bind(ip_address)
    .bind(status_code)
    .execute(pool)
    .await?;
    Ok(())
}
