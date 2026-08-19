//! # Governance Handlers (Issue #1176)
//!
//! Multi-Sig Escrow Lockup & DAO-Governed Fraud Dispute Mediation Protocol.
//!
//! Endpoints:
//! - `GET    /api/v1/admin/governance/disputes` — list disputes with filters
//! - `GET    /api/v1/admin/governance/disputes/:id` — fetch one dispute
//! - `POST   /api/v1/admin/governance/disputes` — open a new dispute
//! - `POST   /api/v1/admin/governance/disputes/:id/votes` — cast a mediation vote
//! - `GET    /api/v1/admin/governance/disputes/:id/votes` — list votes for a dispute
//! - `POST   /api/v1/admin/governance/disputes/:id/resolve` — admin resolves a dispute

use axum::{
    extract::{Path, Query, State},
    Json,
};
use chrono::Utc;
use serde::Deserialize;
use sqlx::PgPool;
use uuid::Uuid;

use crate::utils::error::AppError;
use crate::models::dispute::{Dispute, MediationVote};

/// Shared state for governance handlers.
#[derive(Clone)]
pub struct GovernanceState {
    pub pool: PgPool,
}

// ---------------------------------------------------------------------------
// Query / request DTOs
// ---------------------------------------------------------------------------

#[derive(Debug, Deserialize)]
pub struct DisputeFilters {
    pub event_id: Option<Uuid>,
    pub status: Option<String>,
    pub page: Option<u32>,
    pub page_size: Option<u32>,
}

#[derive(Debug, Deserialize)]
pub struct OpenDisputeRequest {
    pub event_id: Uuid,
    pub opened_by: Uuid,
}

#[derive(Debug, Deserialize)]
pub struct CastVoteRequest {
    pub voter_id: Uuid,
    pub vote: String,
}

// ---------------------------------------------------------------------------
// Handlers
// ---------------------------------------------------------------------------

/// GET /api/v1/admin/governance/disputes
pub async fn list_disputes(
    State(state): State<GovernanceState>,
    Query(params): Query<DisputeFilters>,
) -> Result<Json<Vec<Dispute>>, AppError> {
    let page = params.page.unwrap_or(1).max(1);
    let page_size = params.page_size.unwrap_or(20).clamp(1, 100);
    let offset = ((page - 1) * page_size) as i64;

    let mut sql = String::from("SELECT * FROM disputes");
    let mut conditions = Vec::new();
    let mut bind_idx = 1;

    if let Some(_event_id) = params.event_id {
        conditions.push(format!("event_id = ${}", bind_idx));
        bind_idx += 1;
    }
    if let Some(ref _status) = params.status {
        conditions.push(format!("status = ${}", bind_idx));
        bind_idx += 1;
    }

    if !conditions.is_empty() {
        sql.push_str(" WHERE ");
        sql.push_str(&conditions.join(" AND "));
    }

    sql.push_str(" ORDER BY created_at DESC LIMIT $");
    sql.push_str(&bind_idx.to_string());
    sql.push_str(" OFFSET $");
    sql.push_str(&(bind_idx + 1).to_string());

    let mut query_builder = sqlx::query_as::<_, Dispute>(&sql);

    if let Some(event_id) = params.event_id {
        query_builder = query_builder.bind(event_id);
    }
    if let Some(status) = params.status {
        query_builder = query_builder.bind(status);
    }

    let disputes = query_builder
        .bind(page_size as i64)
        .bind(offset)
        .fetch_all(&state.pool)
        .await?;

    Ok(Json(disputes))
}

/// GET /api/v1/admin/governance/disputes/:id
pub async fn get_dispute(
    State(state): State<GovernanceState>,
    Path(dispute_id): Path<Uuid>,
) -> Result<Json<Dispute>, AppError> {
    let dispute = sqlx::query_as::<_, Dispute>("SELECT * FROM disputes WHERE id = $1")
        .bind(dispute_id)
        .fetch_optional(&state.pool)
        .await?
        .ok_or_else(|| AppError::NotFound(format!("Dispute with id '{}' not found", dispute_id)))?;

    Ok(Json(dispute))
}

/// POST /api/v1/admin/governance/disputes
pub async fn open_dispute(
    State(state): State<GovernanceState>,
    Json(payload): Json<OpenDisputeRequest>,
) -> Result<Json<Dispute>, AppError> {
    let event_ended: bool = sqlx::query_scalar::<_, bool>(
        "SELECT end_time < NOW() FROM events WHERE id = $1",
    )
    .bind(payload.event_id)
    .fetch_optional(&state.pool)
    .await?
    .ok_or_else(|| AppError::NotFound(format!("Event with id '{}' not found", payload.event_id)))?;

    if !event_ended {
        return Err(AppError::ValidationError(
            "Cannot open a dispute for an event that has not ended".to_string(),
        ));
    }

    let total_eligible: i32 = sqlx::query_scalar::<_, Option<i32>>(
        "SELECT COALESCE(SUM(total_quantity), 0) FROM ticket_tiers WHERE event_id = $1",
    )
    .bind(payload.event_id)
    .fetch_one(&state.pool)
    .await?
    .unwrap_or(0);

    let dispute = sqlx::query_as::<_, Dispute>(
        r#"
        INSERT INTO disputes (event_id, opened_by, closes_at, total_eligible_tickets)
        VALUES ($1, $2, NOW() + INTERVAL '7 days', $3)
        RETURNING *
        "#,
    )
    .bind(payload.event_id)
    .bind(payload.opened_by)
    .bind(total_eligible)
    .fetch_one(&state.pool)
    .await?;

    Ok(Json(dispute))
}

/// POST /api/v1/admin/governance/disputes/:id/votes
pub async fn cast_vote(
    State(state): State<GovernanceState>,
    Path(dispute_id): Path<Uuid>,
    Json(payload): Json<CastVoteRequest>,
) -> Result<Json<MediationVote>, AppError> {
    if payload.vote != "buyer_favor" && payload.vote != "organizer_favor" {
        return Err(AppError::ValidationError(
            "Vote must be either 'buyer_favor' or 'organizer_favor'".to_string(),
        ));
    }

    let dispute = sqlx::query_as::<_, Dispute>("SELECT * FROM disputes WHERE id = $1")
        .bind(dispute_id)
        .fetch_optional(&state.pool)
        .await?
        .ok_or_else(|| AppError::NotFound(format!("Dispute with id '{}' not found", dispute_id)))?;

    if matches!(
        dispute.status.as_str(),
        "resolved_buyer" | "resolved_organizer" | "expired"
    ) {
        return Err(AppError::ValidationError(
            "Cannot vote on a resolved or expired dispute".to_string(),
        ));
    }

    let existing: (bool,) = sqlx::query_as(
        "SELECT EXISTS(SELECT 1 FROM mediation_votes WHERE dispute_id = $1 AND voter_id = $2)",
    )
    .bind(dispute_id)
    .bind(payload.voter_id)
    .fetch_one(&state.pool)
    .await?;

    if existing.0 {
        return Err(AppError::Conflict(
            "Voter has already cast a vote on this dispute".to_string(),
        ));
    }

    let vote = sqlx::query_as::<_, MediationVote>(
        r#"
        INSERT INTO mediation_votes (dispute_id, voter_id, vote)
        VALUES ($1, $2, $3)
        RETURNING *
        "#,
    )
    .bind(dispute_id)
    .bind(payload.voter_id)
    .bind(&payload.vote)
    .fetch_one(&state.pool)
    .await?;

    if dispute.status == "open" {
        sqlx::query("UPDATE disputes SET status = 'voting' WHERE id = $1")
            .bind(dispute_id)
            .execute(&state.pool)
            .await?;
    }

    if payload.vote == "buyer_favor" {
        sqlx::query("UPDATE disputes SET buyer_votes = buyer_votes + 1 WHERE id = $1")
            .bind(dispute_id)
            .execute(&state.pool)
            .await?;
    } else {
        sqlx::query("UPDATE disputes SET organizer_votes = organizer_votes + 1 WHERE id = $1")
            .bind(dispute_id)
            .execute(&state.pool)
            .await?;
    }

    Ok(Json(vote))
}

/// POST /api/v1/admin/governance/disputes/:id/resolve
pub async fn resolve_dispute(
    State(state): State<GovernanceState>,
    Path(dispute_id): Path<Uuid>,
) -> Result<Json<Dispute>, AppError> {
    let dispute = sqlx::query_as::<_, Dispute>("SELECT * FROM disputes WHERE id = $1")
        .bind(dispute_id)
        .fetch_optional(&state.pool)
        .await?
        .ok_or_else(|| AppError::NotFound(format!("Dispute with id '{}' not found", dispute_id)))?;

    if matches!(
        dispute.status.as_str(),
        "resolved_buyer" | "resolved_organizer"
    ) {
        return Err(AppError::Conflict(
            "Dispute has already been resolved".to_string(),
        ));
    }

    if dispute.status == "expired" {
        return Err(AppError::ValidationError(
            "Cannot resolve an expired dispute".to_string(),
        ));
    }

    if dispute.closes_at < Utc::now() {
        return Err(AppError::ValidationError(
            "Voting period for this dispute has expired".to_string(),
        ));
    }

    let quorum_threshold = (dispute.total_eligible_tickets as f64 * 0.30).ceil() as i32;
    let (ruling, status) = if dispute.buyer_votes > quorum_threshold {
        ("buyer_favor", "resolved_buyer")
    } else {
        ("organizer_favor", "resolved_organizer")
    };

    let resolved = sqlx::query_as::<_, Dispute>(
        r#"
        UPDATE disputes
        SET status = $1, ruling = $2, resolved_at = NOW()
        WHERE id = $3
        RETURNING *
        "#,
    )
    .bind(status)
    .bind(ruling)
    .bind(dispute_id)
    .fetch_one(&state.pool)
    .await?;

    Ok(Json(resolved))
}

/// GET /api/v1/admin/governance/disputes/:id/votes
pub async fn get_dispute_votes(
    State(state): State<GovernanceState>,
    Path(dispute_id): Path<Uuid>,
) -> Result<Json<Vec<MediationVote>>, AppError> {
    let votes = sqlx::query_as::<_, MediationVote>(
        "SELECT * FROM mediation_votes WHERE dispute_id = $1 ORDER BY voted_at DESC",
    )
    .bind(dispute_id)
    .fetch_all(&state.pool)
    .await?;

    Ok(Json(votes))
}

#[cfg(test)]
mod tests {
    use super::*;
    use axum::{
        body::Body,
        http::{Method, Request, StatusCode},
        routing::{get, post},
        Router,
    };
    use tower::ServiceExt;

    fn test_router() -> Router {
        Router::new()
            .route(
                "/api/v1/admin/governance/disputes",
                get(list_disputes),
            )
            .route(
                "/api/v1/admin/governance/disputes/:id",
                get(get_dispute),
            )
            .route(
                "/api/v1/admin/governance/disputes/:id/votes",
                get(get_dispute_votes),
            )
            .route(
                "/api/v1/admin/governance/disputes/:id/resolve",
                get(resolve_dispute),
            )
            .with_state(GovernanceState {
                pool: PgPool::connect_lazy("postgresql://localhost/test")
                    .expect("test pool"),
            })
    }

    async fn get_status(router: Router, path: &str) -> StatusCode {
        let req = Request::builder().uri(path).body(Body::empty()).unwrap();
        router.oneshot(req).await.unwrap().status()
    }

    #[tokio::test]
    async fn test_list_disputes_route_exists() {
        let router = test_router();
        assert_ne!(
            get_status(router, "/api/v1/admin/governance/disputes").await,
            StatusCode::NOT_FOUND
        );
    }

    #[tokio::test]
    async fn test_get_dispute_route_exists() {
        let router = test_router();
        assert_ne!(
            get_status(router, "/api/v1/admin/governance/disputes/00000000-0000-0000-0000-000000000000").await,
            StatusCode::NOT_FOUND
        );
    }

    #[tokio::test]
    async fn test_get_dispute_votes_route_exists() {
        let router = test_router();
        assert_ne!(
            get_status(router, "/api/v1/admin/governance/disputes/00000000-0000-0000-0000-000000000000/votes").await,
            StatusCode::NOT_FOUND
        );
    }

    #[tokio::test]
    async fn test_resolve_dispute_route_exists() {
        let router = test_router();
        assert_ne!(
            get_status(router, "/api/v1/admin/governance/disputes/00000000-0000-0000-0000-000000000000/resolve").await,
            StatusCode::NOT_FOUND
        );
    }

    #[tokio::test]
    async fn test_cast_vote_validation_rejects_invalid_vote_value() {
        use axum::body::Body;
        use axum::http::Method;
        use tower::ServiceExt;

        let router = Router::new()
            .route(
                "/api/v1/admin/governance/disputes/:id/votes",
                post(cast_vote),
            )
            .with_state(GovernanceState {
                pool: PgPool::connect_lazy("postgresql://localhost/test")
                    .expect("test pool"),
            });

        let body = serde_json::json!({
            "voter_id": "00000000-0000-0000-0000-000000000000",
            "vote": "invalid_vote"
        });

        let req = Request::builder()
            .method(Method::POST)
            .uri("/api/v1/admin/governance/disputes/00000000-0000-0000-0000-000000000000/votes")
            .header("content-type", "application/json")
            .body(Body::from(serde_json::to_vec(&body).unwrap()))
            .unwrap();

        let response = router.oneshot(req).await.unwrap();
        assert_eq!(response.status(), StatusCode::BAD_REQUEST);
    }
}
