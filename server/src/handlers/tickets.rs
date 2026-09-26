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

/// GET /api/v1/tickets/:id/pdf
pub async fn get_ticket_pdf(
    State(state): State<TicketState>,
    headers: HeaderMap,
    Path(ticket_id): Path<Uuid>,
) -> Response {
    let requester = match extract_auth(&headers) {
        Ok(a) => a,
        Err(e) => return e.into_response(),
    };

    // Try Redis cache first
    let cache_key = format!("ticket:pdf:{ticket_id}");
    let redis = state.redis.clone();
    {
        let mut conn = redis.connection();
        let cached: Option<Vec<u8>> = conn.get::<_, Option<Vec<u8>>>(&cache_key).await.unwrap_or(None);
        if let Some(bytes) = cached {
            if !bytes.is_empty() {
                return pdf_response(bytes);
            }
        }
    }

    // Fetch ticket with event details
    let row = match sqlx::query_as::<_, TicketPdfRow>(
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
    .fetch_optional(&state.pool)
    .await
    {
        Ok(Some(r)) => r,
        Ok(None) => return AppError::NotFound(format!("Ticket {ticket_id} not found")).into_response(),
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

    let masked = row
        .owner_wallet
        .as_deref()
        .or(row.buyer_wallet.as_deref())
        .map(mask_wallet)
        .unwrap_or_else(|| "N/A".to_string());

    let tier = row.tier_name.clone().unwrap_or_else(|| "General".to_string());
    let title = row.event_title.clone().unwrap_or_else(|| "Event".to_string());
    let venue = row.venue.clone().unwrap_or_else(|| "Venue TBA".to_string());
    let qr = row.qr_code.clone().unwrap_or_else(|| "no-qr".to_string());
    let date_str = row
        .event_start
        .map(|d| d.format("%Y-%m-%d %H:%M UTC").to_string())
        .unwrap_or_else(|| "Date TBA".to_string());

    let pdf_bytes = generate_ticket_pdf(&title, &date_str, &venue, &tier, &masked, &qr);

    // Cache in Redis for 1 hour
    {
        let mut conn = redis.connection();
        let _: Result<(), _> = conn
            .set_ex::<_, _, ()>(&cache_key, pdf_bytes.clone(), PDF_CACHE_TTL.as_secs())
            .await;
    }

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
    resp
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_cache_key_format() {
        let id = Uuid::new_v4();
        let key = format!("ticket:pdf:{id}");
        assert!(key.starts_with("ticket:pdf:"));
    }

    #[test]
    fn test_pdf_response_has_correct_content_type() {
        let resp = pdf_response(vec![1, 2, 3]);
        assert_eq!(
            resp.headers().get(axum::http::header::CONTENT_TYPE).unwrap(),
            "application/pdf"
        );
    }
}
