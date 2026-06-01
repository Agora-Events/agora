//! # Organizer Profile Handler
//!
//! CRUD operations for organizer-specific metadata stored in `organizer_profiles`.
//!
//! ## Endpoints
//! - `GET  /api/v1/profile`        — fetch the authenticated organizer's profile
//! - `PUT  /api/v1/profile`        — create or update the authenticated organizer's profile
//! - `GET  /api/v1/profile/:addr`  — fetch any organizer's public profile by wallet address

use axum::{
    extract::{Path, State},
    http::HeaderMap,
    response::{IntoResponse, Response},
    Json,
};
use sqlx::PgPool;

use crate::handlers::auth::extract_auth;
use crate::models::organizer_profile::{OrganizerProfile, UpsertProfileRequest};
use crate::utils::error::AppError;
use crate::utils::response::success;

use serde::Serialize;
use once_cell::sync::Lazy;
use std::collections::HashMap;
use std::sync::Mutex;
use std::time::{Duration, Instant};

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
    State(pool): State<PgPool>,
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
    .fetch_one(&pool)
    .await
    {
        Ok(p) => p,
        Err(e) => {
            tracing::error!("Failed to upsert organizer profile: {:?}", e);
            return AppError::DatabaseError(e).into_response();
        }
    };

    success(profile, "Profile updated successfully").into_response()
}

/// `GET /api/v1/profile`
///
/// Returns the authenticated organizer's own profile.
/// Returns 404 if no profile has been created yet.
pub async fn get_my_profile(State(pool): State<PgPool>, headers: HeaderMap) -> Response {
    let address = match extract_auth(&headers) {
        Ok(a) => a,
        Err(e) => return e.into_response(),
    };

    fetch_profile_by_address(&pool, &address).await
}

/// `GET /api/v1/profile/:address`
///
/// Returns any organizer's public profile by their Stellar wallet address.
pub async fn get_profile_by_address(
    State(pool): State<PgPool>,
    Path(address): Path<String>,
) -> Response {
    fetch_profile_by_address(&pool, &address).await
}

async fn fetch_profile_by_address(pool: &PgPool, address: &str) -> Response {
    match sqlx::query_as::<_, OrganizerProfile>(
        "SELECT * FROM organizer_profiles WHERE address = $1",
    )
    .bind(address)
    .fetch_optional(pool)
    .await
    {
        Ok(Some(profile)) => success(profile, "Profile retrieved successfully").into_response(),
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

#[derive(Serialize, Clone)]
struct OrganizerStats {
    pub total_events: i64,
    pub total_tickets_sold: i64,
    pub average_event_rating: f64,
}

static STATS_CACHE: Lazy<Mutex<HashMap<String, (Instant, OrganizerStats)>>> =
    Lazy::new(|| Mutex::new(HashMap::new()));

const STATS_CACHE_TTL: Duration = Duration::from_secs(300); // 5 minutes

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
                return success(stats.clone(), "Organizer stats retrieved from cache").into_response();
            }
        }
    }

    // total events
    let total_events: i64 = match sqlx::query_scalar(
        "SELECT COUNT(*) FROM events WHERE organizer_wallet = $1",
    )
    .bind(&address)
    .fetch_one(&pool)
    .await
    {
        Ok(v) => v,
        Err(e) => {
            tracing::error!("Failed to query total_events: {:?}", e);
            return AppError::DatabaseError(e).into_response();
        }
    };

    // total tickets sold
    let total_tickets_sold: i64 = match sqlx::query_scalar(
        "SELECT COALESCE(SUM(minted_tickets), 0) FROM events WHERE organizer_wallet = $1",
    )
    .bind(&address)
    .fetch_one(&pool)
    .await
    {
        Ok(v) => v,
        Err(e) => {
            tracing::error!("Failed to query total_tickets_sold: {:?}", e);
            return AppError::DatabaseError(e).into_response();
        }
    };

    // average event rating
    let average_event_rating: f64 = match sqlx::query_scalar(
        "SELECT COALESCE(AVG(CAST(sum_of_ratings AS FLOAT) / NULLIF(count_of_ratings, 0)), 0) FROM events WHERE organizer_wallet = $1 AND count_of_ratings > 0",
    )
    .bind(&address)
    .fetch_one(&pool)
    .await
    {
        Ok(v) => v,
        Err(e) => {
            tracing::error!("Failed to query average_event_rating: {:?}", e);
            return AppError::DatabaseError(e).into_response();
        }
    };

    let stats = OrganizerStats {
        total_events,
        total_tickets_sold,
        average_event_rating,
    };

    // store in cache
    {
        let mut cache = STATS_CACHE.lock().unwrap();
        cache.insert(address.clone(), (Instant::now() + STATS_CACHE_TTL, stats.clone()));
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

    #[tokio::test]
    async fn test_get_organizer_stats_cache_hit() {
        use axum::extract::{State, Path};
        // create a lazy pool; it won't hit the DB because the cache will short-circuit
        let pool = sqlx::PgPool::connect_lazy("postgres://localhost/fake");
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
        let bytes = hyper::body::to_bytes(http.into_body()).await.unwrap();
        let v: serde_json::Value = serde_json::from_slice(&bytes).unwrap();
        assert!(v["success"].as_bool().unwrap());
        assert_eq!(v["data"]["total_events"].as_i64().unwrap(), 2);
        assert_eq!(v["data"]["total_tickets_sold"].as_i64().unwrap(), 100);
        assert!((v["data"]["average_event_rating"].as_f64().unwrap() - 4.5).abs() < 1e-6);
    }
}
