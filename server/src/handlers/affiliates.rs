//! Affiliate registration (Issue #1150)
//!
//! Lets a wallet register as an affiliate for an event and receive a unique
//! referral code. Purchases arriving with that code are attributed back to the
//! affiliate through `transactions.referred_by` (Issue #1151).

use axum::{
    extract::{Path, State},
    http::HeaderMap,
    response::{IntoResponse, Response},
};
use rand::Rng;
use serde::Serialize;
use sqlx::PgPool;
use uuid::Uuid;

use crate::handlers::auth::extract_auth;
use crate::models::event_affiliate::EventAffiliate;
use crate::utils::error::AppError;
use crate::utils::response::success;

#[derive(Clone)]
pub struct AffiliateState {
    pub pool: PgPool,
}

/// Crockford-style base32 without `I`, `L`, `O` or `U`: avoids characters that
/// are misread when a code is copied off a poster or read aloud.
const CODE_ALPHABET: &[u8] = b"0123456789ABCDEFGHJKMNPQRSTVWXYZ";
const CODE_LENGTH: usize = 10;

/// How many times to retry on a code collision before giving up.
///
/// With a 32-character alphabet over 10 characters the keyspace is 32^10, so a
/// collision is already vanishingly unlikely; the retry exists so that the
/// unlikely case degrades into a second attempt rather than a 500.
const MAX_CODE_ATTEMPTS: usize = 5;

fn generate_referral_code() -> String {
    let mut rng = rand::thread_rng();
    (0..CODE_LENGTH)
        .map(|_| CODE_ALPHABET[rng.gen_range(0..CODE_ALPHABET.len())] as char)
        .collect()
}

#[derive(Debug, Serialize)]
pub struct AffiliateResponse {
    pub id: Uuid,
    pub event_id: Uuid,
    pub wallet_address: String,
    pub referral_code: String,
    /// True when this wallet was already registered for the event and the
    /// existing code is being returned rather than a new one issued.
    pub already_registered: bool,
}

impl AffiliateResponse {
    fn from_row(row: EventAffiliate, already_registered: bool) -> Self {
        Self {
            id: row.id,
            event_id: row.event_id,
            wallet_address: row.wallet_address,
            referral_code: row.referral_code,
            already_registered,
        }
    }
}

/// POST /api/v1/events/:id/affiliates
///
/// Registers the authenticated wallet as an affiliate for the event and
/// returns its referral code.
///
/// Registration is idempotent: a wallet that is already registered gets its
/// existing code back with `already_registered: true`, rather than a second
/// registration or an error. Re-issuing a new code would silently invalidate
/// links the affiliate has already shared.
pub async fn register_affiliate(
    State(state): State<AffiliateState>,
    headers: HeaderMap,
    Path(event_id): Path<String>,
) -> Response {
    let wallet_address = match extract_auth(&headers) {
        Ok(address) => address,
        Err(e) => return e.into_response(),
    };

    let event_uuid = match Uuid::parse_str(&event_id) {
        Ok(id) => id,
        Err(_) => {
            return AppError::ValidationError(format!("Invalid event id: {event_id}"))
                .into_response()
        }
    };

    // The event must exist before a code is minted for it, otherwise a typo in
    // the URL yields a code that can never attribute anything.
    match sqlx::query_scalar::<_, Uuid>("SELECT id FROM events WHERE id = $1")
        .bind(event_uuid)
        .fetch_optional(&state.pool)
        .await
    {
        Ok(Some(_)) => {}
        Ok(None) => {
            return AppError::NotFound(format!("Event {event_id} not found")).into_response()
        }
        Err(e) => return AppError::DatabaseError(e).into_response(),
    }

    // Return the existing registration if there is one, so a repeated call is
    // a no-op rather than a duplicate.
    match sqlx::query_as::<_, EventAffiliate>(
        "SELECT id, event_id, wallet_address, referral_code, created_at, updated_at \
         FROM event_affiliates WHERE event_id = $1 AND wallet_address = $2",
    )
    .bind(event_uuid)
    .bind(&wallet_address)
    .fetch_optional(&state.pool)
    .await
    {
        Ok(Some(existing)) => {
            return success(
                AffiliateResponse::from_row(existing, true),
                "Already registered as an affiliate for this event",
            )
            .into_response()
        }
        Ok(None) => {}
        Err(e) => return AppError::DatabaseError(e).into_response(),
    }

    for _ in 0..MAX_CODE_ATTEMPTS {
        let referral_code = generate_referral_code();

        let inserted = sqlx::query_as::<_, EventAffiliate>(
            "INSERT INTO event_affiliates (event_id, wallet_address, referral_code) \
             VALUES ($1, $2, $3) \
             RETURNING id, event_id, wallet_address, referral_code, created_at, updated_at",
        )
        .bind(event_uuid)
        .bind(&wallet_address)
        .bind(&referral_code)
        .fetch_one(&state.pool)
        .await;

        match inserted {
            Ok(row) => {
                return success(
                    AffiliateResponse::from_row(row, false),
                    "Registered as an affiliate",
                )
                .into_response()
            }
            Err(sqlx::Error::Database(db_err)) if db_err.constraint().is_some() => {
                match db_err.constraint() {
                    // Another request registered this wallet between our read
                    // and our write. Return that registration instead of
                    // failing: the caller wanted a code, and there is one.
                    Some("event_affiliates_event_wallet_key") => {
                        return match sqlx::query_as::<_, EventAffiliate>(
                            "SELECT id, event_id, wallet_address, referral_code, created_at, updated_at \
                             FROM event_affiliates WHERE event_id = $1 AND wallet_address = $2",
                        )
                        .bind(event_uuid)
                        .bind(&wallet_address)
                        .fetch_one(&state.pool)
                        .await
                        {
                            Ok(row) => success(
                                AffiliateResponse::from_row(row, true),
                                "Already registered as an affiliate for this event",
                            )
                            .into_response(),
                            Err(e) => AppError::DatabaseError(e).into_response(),
                        }
                    }
                    // Code collision — mint another and try again.
                    _ => continue,
                }
            }
            Err(e) => {
                tracing::error!("Failed to register affiliate: {:?}", e);
                return AppError::DatabaseError(e).into_response();
            }
        }
    }

    AppError::Conflict(
        "Could not allocate a unique referral code, please retry".to_string(),
    )
    .into_response()
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::collections::HashSet;

    #[test]
    fn referral_code_has_the_documented_shape() {
        let code = generate_referral_code();
        assert_eq!(code.len(), CODE_LENGTH);
        assert!(
            code.bytes().all(|b| CODE_ALPHABET.contains(&b)),
            "code {code} contains a character outside the alphabet"
        );
    }

    #[test]
    fn referral_code_avoids_ambiguous_characters() {
        // I, L, O and U are excluded so a code stays unambiguous when read off
        // a poster or read aloud.
        for ambiguous in [b'I', b'L', b'O', b'U'] {
            assert!(
                !CODE_ALPHABET.contains(&ambiguous),
                "{} must not be in the alphabet",
                ambiguous as char
            );
        }
    }

    #[test]
    fn referral_codes_do_not_repeat_in_practice() {
        // Not a uniqueness guarantee — that is the database's job via the
        // UNIQUE constraint. This only catches a generator that is constant or
        // seeded identically per call.
        let codes: HashSet<String> = (0..1_000).map(|_| generate_referral_code()).collect();
        assert_eq!(codes.len(), 1_000);
    }

    #[test]
    fn alphabet_is_exactly_base32() {
        assert_eq!(CODE_ALPHABET.len(), 32);
        let unique: HashSet<u8> = CODE_ALPHABET.iter().copied().collect();
        assert_eq!(unique.len(), 32, "alphabet must not repeat a character");
    }

    #[test]
    fn response_reports_whether_registration_already_existed() {
        let row = EventAffiliate {
            id: Uuid::nil(),
            event_id: Uuid::nil(),
            wallet_address: "GA_TEST".to_string(),
            referral_code: "K3M9TQ7XZ2".to_string(),
            created_at: chrono::Utc::now(),
            updated_at: chrono::Utc::now(),
        };

        let fresh = AffiliateResponse::from_row(row.clone(), false);
        assert!(!fresh.already_registered);
        assert_eq!(fresh.referral_code, "K3M9TQ7XZ2");

        let repeat = AffiliateResponse::from_row(row, true);
        assert!(repeat.already_registered);
        // A repeat registration must return the same code, not a new one.
        assert_eq!(repeat.referral_code, "K3M9TQ7XZ2");
    }

    #[test]
    fn response_serializes_the_documented_fields() {
        let resp = AffiliateResponse {
            id: Uuid::nil(),
            event_id: Uuid::nil(),
            wallet_address: "GA_TEST".to_string(),
            referral_code: "K3M9TQ7XZ2".to_string(),
            already_registered: false,
        };
        let json = serde_json::to_value(&resp).expect("serializes");
        for field in [
            "id",
            "event_id",
            "wallet_address",
            "referral_code",
            "already_registered",
        ] {
            assert!(json.get(field).is_some(), "missing field {field}");
        }
    }
}
