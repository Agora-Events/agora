//! Ticket PDF generation handler (Issue #1341)

use axum::{
    extract::{Path, State},
    http::{HeaderMap, HeaderValue, StatusCode},
    response::{IntoResponse, Response},
};
use redis::AsyncCommands;
use sqlx::PgPool;
use std::time::Duration;
use uuid::Uuid;

use crate::cache::RedisCache;
use crate::handlers::auth::extract_auth;
use crate::utils::error::AppError;
use crate::utils::pdf::{generate_ticket_pdf, mask_wallet};

#[derive(Clone)]
pub struct TicketState {
    pub pool: PgPool,
    pub redis: RedisCache,
}

const PDF_CACHE_TTL: Duration = Duration::from_secs(3600);

fn pdf_cache_key(ticket_id: Uuid) -> String {
    format!("ticket:pdf:{ticket_id}")
}

/// GET /api/v1/tickets/:id/pdf
#[utoipa::path(
    get,
    path = "/tickets/{ticket_id}/pdf",
    params(("ticket_id" = Uuid, Path, description = "Ticket identifier")),
    responses(
        (status = 200, description = "Ticket PDF", content_type = "application/pdf", body = String),
        (status = 401, description = "Authentication required"),
        (status = 403, description = "Requester does not own this ticket"),
        (status = 404, description = "Ticket not found"),
    ),
    tag = "Tickets"
)]
pub async fn get_ticket_pdf(
    State(state): State<TicketState>,
    headers: HeaderMap,
    Path(ticket_id): Path<Uuid>,
) -> Response {
    let requester = match extract_auth(&headers) {
        Ok(a) => a,
        Err(e) => return e.into_response(),
    };

    let pool = state.pool.clone();
    let redis = state.redis.clone();

    get_ticket_pdf_inner(
        ticket_id,
        &requester,
        || async move {
            sqlx::query_as::<_, TicketPdfRow>(
                r#"
                SELECT
                    t.id,
                    t.owner_wallet,
                    t.buyer_wallet,
                    t.qr_code,
                    tt.name as tier_name,
                    e.title as event_title,
                    e.location as venue,
                    e.start_time as event_start,
                    e.image_url as event_cover
                FROM tickets t
                LEFT JOIN ticket_tiers tt ON tt.id = t.ticket_tier_id
                LEFT JOIN events e ON e.id = t.event_id
                WHERE t.id = $1
                "#,
            )
            .bind(ticket_id)
            .fetch_optional(&pool)
            .await
        },
        move |cache_key| {
            let redis = redis.clone();
            async move {
                let mut conn = redis.connection();
                conn.get::<_, Option<Vec<u8>>>(&cache_key)
                    .await
                    .unwrap_or(None)
            }
        },
        move |cache_key, pdf_bytes| {
            let redis = state.redis.clone();
            async move {
                let mut conn = redis.connection();
                let _: Result<(), _> = conn
                    .set_ex::<_, _, ()>(&cache_key, pdf_bytes, PDF_CACHE_TTL.as_secs())
                    .await;
            }
        },
    )
    .await
}

async fn get_ticket_pdf_inner<FLoad, FutLoad, FGet, FutGet, FSet, FutSet>(
    ticket_id: Uuid,
    requester: &str,
    load_ticket: FLoad,
    get_cached: FGet,
    set_cached: FSet,
) -> Response
where
    FLoad: FnOnce() -> FutLoad,
    FutLoad: std::future::Future<Output = Result<Option<TicketPdfRow>, sqlx::Error>>,
    FGet: FnOnce(String) -> FutGet,
    FutGet: std::future::Future<Output = Option<Vec<u8>>>,
    FSet: FnOnce(String, Vec<u8>) -> FutSet,
    FutSet: std::future::Future<Output = ()>,
{
    // Load ticket row and verify ownership before reading from the cache.
    let row = match load_ticket().await {
        Ok(Some(r)) => r,
        Ok(None) => {
            return AppError::NotFound(format!("Ticket {ticket_id} not found")).into_response()
        }
        Err(e) => return AppError::DatabaseError(e).into_response(),
    };

    // Ownership check: owner_wallet or buyer_wallet must match requester
    let is_owner = row
        .owner_wallet
        .as_deref()
        .map(|w| w == requester)
        .unwrap_or(false)
        || row
            .buyer_wallet
            .as_deref()
            .map(|w| w == requester)
            .unwrap_or(false);

    if !is_owner {
        return AppError::Forbidden("You do not own this ticket".to_string()).into_response();
    }

    let cache_key = pdf_cache_key(ticket_id);
    if let Some(bytes) = get_cached(cache_key.clone()).await {
        if !bytes.is_empty() {
            return pdf_response(bytes);
        }
    }

    let masked = row
        .owner_wallet
        .as_deref()
        .or(row.buyer_wallet.as_deref())
        .map(mask_wallet)
        .unwrap_or_else(|| "N/A".to_string());

    let tier = row
        .tier_name
        .clone()
        .unwrap_or_else(|| "General".to_string());
    let title = row
        .event_title
        .clone()
        .unwrap_or_else(|| "Event".to_string());
    let venue = row.venue.clone().unwrap_or_else(|| "Venue TBA".to_string());
    let qr = row.qr_code.clone().unwrap_or_else(|| "no-qr".to_string());
    let date_str = row
        .event_start
        .map(|d| d.format("%Y-%m-%d %H:%M UTC").to_string())
        .unwrap_or_else(|| "Date TBA".to_string());

    let pdf_bytes = generate_ticket_pdf(&title, &date_str, &venue, &tier, &masked, &qr);

    // Cache in Redis for 1 hour
    set_cached(cache_key, pdf_bytes.clone()).await;

    pdf_response(pdf_bytes)
}

#[derive(Debug, sqlx::FromRow)]
struct TicketPdfRow {
    id: Uuid,
    owner_wallet: Option<String>,
    buyer_wallet: Option<String>,
    qr_code: Option<String>,
    tier_name: Option<String>,
    event_title: Option<String>,
    venue: Option<String>,
    event_start: Option<chrono::DateTime<chrono::Utc>>,
    event_cover: Option<String>,
}

fn pdf_response(bytes: Vec<u8>) -> Response {
    let mut resp = (StatusCode::OK, bytes).into_response();
    resp.headers_mut().insert(
        axum::http::header::CONTENT_TYPE,
        HeaderValue::from_static("application/pdf"),
    );
    resp.headers_mut().insert(
        axum::http::header::CONTENT_DISPOSITION,
        HeaderValue::from_static("inline; filename=\"ticket.pdf\""),
    );
    resp.headers_mut().insert(
        axum::http::header::CACHE_CONTROL,
        HeaderValue::from_static("private, no-store"),
    );
    resp
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::collections::HashMap;
    use std::sync::{
        atomic::{AtomicBool, Ordering},
        Arc, Mutex,
    };

    fn sample_ticket_row(id: Uuid, owner: &str, buyer: Option<&str>) -> TicketPdfRow {
        TicketPdfRow {
            id,
            owner_wallet: Some(owner.to_string()),
            buyer_wallet: buyer.map(str::to_string),
            qr_code: Some("qr-payload".to_string()),
            tier_name: Some("VIP".to_string()),
            event_title: Some("Stellar Summit".to_string()),
            venue: Some("Main Hall".to_string()),
            event_start: None,
            event_cover: None,
        }
    }

    #[test]
    fn test_pdf_cache_key() {
        let id = Uuid::new_v4();
        let key = pdf_cache_key(id);
        assert_eq!(key, format!("ticket:pdf:{id}"));
    }

    #[test]
    fn test_pdf_response_has_correct_headers() {
        let resp = pdf_response(vec![1, 2, 3]);
        assert_eq!(
            resp.headers()
                .get(axum::http::header::CONTENT_TYPE)
                .unwrap(),
            "application/pdf"
        );
        assert_eq!(
            resp.headers()
                .get(axum::http::header::CACHE_CONTROL)
                .unwrap(),
            "private, no-store"
        );
    }

    #[tokio::test]
    async fn test_non_owner_gets_403_even_when_pdf_is_cached() {
        let ticket_id = Uuid::new_v4();
        let owner = "GOWNER1111111111111111111111111111111111111111111111111";
        let attacker = "GATTACKER2222222222222222222222222222222222222222222222";

        let cache = Arc::new(Mutex::new(HashMap::from([(
            pdf_cache_key(ticket_id),
            b"%PDF-1.4 cached".to_vec(),
        )])));
        let cache_was_read = Arc::new(AtomicBool::new(false));
        let cache_was_read_clone = cache_was_read.clone();

        let resp = get_ticket_pdf_inner(
            ticket_id,
            attacker,
            || async move { Ok(Some(sample_ticket_row(ticket_id, owner, None))) },
            move |key| {
                let cache = cache.clone();
                async move {
                    cache_was_read_clone.store(true, Ordering::SeqCst);
                    cache.lock().unwrap().get(&key).cloned()
                }
            },
            |_key, _bytes| async {},
        )
        .await;

        assert_eq!(resp.status(), StatusCode::FORBIDDEN);
        assert!(
            !cache_was_read.load(Ordering::SeqCst),
            "cache must not be read before ownership check passes"
        );
    }

    #[tokio::test]
    async fn test_owner_receives_cached_pdf_with_no_store_header() {
        let ticket_id = Uuid::new_v4();
        let owner = "GOWNER1111111111111111111111111111111111111111111111111";
        let cached_bytes = b"%PDF-1.4 cached".to_vec();

        let cache = Arc::new(Mutex::new(HashMap::from([(
            pdf_cache_key(ticket_id),
            cached_bytes.clone(),
        )])));

        let resp = get_ticket_pdf_inner(
            ticket_id,
            owner,
            || async move { Ok(Some(sample_ticket_row(ticket_id, owner, None))) },
            move |key| {
                let cache = cache.clone();
                async move { cache.lock().unwrap().get(&key).cloned() }
            },
            |_key, _bytes| async {},
        )
        .await;

        assert_eq!(resp.status(), StatusCode::OK);
        assert_eq!(
            resp.headers()
                .get(axum::http::header::CACHE_CONTROL)
                .unwrap(),
            "private, no-store"
        );
        let body = axum::body::to_bytes(resp.into_body(), usize::MAX)
            .await
            .unwrap();
        assert_eq!(body.as_ref(), cached_bytes.as_slice());
    }
}
