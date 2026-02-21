use axum::{
    extract::{Path, State},
    response::IntoResponse,
    Json,
};
use rust_decimal::Decimal;
use sqlx::PgPool;
use uuid::Uuid;

use crate::models::refund::{
    EventCancellationRequest, EventCancellationResponse, RefundRequest, RefundResponse,
};
use crate::models::ticket::TicketTier;
use crate::utils::error::AppError;
use crate::utils::response;

/// Request a refund for a single ticket
/// Checks if the ticket tier is refundable before processing
pub async fn request_guest_refund(
    State(pool): State<PgPool>,
    Path(ticket_id): Path<Uuid>,
    Json(request): Json<RefundRequest>,
) -> Result<impl IntoResponse, AppError> {
    // Verify ticket exists
    sqlx::query_as::<_, (Uuid,)>("SELECT ticket_tier_id FROM tickets WHERE id = $1")
        .bind(ticket_id)
        .fetch_optional(&pool)
        .await
        .map_err(AppError::DatabaseError)?
        .ok_or(AppError::NotFound("Ticket not found".to_string()))?;

    // Get the ticket tier and check refundability
    let ticket_tier = sqlx::query_as::<_, TicketTier>(
        "SELECT id, event_id, name, description, price, total_quantity, available_quantity, is_refundable, created_at, updated_at 
         FROM ticket_tiers WHERE id = $1"
    )
    .fetch_optional(&pool)
    .await
    .map_err(AppError::DatabaseError)?
    .ok_or(AppError::NotFound("Ticket tier not found".to_string()))?;

    // Check if the ticket tier is refundable
    if !ticket_tier.is_refundable {
        return Err(AppError::NonRefundableTicket);
    }

    // Create refund record
    let refund_id = Uuid::new_v4();
    sqlx::query(
        "INSERT INTO refunds (id, ticket_id, amount, reason, status, initiated_by, created_at, updated_at)
         VALUES ($1, $2, $3, $4, $5, $6, NOW(), NOW())"
    )
    .bind(refund_id)
    .bind(ticket_id)
    .bind(ticket_tier.price)
    .bind(&request.reason)
    .bind("pending")
    .bind("guest")
    .execute(&pool)
    .await
    .map_err(AppError::DatabaseError)?;

    // Update ticket status to cancelled
    sqlx::query("UPDATE tickets SET status = $1, updated_at = NOW() WHERE id = $2")
        .bind("cancelled")
        .bind(ticket_id)
        .execute(&pool)
        .await
        .map_err(AppError::DatabaseError)?;

    let refund_response = RefundResponse {
        id: refund_id,
        ticket_id,
        amount: ticket_tier.price,
        status: "pending".to_string(),
        message: "Refund request submitted successfully".to_string(),
    };

    Ok(response::success(
        refund_response,
        "Refund request processed",
    ))
}

/// Process refunds for an entire event cancellation
/// Bypasses is_refundable check since organizer is initiating the refund
pub async fn handle_event_cancellation(
    State(pool): State<PgPool>,
    Path(event_id): Path<Uuid>,
    Json(request): Json<EventCancellationRequest>,
) -> Result<impl IntoResponse, AppError> {
    // Get all tickets for the event
    #[derive(sqlx::FromRow)]
    struct TicketRow {
        id: Uuid,
        #[allow(dead_code)]
        status: String,
        price: Decimal,
    }

    let tickets = sqlx::query_as::<_, TicketRow>(
        "SELECT t.id, t.status, tt.price 
         FROM tickets t
         JOIN ticket_tiers tt ON t.ticket_tier_id = tt.id
         WHERE tt.event_id = $1 AND t.status != 'cancelled'",
    )
    .bind(event_id)
    .fetch_all(&pool)
    .await
    .map_err(AppError::DatabaseError)?;

    let mut total_refunded = Decimal::from(0);
    let refunds_processed = tickets.len() as i32;

    // Process refunds for all tickets regardless of refundability
    for ticket in tickets {
        let refund_id = Uuid::new_v4();
        let amount = ticket.price;

        sqlx::query(
            "INSERT INTO refunds (id, ticket_id, amount, reason, status, initiated_by, created_at, updated_at)
             VALUES ($1, $2, $3, $4, $5, $6, NOW(), NOW())"
        )
        .bind(refund_id)
        .bind(ticket.id)
        .bind(amount)
        .bind(&request.reason)
        .bind("completed")
        .bind("organizer")
        .execute(&pool)
        .await
        .map_err(AppError::DatabaseError)?;

        // Update ticket status to cancelled
        sqlx::query("UPDATE tickets SET status = $1, updated_at = NOW() WHERE id = $2")
            .bind("cancelled")
            .bind(ticket.id)
            .execute(&pool)
            .await
            .map_err(AppError::DatabaseError)?;

        total_refunded += amount;
    }

    // Mark event as cancelled
    sqlx::query("UPDATE events SET updated_at = NOW() WHERE id = $1")
        .bind(event_id)
        .execute(&pool)
        .await
        .map_err(AppError::DatabaseError)?;

    let response = EventCancellationResponse {
        event_id,
        refunds_processed,
        total_refunded,
        message: format!(
            "Event cancelled. {} refunds processed totalling {}",
            refunds_processed, total_refunded
        ),
    };

    Ok(response::success(response, "Event cancellation processed"))
}
