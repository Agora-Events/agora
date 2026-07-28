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
    Extension,
    Json,
};
use sqlx::PgPool;

use crate::config::Config;
use crate::handlers::auth::extract_auth;
use crate::models::organizer_profile::{OrganizerProfile, UpsertProfileRequest};
use crate::utils::error::AppError;
use crate::utils::response::success;

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
    Extension(config): Extension<Config>,
    headers: HeaderMap,
    Json(payload): Json<UpsertProfileRequest>,
) -> Response {
    // Authenticate
    let address = match extract_auth(&headers, &config) {
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
pub async fn get_my_profile(
    State(pool): State<PgPool>,
    Extension(config): Extension<Config>,
    headers: HeaderMap,
) -> Response {
    let address = match extract_auth(&headers, &config) {
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
}
