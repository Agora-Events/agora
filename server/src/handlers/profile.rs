//! # Organizer Profile Handler
//!
//! CRUD operations for organizer-specific metadata stored in `organizer_profiles`.
//!
//! ## Endpoints
//! - `GET  /api/v1/profile`              — fetch the authenticated organizer's profile
//! - `PUT  /api/v1/profile`              — create or update the authenticated organizer's profile
//! - `GET  /api/v1/profile/transactions` — paginated payment history for the authenticated wallet
//! - `GET  /api/v1/profile/:addr`        — fetch any organizer's public profile by wallet address

use axum::{
    extract::{Path, Query, State},
    http::HeaderMap,
    response::{IntoResponse, Response},
    Json,
};
use sqlx::{PgPool, Postgres, QueryBuilder};
use std::time::Duration;

use chrono::{DateTime, Utc};
use rust_decimal::Decimal;
use sqlx::FromRow;
use uuid::Uuid;

use crate::cache::RedisCache;
use crate::handlers::auth::extract_auth;
use crate::models::organizer_profile::{OrganizerProfile, UpsertProfileRequest};
use crate::utils::error::AppError;
use crate::utils::pagination::{PaginatedResponse, PaginationParams};
use crate::utils::response::success;

use once_cell::sync::Lazy;
use serde::{Deserialize, Serialize};
use serde_json::Value;
use std::collections::HashMap;
use std::sync::Mutex;
use std::time::Instant;

const PROFILE_CACHE_TTL: Duration = Duration::from_secs(600);

/// Application state for profile handlers that use Redis caching.
#[derive(Clone)]
pub struct ProfileState {
    pub pool: PgPool,
    pub redis: RedisCache,
}

// ---------------------------------------------------------------------------
// Validation helpers
// ---------------------------------------------------------------------------

const MAX_DISPLAY_NAME: usize = 50;
const MAX_BIO: usize = 500;

fn validate_upsert(req: &UpsertProfileRequest) -> Result<(), AppError> {
    if req.display_name.trim().is_empty() {
        return Err(AppError::ValidationError(
            "display_name is required".to_string(),
        ));
    }
    if req.display_name.len() > MAX_DISPLAY_NAME {
        return Err(AppError::ValidationError(format!(
            "display_name must be at most {MAX_DISPLAY_NAME} characters"
        )));
    }
    if let Some(ref bio) = req.bio {
        if bio.len() > MAX_BIO {
            return Err(AppError::ValidationError(format!(
                "bio must be at most {MAX_BIO} characters"
            )));
        }
    }
    Ok(())
}

fn validate_patch(req: &PatchProfileRequest) -> Result<(), AppError> {
    if req.display_name.is_none()
        && req.bio.is_none()
        && req.avatar_url.is_none()
        && req.socials.is_none()
    {
        return Err(AppError::ValidationError(
            "At least one profile field is required".to_string(),
        ));
    }

    if let Some(ref display_name) = req.display_name {
        if display_name.trim().is_empty() {
            return Err(AppError::ValidationError(
                "display_name cannot be empty".to_string(),
            ));
        }
        if display_name.len() > MAX_DISPLAY_NAME {
            return Err(AppError::ValidationError(format!(
                "display_name must be at most {MAX_DISPLAY_NAME} characters"
            )));
        }
    }

    if let Some(ref bio) = req.bio {
        if bio.len() > MAX_BIO {
            return Err(AppError::ValidationError(format!(
                "bio must be at most {MAX_BIO} characters"
            )));
        }
    }

    Ok(())
}

fn validate_profile_deletion(active_upcoming_events: i64) -> Result<(), AppError> {
    if active_upcoming_events > 0 {
        return Err(AppError::Conflict(
            "Organizer account cannot be deleted while active upcoming events exist. Cancel or end all events first.".to_string(),
        ));
    }
    Ok(())
}

/// Payload accepted by `PATCH /api/v1/profile`.
#[derive(Debug, Deserialize)]
pub struct PatchProfileRequest {
    #[serde(alias = "displayName")]
    pub display_name: Option<String>,
    pub bio: Option<String>,
    #[serde(alias = "avatarUrl")]
    pub avatar_url: Option<String>,
    pub socials: Option<Value>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OrganizerProfileResponse {
    #[serde(flatten)]
    pub profile: OrganizerProfile,
    pub total_events: i64,
}

fn organizer_total_events_query() -> &'static str {
    r#"
    SELECT COUNT(*)
    FROM events e
    JOIN organizers o ON e.organizer_id = o.id
    WHERE o.wallet_address = $1
      AND e.is_flagged = FALSE
    "#
}

// ---------------------------------------------------------------------------
// Handlers
// ---------------------------------------------------------------------------

/// `PUT /api/v1/profile`
///
/// Creates or updates the authenticated organizer's profile.
/// Requires a valid `Authorization: Bearer <jwt>` header.
///
/// # Validation
/// - `display_name`: required, max 50 chars
/// - `bio`: optional, max 500 chars
pub async fn upsert_profile(
    State(mut state): State<ProfileState>,
    headers: HeaderMap,
    Json(payload): Json<UpsertProfileRequest>,
) -> Response {
    // Authenticate
    let address = match extract_auth(&headers) {
        Ok(a) => a,
        Err(e) => return e.into_response(),
    };

    // Validate
    if let Err(e) = validate_upsert(&payload) {
        return e.into_response();
    }

    let profile = match sqlx::query_as::<_, OrganizerProfile>(
        r#"
        INSERT INTO organizer_profiles (address, display_name, bio, avatar_url, socials)
        VALUES ($1, $2, $3, $4, $5)
        ON CONFLICT (address) DO UPDATE
            SET display_name = EXCLUDED.display_name,
                bio          = EXCLUDED.bio,
                avatar_url   = EXCLUDED.avatar_url,
                socials      = EXCLUDED.socials,
                updated_at   = NOW()
        RETURNING *
        "#,
    )
    .bind(&address)
    .bind(payload.display_name.trim())
    .bind(payload.bio.as_deref())
    .bind(payload.avatar_url.as_deref())
    .bind(&payload.socials)
    .fetch_one(&state.pool)
    .await
    {
        Ok(p) => p,
        Err(e) => {
            tracing::error!("Failed to upsert organizer profile: {:?}", e);
            return AppError::DatabaseError(e).into_response();
        }
    };

    let cache_key = format!("profile:{address}");
    if let Err(e) = state.redis.delete(&cache_key).await {
        tracing::warn!("Failed to invalidate profile cache for {address}: {:?}", e);
    }

    success(profile, "Profile updated successfully").into_response()
}

/// `PATCH /api/v1/profile`
///
/// Partially updates the authenticated organizer's profile.
pub async fn patch_profile(
    State(mut state): State<ProfileState>,
    headers: HeaderMap,
    Json(payload): Json<PatchProfileRequest>,
) -> Response {
    let address = match extract_auth(&headers) {
        Ok(a) => a,
        Err(e) => return e.into_response(),
    };

    if let Err(e) = validate_patch(&payload) {
        return e.into_response();
    }

    let mut builder: QueryBuilder<Postgres> = QueryBuilder::new("UPDATE organizer_profiles SET ");
    {
        let mut separated = builder.separated(", ");

        if let Some(ref display_name) = payload.display_name {
            separated
                .push("display_name = ")
                .push_bind(display_name.trim().to_string());
        }
        if let Some(ref bio) = payload.bio {
            separated.push("bio = ").push_bind(bio);
        }
        if let Some(ref avatar_url) = payload.avatar_url {
            separated.push("avatar_url = ").push_bind(avatar_url);
        }
        if let Some(ref socials) = payload.socials {
            separated.push("socials = ").push_bind(socials);
        }

        separated.push("updated_at = NOW()");
    }
    builder.push(" WHERE address = ");
    builder.push_bind(&address);
    builder.push(" RETURNING *");

    let profile = match builder
        .build_query_as::<OrganizerProfile>()
        .fetch_optional(&state.pool)
        .await
    {
        Ok(Some(profile)) => profile,
        Ok(None) => {
            return AppError::NotFound(format!("No profile found for address '{address}'"))
                .into_response();
        }
        Err(e) => {
            tracing::error!("Failed to patch organizer profile: {:?}", e);
            return AppError::DatabaseError(e).into_response();
        }
    };

    let cache_key = format!("profile:{address}");
    if let Err(e) = state.redis.delete(&cache_key).await {
        tracing::warn!("Failed to invalidate profile cache for {address}: {:?}", e);
    }

    success(profile, "Profile updated successfully").into_response()
}

/// `DELETE /api/v1/profile`
///
/// Deletes the authenticated organizer's profile if there are no active upcoming events.
pub async fn delete_profile(
    State(mut state): State<ProfileState>,
    headers: HeaderMap,
) -> Response {
    let address = match extract_auth(&headers) {
        Ok(a) => a,
        Err(e) => return e.into_response(),
    };

    let active_events: i64 = match sqlx::query_scalar(
        r#"
        SELECT COUNT(*)
        FROM events e
        INNER JOIN organizers o ON e.organizer_id = o.id
        WHERE o.wallet_address = $1
          AND e.start_time > NOW()
        "#,
    )
    .bind(&address)
    .fetch_one(&state.pool)
    .await
    {
        Ok(count) => count,
        Err(e) => {
            tracing::error!("Failed to count active events for profile delete: {:?}", e);
            return AppError::DatabaseError(e).into_response();
        }
    };

    if let Err(e) = validate_profile_deletion(active_events) {
        return e.into_response();
    }

    if let Err(e) = sqlx::query("DELETE FROM organizer_profiles WHERE address = $1")
        .bind(&address)
        .execute(&state.pool)
        .await
    {
        tracing::error!("Failed to delete organizer profile: {:?}", e);
        return AppError::DatabaseError(e).into_response();
    }

    let cache_key = format!("profile:{address}");
    if let Err(e) = state.redis.delete(&cache_key).await {
        tracing::warn!("Failed to invalidate profile cache for {address}: {:?}", e);
    }

    success(serde_json::json!({}), "Profile deleted successfully").into_response()
}

/// `GET /api/v1/profile`
///
/// Returns the authenticated organizer's own profile.
/// Returns 404 if no profile has been created yet.
pub async fn get_my_profile(State(mut state): State<ProfileState>, headers: HeaderMap) -> Response {
    let address = match extract_auth(&headers) {
        Ok(a) => a,
        Err(e) => return e.into_response(),
    };

    fetch_profile_by_address(&state.pool, &mut state.redis, &address).await
}

/// Summary of a payment transaction returned by `GET /api/v1/profile/transactions`.
#[derive(Debug, Clone, Serialize, FromRow)]
pub struct TransactionSummary {
    pub id: Uuid,
    pub event_id: Option<Uuid>,
    pub amount: Decimal,
    pub status: String,
    pub created_at: DateTime<Utc>,
}

/// `GET /api/v1/profile/transactions`
///
/// Returns a paginated list of payment transactions for the authenticated wallet,
/// ordered by `created_at` descending. Requires a valid JWT.
pub async fn list_my_transactions(
    State(state): State<ProfileState>,
    headers: HeaderMap,
    Query(pagination): Query<PaginationParams>,
) -> Response {
    let address = match extract_auth(&headers) {
        Ok(a) => a,
        Err(e) => return e.into_response(),
    };

    let validated = pagination.validate();

    let total = match sqlx::query_scalar::<_, i64>(
        r#"
        SELECT COUNT(*)
        FROM transactions tr
        INNER JOIN tickets t ON tr.ticket_id = t.id
        WHERE t.buyer_wallet = $1
        "#,
    )
    .bind(&address)
    .fetch_one(&state.pool)
    .await
    {
        Ok(n) => n,
        Err(e) => {
            tracing::error!("Failed to count wallet transactions: {:?}", e);
            return AppError::DatabaseError(e).into_response();
        }
    };

    let items = match sqlx::query_as::<_, TransactionSummary>(
        r#"
        SELECT
            tr.id,
            COALESCE(t.event_id, tt.event_id) AS event_id,
            tr.amount,
            tr.status,
            tr.created_at
        FROM transactions tr
        INNER JOIN tickets t ON tr.ticket_id = t.id
        LEFT JOIN ticket_tiers tt ON t.ticket_tier_id = tt.id
        WHERE t.buyer_wallet = $1
        ORDER BY tr.created_at DESC
        LIMIT $2 OFFSET $3
        "#,
    )
    .bind(&address)
    .bind(validated.limit())
    .bind(validated.offset())
    .fetch_all(&state.pool)
    .await
    {
        Ok(rows) => rows,
        Err(e) => {
            tracing::error!("Failed to fetch wallet transactions: {:?}", e);
            return AppError::DatabaseError(e).into_response();
        }
    };

    let response = PaginatedResponse::new(items, validated, total);
    success(response, "Transactions retrieved successfully").into_response()
}

/// `GET /api/v1/profile/:address`
///
/// Returns any organizer's public profile by their Stellar wallet address.
pub async fn get_profile_by_address(
    State(mut state): State<ProfileState>,
    Path(address): Path<String>,
) -> Response {
    fetch_profile_by_address(&state.pool, &mut state.redis, &address).await
}

async fn fetch_profile_by_address(pool: &PgPool, redis: &mut RedisCache, address: &str) -> Response {
    let cache_key = format!("profile:{address}");

    if let Ok(Some(cached)) = redis.get::<OrganizerProfileResponse>(&cache_key).await {
        return success(cached, "Profile retrieved successfully").into_response();
    }

    match sqlx::query_as::<_, OrganizerProfile>(
        "SELECT * FROM organizer_profiles WHERE address = $1",
    )
    .bind(address)
    .fetch_optional(pool)
    .await
    {
        Ok(Some(profile)) => {
            let total_events: i64 = match sqlx::query_scalar(organizer_total_events_query())
                .bind(address)
                .fetch_one(pool)
                .await
            {
                Ok(count) => count,
                Err(e) => {
                    tracing::error!("Failed to count organizer events: {:?}", e);
                    return AppError::DatabaseError(e).into_response();
                }
            };

            let response = OrganizerProfileResponse {
                profile,
                total_events,
            };

            if let Err(e) = redis.set(&cache_key, &response, PROFILE_CACHE_TTL).await {
                tracing::warn!("Failed to cache profile for {address}: {:?}", e);
            }

            success(response, "Profile retrieved successfully").into_response()
        }
        Ok(None) => {
            AppError::NotFound(format!("No profile found for address '{address}'")).into_response()
        }
        Err(e) => {
            tracing::error!("Failed to fetch organizer profile: {:?}", e);
            AppError::DatabaseError(e).into_response()
        }
    }
}

// ---------------------------------------------------------------------------
// Organizer stats endpoint
// ---------------------------------------------------------------------------

#[derive(Serialize, Clone, FromRow)]
struct OrganizerStats {
    pub total_events: i64,
    pub total_tickets_sold: i64,
    pub average_event_rating: f64,
}

static STATS_CACHE: Lazy<Mutex<HashMap<String, (Instant, OrganizerStats)>>> =
    Lazy::new(|| Mutex::new(HashMap::new()));

const STATS_CACHE_TTL: Duration = Duration::from_secs(300); // 5 minutes

fn organizer_stats_query() -> &'static str {
    r#"
    SELECT
        COUNT(*) AS total_events,
        COALESCE(SUM(e.minted_tickets), 0) AS total_tickets_sold,
        COALESCE(AVG(CAST(e.sum_of_ratings AS FLOAT) / NULLIF(e.count_of_ratings, 0)), 0)
            AS average_event_rating
    FROM events e
    JOIN organizers o ON e.organizer_id = o.id
    WHERE o.wallet_address = $1
      AND e.is_flagged = FALSE
    "#
}

/// `GET /api/v1/profile/:address/stats`
///
/// Returns aggregate stats for an organizer: total events created, total tickets sold,
/// and average event rating. Cached in-process for 5 minutes to avoid repeated DB hits.
pub async fn get_organizer_stats(
    State(pool): State<PgPool>,
    Path(address): Path<String>,
) -> Response {
    // Check in-memory cache first
    {
        let cache = STATS_CACHE.lock().unwrap();
        if let Some((expiry, stats)) = cache.get(&address) {
            if Instant::now() < *expiry {
                return success(stats.clone(), "Organizer stats retrieved from cache")
                    .into_response();
            }
        }
    }

    let stats: OrganizerStats = match sqlx::query_as(organizer_stats_query())
        .bind(&address)
        .fetch_one(&pool)
        .await
    {
        Ok(v) => v,
        Err(e) => {
            tracing::error!("Failed to query organizer stats: {:?}", e);
            return AppError::DatabaseError(e).into_response();
        }
    };

    // store in cache
    {
        let mut cache = STATS_CACHE.lock().unwrap();
        cache.insert(
            address.clone(),
            (Instant::now() + STATS_CACHE_TTL, stats.clone()),
        );
    }

    success(stats, "Organizer stats retrieved successfully").into_response()
}

// ---------------------------------------------------------------------------
// Tests
// ---------------------------------------------------------------------------

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;

    #[test]
    fn test_validate_upsert_ok() {
        let req = UpsertProfileRequest {
            display_name: "Agora Events".to_string(),
            bio: Some("We run great events.".to_string()),
            avatar_url: None,
            socials: json!({}),
        };
        assert!(validate_upsert(&req).is_ok());
    }

    #[test]
    fn test_validate_upsert_display_name_too_long() {
        let req = UpsertProfileRequest {
            display_name: "A".repeat(51),
            bio: None,
            avatar_url: None,
            socials: json!({}),
        };
        let err = validate_upsert(&req).unwrap_err();
        assert!(matches!(err, AppError::ValidationError(_)));
    }

    #[test]
    fn test_validate_upsert_bio_too_long() {
        let req = UpsertProfileRequest {
            display_name: "Valid Name".to_string(),
            bio: Some("B".repeat(501)),
            avatar_url: None,
            socials: json!({}),
        };
        let err = validate_upsert(&req).unwrap_err();
        assert!(matches!(err, AppError::ValidationError(_)));
    }

    #[test]
    fn test_validate_upsert_empty_display_name() {
        let req = UpsertProfileRequest {
            display_name: "   ".to_string(),
            bio: None,
            avatar_url: None,
            socials: json!({}),
        };
        let err = validate_upsert(&req).unwrap_err();
        assert!(matches!(err, AppError::ValidationError(_)));
    }

    #[test]
    fn test_validate_upsert_bio_exactly_500() {
        let req = UpsertProfileRequest {
            display_name: "Valid".to_string(),
            bio: Some("B".repeat(500)),
            avatar_url: None,
            socials: json!({}),
        };
        assert!(validate_upsert(&req).is_ok());
    }

    #[test]
    fn test_validate_upsert_display_name_exactly_50() {
        let req = UpsertProfileRequest {
            display_name: "A".repeat(50),
            bio: None,
            avatar_url: None,
            socials: json!({}),
        };
        assert!(validate_upsert(&req).is_ok());
    }

    #[test]
    fn test_validate_patch_allows_partial_bio_update() {
        let req = PatchProfileRequest {
            display_name: None,
            bio: Some("Updated bio".to_string()),
            avatar_url: None,
            socials: None,
        };

        assert!(validate_patch(&req).is_ok());
    }

    #[test]
    fn test_validate_patch_rejects_empty_payload() {
        let req = PatchProfileRequest {
            display_name: None,
            bio: None,
            avatar_url: None,
            socials: None,
        };

        let err = validate_patch(&req).unwrap_err();
        assert!(matches!(err, AppError::ValidationError(_)));
    }

    #[test]
    fn test_validate_patch_rejects_empty_display_name() {
        let req = PatchProfileRequest {
            display_name: Some("   ".to_string()),
            bio: None,
            avatar_url: None,
            socials: None,
        };

        let err = validate_patch(&req).unwrap_err();
        assert!(matches!(err, AppError::ValidationError(_)));
    }

    #[test]
    fn test_validate_profile_deletion_allows_when_no_active_events() {
        assert!(validate_profile_deletion(0).is_ok());
    }

    #[test]
    fn test_validate_profile_deletion_rejects_when_active_events_exist() {
        let err = validate_profile_deletion(2).unwrap_err();
        assert!(matches!(err, AppError::Conflict(_)));
    }

    #[test]
    fn test_validate_patch_accepts_camel_case_aliases() {
        let req: PatchProfileRequest = serde_json::from_value(json!({
            "displayName": "Agora",
            "avatarUrl": "https://example.com/avatar.png"
        }))
        .unwrap();

        assert_eq!(req.display_name.as_deref(), Some("Agora"));
        assert_eq!(
            req.avatar_url.as_deref(),
            Some("https://example.com/avatar.png")
        );
    }

    #[test]
    fn test_profile_response_includes_total_events() {
        let now = chrono::Utc::now();
        let response = OrganizerProfileResponse {
            profile: OrganizerProfile {
                address: "GABC".to_string(),
                display_name: "Agora".to_string(),
                bio: None,
                avatar_url: None,
                socials: json!({}),
                created_at: now,
                updated_at: now,
            },
            total_events: 3,
        };

        let json = serde_json::to_value(response).unwrap();
        assert_eq!(json["address"], "GABC");
        assert_eq!(json["total_events"], 3);
    }

    #[test]
    fn test_profile_cache_key_format() {
        let address = "GTEST123WALLETADDRESS";
        let key = format!("profile:{address}");
        assert_eq!(key, "profile:GTEST123WALLETADDRESS");
        assert!(key.starts_with("profile:"));
    }

    #[test]
    fn test_profile_cache_ttl_is_10_minutes() {
        assert_eq!(PROFILE_CACHE_TTL.as_secs(), 600);
    }

    #[test]
    fn test_organizer_profile_response_deserializes() {
        let now = chrono::Utc::now();
        let response = OrganizerProfileResponse {
            profile: OrganizerProfile {
                address: "GABC".to_string(),
                display_name: "Agora".to_string(),
                bio: None,
                avatar_url: None,
                socials: json!({}),
                created_at: now,
                updated_at: now,
            },
            total_events: 5,
        };
        let json_str = serde_json::to_string(&response).unwrap();
        let decoded: OrganizerProfileResponse = serde_json::from_str(&json_str).unwrap();
        assert_eq!(decoded.profile.address, "GABC");
        assert_eq!(decoded.total_events, 5);
    }

    #[test]
    fn test_organizer_total_events_query_excludes_flagged_events() {
        let query = organizer_total_events_query();

        assert!(query.contains("JOIN organizers"));
        assert!(query.contains("wallet_address = $1"));
        assert!(query.contains("is_flagged = FALSE"));
    }

    #[test]
    fn test_organizer_stats_query_combines_aggregates() {
        let query = organizer_stats_query();

        assert_eq!(query.matches("SELECT").count(), 1);
        assert!(query.contains("COUNT(*) AS total_events"));
        assert!(query.contains("COALESCE(SUM(e.minted_tickets), 0) AS total_tickets_sold"));
        assert!(query.contains("AVG(CAST(e.sum_of_ratings AS FLOAT) / NULLIF(e.count_of_ratings, 0))"));
        assert!(query.contains("JOIN organizers"));
        assert!(query.contains("o.wallet_address = $1"));
        assert!(query.contains("e.is_flagged = FALSE"));
    }

    #[tokio::test]
    async fn test_get_organizer_stats_cache_hit() {
        use axum::extract::{Path, State};
        // create a lazy pool; it won't hit the DB because the cache will short-circuit
        let pool = sqlx::PgPool::connect_lazy("postgres://localhost/fake").unwrap();
        let addr = "TEST-ADDR".to_string();
        {
            let mut cache = STATS_CACHE.lock().unwrap();
            cache.insert(
                addr.clone(),
                (
                    Instant::now() + Duration::from_secs(300),
                    OrganizerStats {
                        total_events: 2,
                        total_tickets_sold: 100,
                        average_event_rating: 4.5,
                    },
                ),
            );
        }

        let resp = get_organizer_stats(State(pool), Path(addr.clone())).await;
        let http = resp.into_response();
        let bytes = axum::body::to_bytes(http.into_body(), 1024 * 1024)
            .await
            .unwrap();
        let v: serde_json::Value = serde_json::from_slice(&bytes).unwrap();
        assert!(v["success"].as_bool().unwrap());
        assert_eq!(v["data"]["total_events"].as_i64().unwrap(), 2);
        assert_eq!(v["data"]["total_tickets_sold"].as_i64().unwrap(), 100);
        assert!((v["data"]["average_event_rating"].as_f64().unwrap() - 4.5).abs() < 1e-6);
    }

    #[test]
    fn test_transaction_summary_serializes_required_fields() {
        let now = Utc::now();
        let event_id = Uuid::new_v4();
        let summary = TransactionSummary {
            id: Uuid::new_v4(),
            event_id: Some(event_id),
            amount: Decimal::new(2500, 2),
            status: "completed".to_string(),
            created_at: now,
        };

        let value = serde_json::to_value(&summary).unwrap();
        assert!(value.get("id").is_some());
        assert_eq!(value["event_id"], json!(event_id));
        assert_eq!(value["amount"], json!("25.00"));
        assert_eq!(value["status"], json!("completed"));
        assert!(value.get("created_at").is_some());
    }
}
