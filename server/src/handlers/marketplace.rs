//! # Secondary Ticket Market (Issue #1184)
//!
//! The off-chain half of the capped resale protocol. Money and ownership move
//! atomically on-chain in `contract/contracts/ticket_payment/src/resale.rs`;
//! this module handles the two things a Soroban contract cannot:
//!
//! 1. **A browsable index.** On-chain listings are keyed by `payment_id` with
//!    no iteration, so buyers cannot discover what is for sale by reading
//!    ledger state. Sellers mirror their confirmed listing here after the
//!    listing transaction lands.
//! 2. **Handing over the ticket secret.** Owning the ticket on-chain is not
//!    enough to get through the gate — the holder also needs the raw secret
//!    whose SHA-256 digest the contract stores as the payment's
//!    `ValidationHash`. That secret has to travel from seller to buyer.
//!
//! ## Threat model for the key handover
//!
//! The server is a **blind relay**. The seller's device seals the ticket
//! secret to the buyer's X25519 public key using NaCl `box`
//! (X25519 key agreement + XSalsa20-Poly1305), and uploads only the
//! ciphertext, the nonce, and the ephemeral public key. This process never
//! sees, and cannot derive, the plaintext: opening the box requires the
//! buyer's X25519 secret key, which never leaves their device.
//!
//! What this design does buy:
//!   * a database dump does not yield a single usable ticket secret;
//!   * a compromised server cannot mint working tickets for itself.
//!
//! What it deliberately does **not** claim:
//!   * the server chooses which public key it hands the seller, so it could
//!     substitute its own and mount a MITM. Buyers should pin the key they
//!     published; a future revision should have the buyer sign their X25519
//!     key with their Stellar key so the seller can verify it independently.
//!   * a malicious seller can keep a copy of the secret they sold. That is
//!     inherent to a shared-secret check-in scheme and is mitigated at the
//!     gate, not here — check-in is single-use per ticket.
//!
//! Envelope fields are validated for encoding and length but are otherwise
//! opaque; no endpoint here ever attempts to decrypt one.
//!
//! ## Endpoints
//! - `POST   /api/v1/marketplace/listings` — mirror a confirmed on-chain listing
//! - `GET    /api/v1/marketplace/listings` — browse active listings
//! - `GET    /api/v1/marketplace/listings/:payment_id` — read one listing
//! - `DELETE /api/v1/marketplace/listings/:payment_id` — mark a listing cancelled
//! - `POST   /api/v1/marketplace/listings/:payment_id/offers` — buyer makes an offer
//! - `GET    /api/v1/marketplace/listings/:payment_id/offers` — seller reads offers
//! - `POST   /api/v1/marketplace/listings/:payment_id/key-envelope` — seller seals the secret
//! - `GET    /api/v1/marketplace/listings/:payment_id/key-envelope` — buyer claims it
//! - `POST   /api/v1/marketplace/push-token` — register a device for sale alerts

use axum::{
    extract::{Path, Query, State},
    http::{HeaderMap, StatusCode},
    response::{IntoResponse, Response},
    Json,
};
use base64::{engine::general_purpose, Engine as _};
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use sqlx::PgPool;
use std::sync::Arc;

use crate::handlers::auth::extract_auth;
use crate::notifications::{Notification, NotificationService};
use crate::utils::error::AppError;
use crate::utils::response::success;

/// Basis-point denominator, mirroring the contract's `MAX_BPS`.
const MAX_BPS: i64 = 10_000;

/// Raw byte length of an X25519 public key.
const X25519_PUBLIC_KEY_LEN: usize = 32;
/// Raw byte length of an XSalsa20 nonce (NaCl `box`).
const BOX_NONCE_LEN: usize = 24;
/// Generous ceiling on the sealed payload. The plaintext is a 32-byte ticket
/// secret plus a small JSON envelope; anything near this is abuse.
const MAX_CIPHERTEXT_BYTES: usize = 4096;

/// Default page size for the browse feed.
const DEFAULT_LIMIT: i64 = 20;
const MAX_LIMIT: i64 = 100;

/// Shared state for the marketplace routes.
#[derive(Clone)]
pub struct MarketplaceState {
    pub pool: PgPool,
    /// Used to alert a seller when their listing sells. Best-effort — a
    /// delivery failure never fails the sale.
    pub notifications: Arc<NotificationService>,
}

// ---------------------------------------------------------------------------
// Row / DTO types
// ---------------------------------------------------------------------------

#[derive(Debug, Serialize, sqlx::FromRow)]
pub struct ResaleListingRow {
    pub payment_id: String,
    pub event_id: String,
    pub seller_wallet: String,
    pub price_stroops: i64,
    pub max_price_stroops: i64,
    pub royalty_bps: i32,
    pub status: String,
    pub buyer_wallet: Option<String>,
    pub listing_tx_hash: Option<String>,
    pub sale_tx_hash: Option<String>,
    pub sold_at: Option<DateTime<Utc>>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

/// A listing plus the figures a buyer needs to decide, derived rather than
/// stored so they can never drift from `price_stroops` / `royalty_bps`.
#[derive(Debug, Serialize)]
pub struct ResaleListingView {
    #[serde(flatten)]
    pub listing: ResaleListingRow,
    /// Portion of the price that goes to the organizer as a royalty.
    pub royalty_stroops: i64,
    /// What the seller actually receives (`price - royalty`).
    pub seller_proceeds_stroops: i64,
    /// Headroom left under the cap, in stroops. Zero when listed at the cap.
    pub headroom_stroops: i64,
}

impl From<ResaleListingRow> for ResaleListingView {
    fn from(listing: ResaleListingRow) -> Self {
        // Mirrors `resale::split_proceeds`: royalty rounds down, dust to seller.
        let royalty_stroops = listing
            .price_stroops
            .saturating_mul(listing.royalty_bps as i64)
            / MAX_BPS;
        let seller_proceeds_stroops = listing.price_stroops - royalty_stroops;
        let headroom_stroops = listing.max_price_stroops - listing.price_stroops;

        Self {
            listing,
            royalty_stroops,
            seller_proceeds_stroops,
            headroom_stroops,
        }
    }
}

#[derive(Debug, Serialize, sqlx::FromRow)]
pub struct ResaleOfferRow {
    pub id: uuid::Uuid,
    pub payment_id: String,
    pub buyer_wallet: String,
    pub buyer_public_key: String,
    pub offer_price_stroops: i64,
    pub status: String,
    pub created_at: DateTime<Utc>,
}

#[derive(Debug, Serialize, sqlx::FromRow)]
pub struct KeyEnvelopeRow {
    pub payment_id: String,
    pub buyer_wallet: String,
    pub ephemeral_public_key: String,
    pub nonce: String,
    pub ciphertext: String,
    pub claimed_at: Option<DateTime<Utc>>,
    pub created_at: DateTime<Utc>,
}

#[derive(Debug, Deserialize)]
pub struct CreateListingRequest {
    /// On-chain `payment_id` of the ticket being sold.
    pub payment_id: String,
    pub event_id: String,
    pub price_stroops: i64,
    /// Ceiling the contract validated the listing against, read back from the
    /// `ResaleListing` the contract returned.
    pub max_price_stroops: i64,
    pub royalty_bps: i32,
    /// Hash of the transaction that created the listing on-chain.
    pub listing_tx_hash: Option<String>,
}

#[derive(Debug, Deserialize)]
pub struct ListListingsQuery {
    /// Restrict to one event.
    pub event_id: Option<String>,
    /// Restrict to one seller — powers the "my listings" tab.
    pub seller_wallet: Option<String>,
    /// `active` (default), `cancelled`, or `sold`.
    pub status: Option<String>,
    pub limit: Option<i64>,
    pub offset: Option<i64>,
}

#[derive(Debug, Deserialize)]
pub struct CreateOfferRequest {
    /// Base64 X25519 public key (32 raw bytes) the seller should seal to.
    pub buyer_public_key: String,
    pub offer_price_stroops: i64,
}

#[derive(Debug, Deserialize)]
pub struct CreateKeyEnvelopeRequest {
    /// Wallet the envelope is sealed for. Must have an offer on this listing.
    pub buyer_wallet: String,
    /// Base64 X25519 public key of the seller's ephemeral sending keypair.
    pub ephemeral_public_key: String,
    /// Base64 24-byte nonce.
    pub nonce: String,
    /// Base64 NaCl box ciphertext.
    pub ciphertext: String,
    /// Hash of the settlement transaction, recorded against the listing.
    pub sale_tx_hash: Option<String>,
}

#[derive(Debug, Deserialize)]
pub struct RegisterPushTokenRequest {
    pub token: String,
    /// `ios`, `android`, or `web`.
    pub platform: String,
}

// ---------------------------------------------------------------------------
// Validation helpers
// ---------------------------------------------------------------------------

/// Decodes a base64 field and asserts an exact raw length.
///
/// Length is checked on the decoded bytes rather than the base64 text so that
/// padding variations cannot smuggle a wrong-sized key past the check.
fn validate_b64_exact(field: &str, value: &str, expected_len: usize) -> Result<(), AppError> {
    let decoded = general_purpose::STANDARD
        .decode(value)
        .map_err(|_| AppError::ValidationError(format!("{field} must be valid base64")))?;

    if decoded.len() != expected_len {
        return Err(AppError::ValidationError(format!(
            "{field} must decode to exactly {expected_len} bytes, got {}",
            decoded.len()
        )));
    }
    Ok(())
}

/// Decodes a base64 field and asserts a maximum raw length.
fn validate_b64_max(field: &str, value: &str, max_len: usize) -> Result<(), AppError> {
    let decoded = general_purpose::STANDARD
        .decode(value)
        .map_err(|_| AppError::ValidationError(format!("{field} must be valid base64")))?;

    if decoded.is_empty() {
        return Err(AppError::ValidationError(format!(
            "{field} must not be empty"
        )));
    }
    if decoded.len() > max_len {
        return Err(AppError::ValidationError(format!(
            "{field} must decode to at most {max_len} bytes, got {}",
            decoded.len()
        )));
    }
    Ok(())
}

fn validate_identifier(field: &str, value: &str) -> Result<(), AppError> {
    let trimmed = value.trim();
    if trimmed.is_empty() {
        return Err(AppError::ValidationError(format!("{field} is required")));
    }
    if trimmed.len() > 128 {
        return Err(AppError::ValidationError(format!(
            "{field} must be 128 characters or fewer"
        )));
    }
    Ok(())
}

/// Loads a listing or returns a 404.
async fn load_listing(pool: &PgPool, payment_id: &str) -> Result<ResaleListingRow, AppError> {
    sqlx::query_as::<_, ResaleListingRow>("SELECT * FROM resale_listings WHERE payment_id = $1")
        .bind(payment_id)
        .fetch_optional(pool)
        .await?
        .ok_or_else(|| AppError::NotFound(format!("No resale listing for ticket '{payment_id}'")))
}

/// Loads a listing and asserts the caller owns it.
async fn load_listing_as_seller(
    pool: &PgPool,
    payment_id: &str,
    caller: &str,
) -> Result<ResaleListingRow, AppError> {
    let listing = load_listing(pool, payment_id).await?;
    if listing.seller_wallet != caller {
        // Deliberately not 404: the listing is public, so hiding its existence
        // buys nothing, and a clear 403 is easier for clients to act on.
        return Err(AppError::Forbidden(
            "Only the seller of this listing can perform this action".to_string(),
        ));
    }
    Ok(listing)
}

// ---------------------------------------------------------------------------
// Listings
// ---------------------------------------------------------------------------

/// `POST /api/v1/marketplace/listings`
///
/// Mirrors a listing the caller has already created on-chain so it becomes
/// discoverable. The caller must be the seller; `price_stroops` is re-checked
/// against the cap the contract reported, so a client cannot advertise a price
/// the contract would refuse to settle.
///
/// Idempotent by `payment_id`: re-posting updates the listing in place, which
/// is what a seller re-listing a cancelled ticket does.
pub async fn create_listing(
    State(state): State<MarketplaceState>,
    headers: HeaderMap,
    Json(payload): Json<CreateListingRequest>,
) -> Response {
    let seller = match extract_auth(&headers) {
        Ok(a) => a,
        Err(e) => return e.into_response(),
    };

    if let Err(e) = validate_create_listing(&payload) {
        return e.into_response();
    }

    // A ticket already sold on this market cannot be re-listed by the same
    // seller — they no longer own it. Guarding here keeps a stale client from
    // resurrecting a settled listing.
    match sqlx::query_scalar::<_, String>(
        "SELECT status FROM resale_listings WHERE payment_id = $1 AND seller_wallet <> $2",
    )
    .bind(&payload.payment_id)
    .bind(&seller)
    .fetch_optional(&state.pool)
    .await
    {
        Ok(Some(_)) => {
            return AppError::Forbidden("This ticket is listed by a different seller".to_string())
                .into_response()
        }
        Ok(None) => {}
        Err(e) => return AppError::DatabaseError(e).into_response(),
    }

    let listing = match sqlx::query_as::<_, ResaleListingRow>(
        r#"
        INSERT INTO resale_listings (
            payment_id, event_id, seller_wallet, price_stroops,
            max_price_stroops, royalty_bps, listing_tx_hash, status
        )
        VALUES ($1, $2, $3, $4, $5, $6, $7, 'active')
        ON CONFLICT (payment_id) DO UPDATE
            SET event_id          = EXCLUDED.event_id,
                price_stroops     = EXCLUDED.price_stroops,
                max_price_stroops = EXCLUDED.max_price_stroops,
                royalty_bps       = EXCLUDED.royalty_bps,
                listing_tx_hash   = EXCLUDED.listing_tx_hash,
                status            = 'active',
                buyer_wallet      = NULL,
                sale_tx_hash      = NULL,
                sold_at           = NULL
        RETURNING *
        "#,
    )
    .bind(&payload.payment_id)
    .bind(&payload.event_id)
    .bind(&seller)
    .bind(payload.price_stroops)
    .bind(payload.max_price_stroops)
    .bind(payload.royalty_bps)
    .bind(payload.listing_tx_hash.as_deref())
    .fetch_one(&state.pool)
    .await
    {
        Ok(l) => l,
        Err(e) => {
            tracing::error!("Failed to create resale listing: {:?}", e);
            return AppError::DatabaseError(e).into_response();
        }
    };

    (StatusCode::CREATED, Json(ResaleListingView::from(listing))).into_response()
}

fn validate_create_listing(payload: &CreateListingRequest) -> Result<(), AppError> {
    validate_identifier("payment_id", &payload.payment_id)?;
    validate_identifier("event_id", &payload.event_id)?;

    if payload.price_stroops <= 0 {
        return Err(AppError::ValidationError(
            "price_stroops must be greater than zero".to_string(),
        ));
    }
    if payload.max_price_stroops <= 0 {
        return Err(AppError::ValidationError(
            "max_price_stroops must be greater than zero".to_string(),
        ));
    }
    // The contract is the authority on this, but rejecting it here gives the
    // seller an immediate, readable error instead of a silently unsellable
    // listing.
    if payload.price_stroops > payload.max_price_stroops {
        return Err(AppError::ValidationError(format!(
            "price_stroops ({}) exceeds the resale cap for this ticket ({})",
            payload.price_stroops, payload.max_price_stroops
        )));
    }
    if !(0..=MAX_BPS as i32).contains(&payload.royalty_bps) {
        return Err(AppError::ValidationError(
            "royalty_bps must be between 0 and 10000".to_string(),
        ));
    }
    Ok(())
}

/// `GET /api/v1/marketplace/listings`
///
/// Browse feed. Defaults to active listings, newest first.
pub async fn list_listings(
    State(state): State<MarketplaceState>,
    Query(params): Query<ListListingsQuery>,
) -> Response {
    let status = params.status.as_deref().unwrap_or("active");
    if !matches!(status, "active" | "cancelled" | "sold") {
        return AppError::ValidationError(
            "status must be one of: active, cancelled, sold".to_string(),
        )
        .into_response();
    }

    let limit = params.limit.unwrap_or(DEFAULT_LIMIT).clamp(1, MAX_LIMIT);
    let offset = params.offset.unwrap_or(0).max(0);

    // `$2 IS NULL OR` keeps this a single prepared statement instead of
    // concatenating SQL per filter combination.
    let listings = match sqlx::query_as::<_, ResaleListingRow>(
        r#"
        SELECT * FROM resale_listings
        WHERE status = $1
          AND ($2::TEXT IS NULL OR event_id = $2)
          AND ($3::TEXT IS NULL OR seller_wallet = $3)
        ORDER BY created_at DESC
        LIMIT $4 OFFSET $5
        "#,
    )
    .bind(status)
    .bind(params.event_id.as_deref())
    .bind(params.seller_wallet.as_deref())
    .bind(limit)
    .bind(offset)
    .fetch_all(&state.pool)
    .await
    {
        Ok(rows) => rows,
        Err(e) => {
            tracing::error!("Failed to list resale listings: {:?}", e);
            return AppError::DatabaseError(e).into_response();
        }
    };

    let views: Vec<ResaleListingView> = listings.into_iter().map(Into::into).collect();
    success(views, "Resale listings retrieved").into_response()
}

/// `GET /api/v1/marketplace/listings/:payment_id`
pub async fn get_listing(
    State(state): State<MarketplaceState>,
    Path(payment_id): Path<String>,
) -> Response {
    match load_listing(&state.pool, &payment_id).await {
        Ok(listing) => {
            success(ResaleListingView::from(listing), "Resale listing retrieved").into_response()
        }
        Err(e) => e.into_response(),
    }
}

/// `DELETE /api/v1/marketplace/listings/:payment_id`
///
/// Mirrors an on-chain `cancel_resale_listing`. Seller-only. A sold listing
/// cannot be cancelled — settlement already happened on-chain.
pub async fn cancel_listing(
    State(state): State<MarketplaceState>,
    headers: HeaderMap,
    Path(payment_id): Path<String>,
) -> Response {
    let seller = match extract_auth(&headers) {
        Ok(a) => a,
        Err(e) => return e.into_response(),
    };

    let listing = match load_listing_as_seller(&state.pool, &payment_id, &seller).await {
        Ok(l) => l,
        Err(e) => return e.into_response(),
    };

    if listing.status == "sold" {
        return AppError::Conflict(
            "This ticket has already been sold and cannot be unlisted".to_string(),
        )
        .into_response();
    }

    if let Err(e) =
        sqlx::query("UPDATE resale_listings SET status = 'cancelled' WHERE payment_id = $1")
            .bind(&payment_id)
            .execute(&state.pool)
            .await
    {
        tracing::error!("Failed to cancel resale listing: {:?}", e);
        return AppError::DatabaseError(e).into_response();
    }

    // Outstanding offers are meaningless once the listing is gone.
    if let Err(e) = sqlx::query(
        "UPDATE resale_offers SET status = 'declined' WHERE payment_id = $1 AND status = 'pending'",
    )
    .bind(&payment_id)
    .execute(&state.pool)
    .await
    {
        tracing::error!("Failed to decline offers for cancelled listing: {:?}", e);
    }

    success(payment_id, "Resale listing cancelled").into_response()
}

// ---------------------------------------------------------------------------
// Offers
// ---------------------------------------------------------------------------

/// `POST /api/v1/marketplace/listings/:payment_id/offers`
///
/// A buyer signals interest and publishes the X25519 public key the seller
/// should seal the ticket secret to. Re-posting updates the standing offer,
/// which is also how a buyer rotates their encryption key.
pub async fn create_offer(
    State(state): State<MarketplaceState>,
    headers: HeaderMap,
    Path(payment_id): Path<String>,
    Json(payload): Json<CreateOfferRequest>,
) -> Response {
    let buyer = match extract_auth(&headers) {
        Ok(a) => a,
        Err(e) => return e.into_response(),
    };

    if let Err(e) = validate_b64_exact(
        "buyer_public_key",
        &payload.buyer_public_key,
        X25519_PUBLIC_KEY_LEN,
    ) {
        return e.into_response();
    }
    if payload.offer_price_stroops <= 0 {
        return AppError::ValidationError(
            "offer_price_stroops must be greater than zero".to_string(),
        )
        .into_response();
    }

    let listing = match load_listing(&state.pool, &payment_id).await {
        Ok(l) => l,
        Err(e) => return e.into_response(),
    };

    if listing.status != "active" {
        return AppError::Conflict(format!(
            "This listing is {} and is not accepting offers",
            listing.status
        ))
        .into_response();
    }
    if listing.seller_wallet == buyer {
        return AppError::ValidationError(
            "You cannot make an offer on your own listing".to_string(),
        )
        .into_response();
    }
    // The contract settles at the listed price, so an offer below it can never
    // be accepted as-is. Rejecting it here avoids a dead-end in the UI.
    if payload.offer_price_stroops < listing.price_stroops {
        return AppError::ValidationError(format!(
            "Offer must be at least the listed price ({} stroops)",
            listing.price_stroops
        ))
        .into_response();
    }
    if payload.offer_price_stroops > listing.max_price_stroops {
        return AppError::ValidationError(format!(
            "Offer exceeds the resale cap for this ticket ({} stroops)",
            listing.max_price_stroops
        ))
        .into_response();
    }

    let offer = match sqlx::query_as::<_, ResaleOfferRow>(
        r#"
        INSERT INTO resale_offers (
            payment_id, buyer_wallet, buyer_public_key, offer_price_stroops
        )
        VALUES ($1, $2, $3, $4)
        ON CONFLICT (payment_id, buyer_wallet) DO UPDATE
            SET buyer_public_key    = EXCLUDED.buyer_public_key,
                offer_price_stroops = EXCLUDED.offer_price_stroops,
                status              = 'pending'
        RETURNING id, payment_id, buyer_wallet, buyer_public_key,
                  offer_price_stroops, status, created_at
        "#,
    )
    .bind(&payment_id)
    .bind(&buyer)
    .bind(&payload.buyer_public_key)
    .bind(payload.offer_price_stroops)
    .fetch_one(&state.pool)
    .await
    {
        Ok(o) => o,
        Err(e) => {
            tracing::error!("Failed to record resale offer: {:?}", e);
            return AppError::DatabaseError(e).into_response();
        }
    };

    (StatusCode::CREATED, Json(offer)).into_response()
}

/// `GET /api/v1/marketplace/listings/:payment_id/offers`
///
/// Seller-only: the buyer public keys on these rows are what the seller seals
/// to, and exposing the offer book publicly would leak bidding positions.
pub async fn list_offers(
    State(state): State<MarketplaceState>,
    headers: HeaderMap,
    Path(payment_id): Path<String>,
) -> Response {
    let seller = match extract_auth(&headers) {
        Ok(a) => a,
        Err(e) => return e.into_response(),
    };

    if let Err(e) = load_listing_as_seller(&state.pool, &payment_id, &seller).await {
        return e.into_response();
    }

    let offers = match sqlx::query_as::<_, ResaleOfferRow>(
        r#"
        SELECT id, payment_id, buyer_wallet, buyer_public_key,
               offer_price_stroops, status, created_at
        FROM resale_offers
        WHERE payment_id = $1
        ORDER BY offer_price_stroops DESC, created_at ASC
        "#,
    )
    .bind(&payment_id)
    .fetch_all(&state.pool)
    .await
    {
        Ok(rows) => rows,
        Err(e) => {
            tracing::error!("Failed to list resale offers: {:?}", e);
            return AppError::DatabaseError(e).into_response();
        }
    };

    success(offers, "Resale offers retrieved").into_response()
}

// ---------------------------------------------------------------------------
// E2EE key envelope
// ---------------------------------------------------------------------------

/// `POST /api/v1/marketplace/listings/:payment_id/key-envelope`
///
/// Seller-only. Called after `purchase_resale_ticket` settles on-chain: the
/// seller uploads the ticket secret sealed to the buyer's X25519 key, and the
/// listing flips to `sold`.
///
/// The three envelope fields are validated for encoding and size only. This
/// process cannot decrypt them and must not try.
pub async fn create_key_envelope(
    State(state): State<MarketplaceState>,
    headers: HeaderMap,
    Path(payment_id): Path<String>,
    Json(payload): Json<CreateKeyEnvelopeRequest>,
) -> Response {
    let seller = match extract_auth(&headers) {
        Ok(a) => a,
        Err(e) => return e.into_response(),
    };

    if let Err(e) = validate_envelope(&payload) {
        return e.into_response();
    }

    let listing = match load_listing_as_seller(&state.pool, &payment_id, &seller).await {
        Ok(l) => l,
        Err(e) => return e.into_response(),
    };

    if listing.status == "cancelled" {
        return AppError::Conflict(
            "This listing was cancelled; re-list the ticket before completing a sale".to_string(),
        )
        .into_response();
    }
    if payload.buyer_wallet == seller {
        return AppError::ValidationError(
            "The buyer and seller must be different wallets".to_string(),
        )
        .into_response();
    }

    // The buyer must have an offer on record — that offer is where their
    // public key came from, so without it there is nothing to seal against and
    // the envelope would be addressed to a key nobody published.
    match sqlx::query_scalar::<_, i64>(
        "SELECT COUNT(*) FROM resale_offers WHERE payment_id = $1 AND buyer_wallet = $2",
    )
    .bind(&payment_id)
    .bind(&payload.buyer_wallet)
    .fetch_one(&state.pool)
    .await
    {
        Ok(0) => {
            return AppError::ValidationError("That buyer has no offer on this listing".to_string())
                .into_response()
        }
        Ok(_) => {}
        Err(e) => return AppError::DatabaseError(e).into_response(),
    }

    // The envelope, the listing status and the offer statuses have to move
    // together: a sold listing with no envelope leaves the buyer holding a
    // ticket they cannot check in with.
    let mut tx = match state.pool.begin().await {
        Ok(t) => t,
        Err(e) => return AppError::DatabaseError(e).into_response(),
    };

    let envelope = match sqlx::query_as::<_, KeyEnvelopeRow>(
        r#"
        INSERT INTO resale_key_envelopes (
            payment_id, buyer_wallet, ephemeral_public_key, nonce, ciphertext
        )
        VALUES ($1, $2, $3, $4, $5)
        ON CONFLICT (payment_id, buyer_wallet) DO UPDATE
            SET ephemeral_public_key = EXCLUDED.ephemeral_public_key,
                nonce                = EXCLUDED.nonce,
                ciphertext           = EXCLUDED.ciphertext,
                claimed_at           = NULL
        RETURNING payment_id, buyer_wallet, ephemeral_public_key, nonce,
                  ciphertext, claimed_at, created_at
        "#,
    )
    .bind(&payment_id)
    .bind(&payload.buyer_wallet)
    .bind(&payload.ephemeral_public_key)
    .bind(&payload.nonce)
    .bind(&payload.ciphertext)
    .fetch_one(&mut *tx)
    .await
    {
        Ok(row) => row,
        Err(e) => {
            tracing::error!("Failed to store resale key envelope: {:?}", e);
            return AppError::DatabaseError(e).into_response();
        }
    };

    if let Err(e) = sqlx::query(
        r#"
        UPDATE resale_listings
        SET status       = 'sold',
            buyer_wallet = $2,
            sale_tx_hash = COALESCE($3, sale_tx_hash),
            sold_at      = NOW()
        WHERE payment_id = $1
        "#,
    )
    .bind(&payment_id)
    .bind(&payload.buyer_wallet)
    .bind(payload.sale_tx_hash.as_deref())
    .execute(&mut *tx)
    .await
    {
        tracing::error!("Failed to mark resale listing sold: {:?}", e);
        return AppError::DatabaseError(e).into_response();
    }

    if let Err(e) = sqlx::query(
        r#"
        UPDATE resale_offers
        SET status = CASE WHEN buyer_wallet = $2 THEN 'accepted' ELSE 'declined' END
        WHERE payment_id = $1
        "#,
    )
    .bind(&payment_id)
    .bind(&payload.buyer_wallet)
    .execute(&mut *tx)
    .await
    {
        tracing::error!("Failed to settle resale offers: {:?}", e);
        return AppError::DatabaseError(e).into_response();
    }

    if let Err(e) = tx.commit().await {
        tracing::error!("Failed to commit resale settlement: {:?}", e);
        return AppError::DatabaseError(e).into_response();
    }

    // The sale is durable at this point; notifying is best-effort.
    notify_ticket_sold(&state, &listing, &payload.buyer_wallet).await;

    (StatusCode::CREATED, Json(envelope)).into_response()
}

fn validate_envelope(payload: &CreateKeyEnvelopeRequest) -> Result<(), AppError> {
    validate_identifier("buyer_wallet", &payload.buyer_wallet)?;
    validate_b64_exact(
        "ephemeral_public_key",
        &payload.ephemeral_public_key,
        X25519_PUBLIC_KEY_LEN,
    )?;
    validate_b64_exact("nonce", &payload.nonce, BOX_NONCE_LEN)?;
    validate_b64_max("ciphertext", &payload.ciphertext, MAX_CIPHERTEXT_BYTES)?;
    Ok(())
}

/// `GET /api/v1/marketplace/listings/:payment_id/key-envelope`
///
/// Buyer-only. Returns the sealed secret addressed to the caller; the caller's
/// wallet is taken from their token, never from the request, so one buyer
/// cannot fetch another's envelope.
pub async fn get_key_envelope(
    State(state): State<MarketplaceState>,
    headers: HeaderMap,
    Path(payment_id): Path<String>,
) -> Response {
    let buyer = match extract_auth(&headers) {
        Ok(a) => a,
        Err(e) => return e.into_response(),
    };

    // Stamping `claimed_at` on read gives the seller confirmation the handover
    // landed. `COALESCE` keeps the first claim time rather than the latest.
    let envelope = match sqlx::query_as::<_, KeyEnvelopeRow>(
        r#"
        UPDATE resale_key_envelopes
        SET claimed_at = COALESCE(claimed_at, NOW())
        WHERE payment_id = $1 AND buyer_wallet = $2
        RETURNING payment_id, buyer_wallet, ephemeral_public_key, nonce,
                  ciphertext, claimed_at, created_at
        "#,
    )
    .bind(&payment_id)
    .bind(&buyer)
    .fetch_optional(&state.pool)
    .await
    {
        Ok(Some(row)) => row,
        Ok(None) => {
            return AppError::NotFound(
                "No key envelope is available for you on this ticket yet".to_string(),
            )
            .into_response()
        }
        Err(e) => {
            tracing::error!("Failed to fetch resale key envelope: {:?}", e);
            return AppError::DatabaseError(e).into_response();
        }
    };

    success(envelope, "Key envelope retrieved").into_response()
}

// ---------------------------------------------------------------------------
// Push tokens
// ---------------------------------------------------------------------------

/// `POST /api/v1/marketplace/push-token`
///
/// Registers this device against the caller's wallet so they can be told when
/// their listing sells. Tokens are unique, so re-registering a token that has
/// moved to a different wallet reassigns it.
pub async fn register_push_token(
    State(state): State<MarketplaceState>,
    headers: HeaderMap,
    Json(payload): Json<RegisterPushTokenRequest>,
) -> Response {
    let wallet = match extract_auth(&headers) {
        Ok(a) => a,
        Err(e) => return e.into_response(),
    };

    if let Err(e) = validate_identifier("token", &payload.token) {
        return e.into_response();
    }
    if !matches!(payload.platform.as_str(), "ios" | "android" | "web") {
        return AppError::ValidationError("platform must be one of: ios, android, web".to_string())
            .into_response();
    }

    if let Err(e) = sqlx::query(
        r#"
        INSERT INTO push_tokens (wallet_address, token, platform)
        VALUES ($1, $2, $3)
        ON CONFLICT (token) DO UPDATE
            SET wallet_address = EXCLUDED.wallet_address,
                platform       = EXCLUDED.platform
        "#,
    )
    .bind(&wallet)
    .bind(payload.token.trim())
    .bind(&payload.platform)
    .execute(&state.pool)
    .await
    {
        tracing::error!("Failed to register push token: {:?}", e);
        return AppError::DatabaseError(e).into_response();
    }

    success(wallet, "Push token registered").into_response()
}

/// Pushes a "your ticket sold" alert to every device the seller has registered.
///
/// Failures are logged and swallowed: the sale is already settled on-chain and
/// committed here, so an undeliverable push must not turn into a client error
/// that makes the seller think the sale failed.
async fn notify_ticket_sold(
    state: &MarketplaceState,
    listing: &ResaleListingRow,
    buyer_wallet: &str,
) {
    let tokens = match sqlx::query_scalar::<_, String>(
        "SELECT token FROM push_tokens WHERE wallet_address = $1",
    )
    .bind(&listing.seller_wallet)
    .fetch_all(&state.pool)
    .await
    {
        Ok(t) => t,
        Err(e) => {
            tracing::warn!("Could not load push tokens for resale seller: {:?}", e);
            return;
        }
    };

    if tokens.is_empty() {
        return;
    }

    let body = format!(
        "Your ticket sold for {} USDC to {}…{}.",
        format_stroops(listing.price_stroops),
        &buyer_wallet[..buyer_wallet.len().min(4)],
        &buyer_wallet[buyer_wallet.len().saturating_sub(4)..],
    );

    for token in tokens {
        let notification = Notification {
            recipient: token,
            subject: "Ticket sold".to_string(),
            body: body.clone(),
        };
        if let Err(e) = state.notifications.send(&notification).await {
            tracing::warn!("Resale sale notification failed: {:?}", e);
        }
    }
}

/// Renders stroops (7 decimal places) as a human-readable USDC amount.
fn format_stroops(stroops: i64) -> String {
    format!("{:.2}", stroops as f64 / 10_000_000.0)
}

#[cfg(test)]
mod tests {
    use super::*;

    fn b64(bytes: &[u8]) -> String {
        general_purpose::STANDARD.encode(bytes)
    }

    #[test]
    fn accepts_a_correctly_sized_x25519_key() {
        assert!(validate_b64_exact("k", &b64(&[7u8; 32]), X25519_PUBLIC_KEY_LEN).is_ok());
    }

    #[test]
    fn rejects_a_key_of_the_wrong_length() {
        // 31 bytes base64-encodes to a string that *looks* plausible, so the
        // check has to be on decoded length, not string length.
        assert!(validate_b64_exact("k", &b64(&[7u8; 31]), X25519_PUBLIC_KEY_LEN).is_err());
        assert!(validate_b64_exact("k", &b64(&[7u8; 33]), X25519_PUBLIC_KEY_LEN).is_err());
    }

    #[test]
    fn rejects_non_base64() {
        assert!(validate_b64_exact("k", "not base64!!", X25519_PUBLIC_KEY_LEN).is_err());
    }

    #[test]
    fn rejects_empty_and_oversized_ciphertext() {
        assert!(validate_b64_max("c", "", MAX_CIPHERTEXT_BYTES).is_err());
        assert!(validate_b64_max("c", &b64(&[0u8; 32]), MAX_CIPHERTEXT_BYTES).is_ok());
        assert!(validate_b64_max(
            "c",
            &b64(&vec![0u8; MAX_CIPHERTEXT_BYTES + 1]),
            MAX_CIPHERTEXT_BYTES
        )
        .is_err());
    }

    #[test]
    fn rejects_a_listing_priced_above_its_cap() {
        let payload = CreateListingRequest {
            payment_id: "pay-1".to_string(),
            event_id: "event-1".to_string(),
            price_stroops: 1_100_000_001,
            max_price_stroops: 1_100_000_000,
            royalty_bps: 500,
            listing_tx_hash: None,
        };
        assert!(validate_create_listing(&payload).is_err());
    }

    #[test]
    fn accepts_a_listing_priced_exactly_at_its_cap() {
        let payload = CreateListingRequest {
            payment_id: "pay-1".to_string(),
            event_id: "event-1".to_string(),
            price_stroops: 1_100_000_000,
            max_price_stroops: 1_100_000_000,
            royalty_bps: 500,
            listing_tx_hash: None,
        };
        assert!(validate_create_listing(&payload).is_ok());
    }

    #[test]
    fn derives_royalty_and_proceeds_the_same_way_the_contract_does() {
        let row = ResaleListingRow {
            payment_id: "pay-1".to_string(),
            event_id: "event-1".to_string(),
            seller_wallet: "GSELLER".to_string(),
            price_stroops: 1_100_000_000, // 110 USDC
            max_price_stroops: 1_100_000_000,
            royalty_bps: 500, // 5%
            status: "active".to_string(),
            buyer_wallet: None,
            listing_tx_hash: None,
            sale_tx_hash: None,
            sold_at: None,
            created_at: Utc::now(),
            updated_at: Utc::now(),
        };

        let view = ResaleListingView::from(row);
        assert_eq!(view.royalty_stroops, 55_000_000);
        assert_eq!(view.seller_proceeds_stroops, 1_045_000_000);
        assert_eq!(view.headroom_stroops, 0);
    }

    #[test]
    fn rounds_royalty_dust_in_the_sellers_favour() {
        // 1 stroop at 5% is 0.05, which must floor to 0 rather than round up —
        // matching `resale::split_proceeds`.
        let row = ResaleListingRow {
            payment_id: "pay-1".to_string(),
            event_id: "event-1".to_string(),
            seller_wallet: "GSELLER".to_string(),
            price_stroops: 1,
            max_price_stroops: 10,
            royalty_bps: 500,
            status: "active".to_string(),
            buyer_wallet: None,
            listing_tx_hash: None,
            sale_tx_hash: None,
            sold_at: None,
            created_at: Utc::now(),
            updated_at: Utc::now(),
        };

        let view = ResaleListingView::from(row);
        assert_eq!(view.royalty_stroops, 0);
        assert_eq!(view.seller_proceeds_stroops, 1);
        assert_eq!(view.headroom_stroops, 9);
    }
}
