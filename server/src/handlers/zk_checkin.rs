//! # Zero-Knowledge Check-In Handler (Issue #1186)
//!
//! The HTTP surface over [`crate::utils::zkp_verifier`]. That module is the
//! cryptography; this one is the lifecycle, the database, and the parts that
//! decide whether someone gets through the gate.
//!
//! ## Endpoints
//!
//! | Method | Path                                | Auth      | Description |
//! |--------|-------------------------------------|-----------|-------------|
//! | POST   | `/api/v1/admin/zk/commitments`      | admin     | Register a ticket commitment (Merkle leaf) |
//! | POST   | `/api/v1/admin/zk/buckets/seal`     | admin     | Freeze an anonymity set and publish its root |
//! | GET    | `/api/v1/zk/ring`                   | public    | Fetch a sealed anonymity set for proving |
//! | POST   | `/api/v1/zk/checkin`                | public    | Verify a proof and burn its nullifier |
//!
//! ## Why `GET /zk/ring` is public
//!
//! It returns the whole anonymity set. That looks like a leak and is the
//! opposite: the set must be public for the proof to mean anything. If the
//! attendee could not see all `n` commitments they could not build a ring, and
//! if the *verifier* accepted a ring it had not published, a malicious scanner
//! could hand over a one-element ring and learn exactly who was standing in
//! front of it. Commitments are unlinkable to identities without the secrets,
//! which never leave the attendee's device.
//!
//! ## What the server learns from a successful check-in
//!
//! The tier, the nullifier, the anonymity set size, and the time. Not the
//! ticket id, not the wallet, not the name, not the email. A nullifier is
//! deterministic per (secret, event, epoch), so it detects a second entry —
//! and it is unlinkable to any commitment, so it cannot be walked back to a
//! ticket even with full database access.
//!
//! ## Ordering: verify, then burn
//!
//! Nullifier insertion is the commit point and it is a single statement. Two
//! scanners racing the same ticket both reach `INSERT`; the primary key on
//! `(event_id, epoch, nullifier)` means exactly one succeeds and the loser
//! sees a unique violation, which is reported as a replay. No advisory locks,
//! no read-then-write window.

use std::time::Instant;

use axum::{
    extract::{Query, State},
    response::{IntoResponse, Response},
    Json,
};
use serde::{Deserialize, Serialize};
use sqlx::PgPool;
use tracing::{info, warn};

use crate::utils::error::AppError;
use crate::utils::response::success;
use crate::utils::zkp_verifier::{
    merkle_root_of_commitments, AttestationTier, ProofSystem, RingChaumPedersen, RingMember,
    VerificationContext, ZkpError, DEFAULT_EPOCH, MAX_RING_SIZE,
};

// ──────────────────────────────────────────────────────────────────────────────
// Constants
// ──────────────────────────────────────────────────────────────────────────────

/// Ceiling on a base64/hex proof body. `MAX_RING_SIZE` members encode to about
/// 24 KB raw and ~33 KB hex; this leaves headroom without letting a caller
/// stream megabytes into the parser.
const MAX_PROOF_ENCODED_BYTES: usize = 64 * 1024;

/// The active proof backend. Swapping this for a Groth16 implementation of
/// [`ProofSystem`] is the whole extent of the change on this side.
const PROOF_SYSTEM: RingChaumPedersen = RingChaumPedersen;

// ──────────────────────────────────────────────────────────────────────────────
// State
// ──────────────────────────────────────────────────────────────────────────────

/// Shared state for the ZK check-in endpoints.
#[derive(Clone)]
pub struct ZkCheckinState {
    /// Postgres pool backing the commitment registry and nullifier store.
    pub pool: PgPool,
}

impl ZkCheckinState {
    /// Builds the state from a pool.
    pub fn new(pool: PgPool) -> Self {
        Self { pool }
    }
}

// ──────────────────────────────────────────────────────────────────────────────
// Wire types
// ──────────────────────────────────────────────────────────────────────────────

/// Request body for registering a ticket commitment.
#[derive(Debug, Deserialize)]
pub struct RegisterCommitmentRequest {
    /// Event the ticket belongs to.
    pub event_id: String,
    /// Attribute tier: `general`, `age21plus`, or `vip`.
    pub tier: String,
    /// Compressed ristretto255 commitment, hex-encoded (64 chars).
    pub commitment: String,
    /// Anonymity bucket to place the commitment in. Defaults to 0.
    #[serde(default)]
    pub bucket_index: i32,
}

/// Response after registering a commitment.
#[derive(Debug, Serialize)]
pub struct RegisterCommitmentResponse {
    /// Event the commitment was filed under.
    pub event_id: String,
    /// Tier it attests to.
    pub tier: String,
    /// Bucket it landed in.
    pub bucket_index: i32,
    /// Commitments now in that bucket.
    pub bucket_size: i64,
    /// False when the commitment was already registered (idempotent replay).
    pub created: bool,
}

/// Request body for sealing an anonymity bucket.
#[derive(Debug, Deserialize)]
pub struct SealBucketRequest {
    /// Event whose bucket is being sealed.
    pub event_id: String,
    /// Tier of the bucket.
    pub tier: String,
    /// Bucket index to seal.
    #[serde(default)]
    pub bucket_index: i32,
}

/// Response after sealing a bucket.
#[derive(Debug, Serialize)]
pub struct SealBucketResponse {
    /// Event the bucket belongs to.
    pub event_id: String,
    /// Tier of the bucket.
    pub tier: String,
    /// Bucket index that was sealed.
    pub bucket_index: i32,
    /// Published Merkle root, hex-encoded.
    pub merkle_root: String,
    /// Number of commitments frozen into the set.
    pub commitment_count: i64,
}

/// Query parameters for fetching an anonymity set.
#[derive(Debug, Deserialize)]
pub struct RingQuery {
    /// Event to fetch the ring for.
    pub event_id: String,
    /// Tier to fetch.
    pub tier: String,
    /// Bucket index. Defaults to 0.
    #[serde(default)]
    pub bucket_index: i32,
}

/// A sealed anonymity set, everything the mobile prover needs.
#[derive(Debug, Serialize)]
pub struct RingResponse {
    /// Event the set belongs to.
    pub event_id: String,
    /// Tier the set attests to.
    pub tier: String,
    /// Bucket index.
    pub bucket_index: i32,
    /// Published Merkle root, hex-encoded.
    pub merkle_root: String,
    /// Nullifier epoch to derive `Ω` from.
    pub epoch: String,
    /// The commitments, hex-encoded, in the order the root was computed over.
    /// Order is load-bearing: a reordered ring produces a different root.
    pub commitments: Vec<String>,
}

/// Request body for a zero-knowledge check-in.
#[derive(Debug, Deserialize)]
pub struct ZkCheckinRequest {
    /// Event being entered.
    pub event_id: String,
    /// Tier being claimed.
    pub tier: String,
    /// Bucket the proof was built against.
    #[serde(default)]
    pub bucket_index: i32,
    /// Nullifier epoch. Defaults to [`DEFAULT_EPOCH`].
    #[serde(default)]
    pub epoch: Option<String>,
    /// The proof itself, hex-encoded.
    pub proof: String,
    /// Optional gate-device identifier, for operational reporting only.
    #[serde(default)]
    pub scanner_id: Option<String>,
}

/// Result of a zero-knowledge check-in.
#[derive(Debug, Serialize)]
pub struct ZkCheckinResponse {
    /// Whether the holder may enter.
    pub admitted: bool,
    /// Tier that was proven.
    pub tier: String,
    /// Size of the anonymity set the holder hid in — the real privacy
    /// denominator, surfaced so operators can see it rather than assume it.
    pub anonymity_set_size: i32,
    /// Nullifier that was burned, hex-encoded. Safe to display: it identifies
    /// the *entry*, not the person.
    pub nullifier: String,
    /// Server-side verification time in microseconds. The issue budgets 20 ms
    /// per scan; exposing the measurement makes that testable in production.
    pub verification_us: u64,
}

// ──────────────────────────────────────────────────────────────────────────────
// Helpers
// ──────────────────────────────────────────────────────────────────────────────

/// Parses a tier label, rejecting anything the schema would not accept.
fn parse_tier(label: &str) -> Result<AttestationTier, AppError> {
    AttestationTier::from_str_label(label).ok_or_else(|| {
        AppError::ValidationError(format!(
            "Unknown attestation tier '{label}'. Expected one of: general, age21plus, vip"
        ))
    })
}

/// Decodes a hex field of an exact expected byte length.
fn parse_hex_bytes(field: &str, value: &str, expected_len: usize) -> Result<Vec<u8>, AppError> {
    let bytes = hex::decode(value)
        .map_err(|_| AppError::ValidationError(format!("{field} must be hex-encoded")))?;
    if bytes.len() != expected_len {
        return Err(AppError::ValidationError(format!(
            "{field} must decode to {expected_len} bytes, got {}",
            bytes.len()
        )));
    }
    Ok(bytes)
}

/// Rejects a non-positive bucket index before it reaches the CHECK constraint.
fn validate_bucket_index(bucket_index: i32) -> Result<(), AppError> {
    if bucket_index < 0 {
        return Err(AppError::ValidationError(
            "bucket_index must be zero or greater".to_string(),
        ));
    }
    Ok(())
}

/// One row of the commitment registry.
#[derive(Debug, sqlx::FromRow)]
struct CommitmentRow {
    commitment: Vec<u8>,
}

/// A sealed bucket's published root.
#[derive(Debug, sqlx::FromRow)]
struct SealedBucketRow {
    merkle_root: Option<Vec<u8>>,
    commitment_count: i32,
}

/// Loads a bucket's commitments in the canonical order the root was built over.
async fn load_bucket_commitments(
    pool: &PgPool,
    event_id: &str,
    tier: AttestationTier,
    bucket_index: i32,
) -> Result<Vec<CommitmentRow>, AppError> {
    sqlx::query_as::<_, CommitmentRow>(
        r#"
        SELECT commitment
        FROM zk_ticket_commitments
        WHERE event_id = $1 AND tier = $2 AND bucket_index = $3
        ORDER BY id
        "#,
    )
    .bind(event_id)
    .bind(tier.as_str())
    .bind(bucket_index)
    .fetch_all(pool)
    .await
    .map_err(AppError::from)
}

/// Loads a sealed bucket, or fails if it is still filling.
async fn load_sealed_bucket(
    pool: &PgPool,
    event_id: &str,
    tier: AttestationTier,
    bucket_index: i32,
) -> Result<(Vec<u8>, i32), AppError> {
    let row = sqlx::query_as::<_, SealedBucketRow>(
        r#"
        SELECT merkle_root, commitment_count
        FROM zk_anonymity_buckets
        WHERE event_id = $1 AND tier = $2 AND bucket_index = $3
        "#,
    )
    .bind(event_id)
    .bind(tier.as_str())
    .bind(bucket_index)
    .fetch_optional(pool)
    .await?
    .ok_or_else(|| {
        AppError::NotFound(format!(
            "No anonymity bucket {bucket_index} for event '{event_id}' tier '{}'",
            tier.as_str()
        ))
    })?;

    // A bucket that is still filling has a moving root, so any proof against it
    // would be invalidated by the next commitment. Refuse rather than fail
    // mysteriously at the gate.
    let root = row.merkle_root.ok_or_else(|| {
        AppError::Conflict(format!(
            "Anonymity bucket {bucket_index} for event '{event_id}' is not sealed yet; \
             seal it before check-in opens"
        ))
    })?;

    Ok((root, row.commitment_count))
}

/// Maps a verifier error onto an HTTP error.
///
/// Everything cryptographic collapses to one opaque 401. Telling a caller
/// *which* step failed turns the gate into an oracle they can probe; the
/// structural errors are safe to describe because they are about the request
/// shape, not about secrets.
fn map_zkp_error(err: ZkpError) -> AppError {
    match err {
        ZkpError::MalformedProof | ZkpError::NonCanonicalEncoding => {
            AppError::ValidationError("Proof is malformed".to_string())
        }
        ZkpError::UnsupportedVersion(v) => {
            AppError::ValidationError(format!("Unsupported proof version {v}"))
        }
        ZkpError::RingSize { got, min, max } => AppError::ValidationError(format!(
            "Anonymity set of {got} is outside the permitted range {min}..={max} for this tier"
        )),
        ZkpError::RingMismatch | ZkpError::RootMismatch => AppError::Conflict(
            "Proof was built against a different anonymity set; refetch the ring".to_string(),
        ),
        ZkpError::DegenerateNullifier
        | ZkpError::VerificationFailed
        | ZkpError::WitnessOutOfRange(_)
        | ZkpError::WitnessMismatch => {
            AppError::AuthError("Zero-knowledge proof verification failed".to_string())
        }
    }
}

// ──────────────────────────────────────────────────────────────────────────────
// Handlers
// ──────────────────────────────────────────────────────────────────────────────

/// `POST /api/v1/admin/zk/commitments` — register a ticket commitment.
///
/// Idempotent: re-registering the same commitment reports `created: false`
/// rather than erroring, so a retried mint does not fail and does not put a
/// duplicate in the ring.
///
/// Refuses to touch a sealed bucket. Sealing publishes a root; admitting a
/// commitment afterwards would silently invalidate every proof already built
/// against that root.
pub async fn register_commitment(
    State(state): State<ZkCheckinState>,
    Json(payload): Json<RegisterCommitmentRequest>,
) -> Response {
    match register_commitment_inner(&state, payload).await {
        Ok(response) => success(response, "Commitment registered").into_response(),
        Err(err) => err.into_response(),
    }
}

async fn register_commitment_inner(
    state: &ZkCheckinState,
    payload: RegisterCommitmentRequest,
) -> Result<RegisterCommitmentResponse, AppError> {
    let tier = parse_tier(&payload.tier)?;
    validate_bucket_index(payload.bucket_index)?;
    let commitment = parse_hex_bytes("commitment", &payload.commitment, 32)?;

    // The commitment must be a real group element. Storing an undecodable one
    // would poison the whole bucket: every future proof against that ring would
    // fail at verification with no way to tell which leaf was bad.
    let member = RingMember::from_bytes(&commitment).map_err(map_zkp_error)?;
    let leaf = member.leaf();

    let mut tx = state.pool.begin().await?;

    // Refuse to grow a sealed set.
    let sealed: Option<Option<chrono::DateTime<chrono::Utc>>> = sqlx::query_scalar(
        r#"
        SELECT sealed_at FROM zk_anonymity_buckets
        WHERE event_id = $1 AND tier = $2 AND bucket_index = $3
        "#,
    )
    .bind(&payload.event_id)
    .bind(tier.as_str())
    .bind(payload.bucket_index)
    .fetch_optional(&mut *tx)
    .await?;
    if matches!(sealed, Some(Some(_))) {
        return Err(AppError::Conflict(format!(
            "Anonymity bucket {} for event '{}' is already sealed",
            payload.bucket_index, payload.event_id
        )));
    }

    // Cap the bucket so verification stays inside its latency budget.
    let existing: i64 = sqlx::query_scalar(
        r#"
        SELECT COUNT(*) FROM zk_ticket_commitments
        WHERE event_id = $1 AND tier = $2 AND bucket_index = $3
        "#,
    )
    .bind(&payload.event_id)
    .bind(tier.as_str())
    .bind(payload.bucket_index)
    .fetch_one(&mut *tx)
    .await?;
    if existing >= MAX_RING_SIZE as i64 {
        return Err(AppError::Conflict(format!(
            "Anonymity bucket {} is full ({MAX_RING_SIZE} commitments); use the next bucket",
            payload.bucket_index
        )));
    }

    let inserted: Option<i64> = sqlx::query_scalar(
        r#"
        INSERT INTO zk_ticket_commitments (event_id, tier, bucket_index, commitment, leaf_hash)
        VALUES ($1, $2, $3, $4, $5)
        ON CONFLICT (event_id, tier, commitment) DO NOTHING
        RETURNING id
        "#,
    )
    .bind(&payload.event_id)
    .bind(tier.as_str())
    .bind(payload.bucket_index)
    .bind(&commitment)
    .bind(leaf.as_slice())
    .fetch_optional(&mut *tx)
    .await?;

    // Track the bucket so it can be sealed later, even before it is full.
    sqlx::query(
        r#"
        INSERT INTO zk_anonymity_buckets (event_id, tier, bucket_index, commitment_count)
        VALUES ($1, $2, $3, 0)
        ON CONFLICT (event_id, tier, bucket_index) DO NOTHING
        "#,
    )
    .bind(&payload.event_id)
    .bind(tier.as_str())
    .bind(payload.bucket_index)
    .execute(&mut *tx)
    .await?;

    let bucket_size: i64 = sqlx::query_scalar(
        r#"
        SELECT COUNT(*) FROM zk_ticket_commitments
        WHERE event_id = $1 AND tier = $2 AND bucket_index = $3
        "#,
    )
    .bind(&payload.event_id)
    .bind(tier.as_str())
    .bind(payload.bucket_index)
    .fetch_one(&mut *tx)
    .await?;

    tx.commit().await?;

    Ok(RegisterCommitmentResponse {
        event_id: payload.event_id,
        tier: tier.as_str().to_string(),
        bucket_index: payload.bucket_index,
        bucket_size,
        created: inserted.is_some(),
    })
}

/// `POST /api/v1/admin/zk/buckets/seal` — freeze an anonymity set.
///
/// Computes the Merkle root over the bucket's commitments in `id` order and
/// publishes it. After this the set is immutable, which is what lets an
/// attendee build a proof that is still valid when they reach the gate.
///
/// Sealing is idempotent in the useful sense: sealing an already-sealed bucket
/// returns the same root rather than recomputing it, so a retry cannot
/// republish a different set.
pub async fn seal_bucket(
    State(state): State<ZkCheckinState>,
    Json(payload): Json<SealBucketRequest>,
) -> Response {
    match seal_bucket_inner(&state, payload).await {
        Ok(response) => success(response, "Anonymity bucket sealed").into_response(),
        Err(err) => err.into_response(),
    }
}

async fn seal_bucket_inner(
    state: &ZkCheckinState,
    payload: SealBucketRequest,
) -> Result<SealBucketResponse, AppError> {
    let tier = parse_tier(&payload.tier)?;
    validate_bucket_index(payload.bucket_index)?;

    // Already sealed? Return the published root untouched.
    let existing: Option<SealedBucketRow> = sqlx::query_as(
        r#"
        SELECT merkle_root, commitment_count
        FROM zk_anonymity_buckets
        WHERE event_id = $1 AND tier = $2 AND bucket_index = $3
        "#,
    )
    .bind(&payload.event_id)
    .bind(tier.as_str())
    .bind(payload.bucket_index)
    .fetch_optional(&state.pool)
    .await?;

    if let Some(SealedBucketRow {
        merkle_root: Some(root),
        commitment_count,
    }) = existing
    {
        return Ok(SealBucketResponse {
            event_id: payload.event_id,
            tier: tier.as_str().to_string(),
            bucket_index: payload.bucket_index,
            merkle_root: hex::encode(&root),
            commitment_count: commitment_count as i64,
        });
    }

    let rows =
        load_bucket_commitments(&state.pool, &payload.event_id, tier, payload.bucket_index).await?;

    // Sealing an empty or too-small bucket would publish a root nobody can
    // prove against, so catch it here rather than at the gate.
    let min_ring = tier.min_ring();
    if rows.len() < min_ring {
        return Err(AppError::ValidationError(format!(
            "Bucket holds {} commitments; tier '{}' requires at least {min_ring} for the \
             anonymity guarantee to be meaningful",
            rows.len(),
            tier.as_str()
        )));
    }

    let mut commitments = Vec::with_capacity(rows.len());
    for row in &rows {
        let member = RingMember::from_bytes(&row.commitment).map_err(map_zkp_error)?;
        commitments.push(member.compressed);
    }
    let root = merkle_root_of_commitments(&commitments);

    sqlx::query(
        r#"
        UPDATE zk_anonymity_buckets
        SET merkle_root = $4, commitment_count = $5, sealed_at = NOW()
        WHERE event_id = $1 AND tier = $2 AND bucket_index = $3
          AND sealed_at IS NULL
        "#,
    )
    .bind(&payload.event_id)
    .bind(tier.as_str())
    .bind(payload.bucket_index)
    .bind(root.as_slice())
    .bind(commitments.len() as i32)
    .execute(&state.pool)
    .await?;

    info!(
        event_id = %payload.event_id,
        tier = tier.as_str(),
        bucket_index = payload.bucket_index,
        commitment_count = commitments.len(),
        "sealed ZK anonymity bucket"
    );

    Ok(SealBucketResponse {
        event_id: payload.event_id,
        tier: tier.as_str().to_string(),
        bucket_index: payload.bucket_index,
        merkle_root: hex::encode(root),
        commitment_count: commitments.len() as i64,
    })
}

/// `GET /api/v1/zk/ring` — fetch a sealed anonymity set.
///
/// The mobile prover calls this once, caches the result, and can then build
/// proofs entirely offline. See the module docs for why this is public.
pub async fn get_ring(
    State(state): State<ZkCheckinState>,
    Query(query): Query<RingQuery>,
) -> Response {
    match get_ring_inner(&state, query).await {
        Ok(response) => success(response, "Anonymity set retrieved").into_response(),
        Err(err) => err.into_response(),
    }
}

async fn get_ring_inner(
    state: &ZkCheckinState,
    query: RingQuery,
) -> Result<RingResponse, AppError> {
    let tier = parse_tier(&query.tier)?;
    validate_bucket_index(query.bucket_index)?;

    let (root, _) =
        load_sealed_bucket(&state.pool, &query.event_id, tier, query.bucket_index).await?;
    let rows =
        load_bucket_commitments(&state.pool, &query.event_id, tier, query.bucket_index).await?;

    Ok(RingResponse {
        event_id: query.event_id,
        tier: tier.as_str().to_string(),
        bucket_index: query.bucket_index,
        merkle_root: hex::encode(root),
        epoch: DEFAULT_EPOCH.to_string(),
        commitments: rows.iter().map(|r| hex::encode(&r.commitment)).collect(),
    })
}

/// `POST /api/v1/zk/checkin` — verify a proof and admit the holder.
///
/// The hot path. Budget is 20 ms of verification per scan; the response
/// reports the measured time so that budget is observable in production
/// rather than assumed.
pub async fn zk_checkin(
    State(state): State<ZkCheckinState>,
    Json(payload): Json<ZkCheckinRequest>,
) -> Response {
    match zk_checkin_inner(&state, payload).await {
        Ok(response) => success(response, "Admitted").into_response(),
        Err(err) => err.into_response(),
    }
}

async fn zk_checkin_inner(
    state: &ZkCheckinState,
    payload: ZkCheckinRequest,
) -> Result<ZkCheckinResponse, AppError> {
    let tier = parse_tier(&payload.tier)?;
    validate_bucket_index(payload.bucket_index)?;

    if payload.proof.len() > MAX_PROOF_ENCODED_BYTES {
        return Err(AppError::ValidationError(format!(
            "Proof exceeds the {MAX_PROOF_ENCODED_BYTES}-byte encoded limit"
        )));
    }
    let proof_bytes = hex::decode(&payload.proof)
        .map_err(|_| AppError::ValidationError("proof must be hex-encoded".to_string()))?;

    let epoch = payload.epoch.as_deref().unwrap_or(DEFAULT_EPOCH);

    // Assemble the ring the proof must be against. The root is the published
    // one, never a caller-supplied value — that is what stops a scanner from
    // narrowing the anonymity set to learn who is at the gate.
    let (root_bytes, _) =
        load_sealed_bucket(&state.pool, &payload.event_id, tier, payload.bucket_index).await?;
    let mut merkle_root = [0u8; 32];
    merkle_root.copy_from_slice(&root_bytes);

    let rows =
        load_bucket_commitments(&state.pool, &payload.event_id, tier, payload.bucket_index).await?;
    let mut ring = Vec::with_capacity(rows.len());
    for row in &rows {
        ring.push(RingMember::from_bytes(&row.commitment).map_err(map_zkp_error)?);
    }

    let ctx = VerificationContext {
        event_id: &payload.event_id,
        epoch,
        tier,
        merkle_root,
        ring: &ring,
    };

    let started = Instant::now();
    let attestation = PROOF_SYSTEM
        .verify_encoded(&proof_bytes, &ctx)
        .map_err(map_zkp_error)?;
    let verification_us = started.elapsed().as_micros() as u64;

    let nullifier_hex = attestation.nullifier_hex();

    // Burn the nullifier. The primary key does the double-spend enforcement,
    // so a racing duplicate loses here rather than in an earlier SELECT.
    let insert = sqlx::query(
        r#"
        INSERT INTO zk_spent_nullifiers (
            nullifier, event_id, epoch, tier, scheme, anonymity_set_size, scanner_id
        )
        VALUES ($1, $2, $3, $4, $5, $6, $7)
        ON CONFLICT (event_id, epoch, nullifier) DO NOTHING
        "#,
    )
    .bind(attestation.nullifier.as_bytes().as_slice())
    .bind(&payload.event_id)
    .bind(epoch)
    .bind(tier.as_str())
    .bind(PROOF_SYSTEM.scheme_id())
    .bind(attestation.anonymity_set_size as i32)
    .bind(payload.scanner_id.as_deref())
    .execute(&state.pool)
    .await?;

    if insert.rows_affected() == 0 {
        // Cryptographically valid, already used. This is the replay case, and
        // it is the only situation where a *correct* proof is refused.
        warn!(
            event_id = %payload.event_id,
            tier = tier.as_str(),
            nullifier = %nullifier_hex,
            "rejected replayed ZK ticket proof"
        );
        return Err(AppError::Conflict(
            "This ticket has already been used to check in".to_string(),
        ));
    }

    info!(
        event_id = %payload.event_id,
        tier = tier.as_str(),
        anonymity_set_size = attestation.anonymity_set_size,
        verification_us,
        "admitted ZK ticket proof"
    );

    Ok(ZkCheckinResponse {
        admitted: true,
        tier: tier.as_str().to_string(),
        anonymity_set_size: attestation.anonymity_set_size as i32,
        nullifier: nullifier_hex,
        verification_us,
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn tier_parsing_accepts_the_schema_labels() {
        assert_eq!(parse_tier("general").unwrap(), AttestationTier::General);
        assert_eq!(parse_tier("age21plus").unwrap(), AttestationTier::Age21Plus);
        assert_eq!(parse_tier("vip").unwrap(), AttestationTier::Vip);
    }

    #[test]
    fn tier_parsing_rejects_anything_else() {
        // Notably including a tier that would pass a naive prefix check but is
        // not in the CHECK constraint, which would fail at INSERT instead.
        for label in ["", "age21", "GENERAL", "admin", "vip "] {
            assert!(parse_tier(label).is_err(), "accepted {label:?}");
        }
    }

    #[test]
    fn hex_parsing_enforces_the_exact_length() {
        let ok = "ab".repeat(32);
        assert_eq!(parse_hex_bytes("commitment", &ok, 32).unwrap().len(), 32);

        assert!(parse_hex_bytes("commitment", &"ab".repeat(31), 32).is_err());
        assert!(parse_hex_bytes("commitment", &"ab".repeat(33), 32).is_err());
        assert!(parse_hex_bytes("commitment", "nothex!!", 32).is_err());
    }

    #[test]
    fn negative_bucket_index_is_rejected() {
        assert!(validate_bucket_index(-1).is_err());
        assert!(validate_bucket_index(0).is_ok());
        assert!(validate_bucket_index(7).is_ok());
    }

    #[test]
    fn cryptographic_failures_all_map_to_one_opaque_status() {
        // A verifier that distinguishes these is an oracle. They must be
        // indistinguishable from outside.
        let opaque = [
            ZkpError::VerificationFailed,
            ZkpError::DegenerateNullifier,
            ZkpError::WitnessMismatch,
            ZkpError::WitnessOutOfRange(3),
        ];
        for err in opaque {
            let mapped = map_zkp_error(err);
            assert_eq!(
                mapped.status_code(),
                axum::http::StatusCode::UNAUTHORIZED,
                "{mapped:?} should be an opaque 401"
            );
            assert!(
                matches!(mapped, AppError::AuthError(ref m) if m == "Zero-knowledge proof verification failed")
            );
        }
    }

    #[test]
    fn structural_failures_are_described_to_the_caller() {
        assert_eq!(
            map_zkp_error(ZkpError::MalformedProof).status_code(),
            axum::http::StatusCode::BAD_REQUEST
        );
        assert_eq!(
            map_zkp_error(ZkpError::UnsupportedVersion(9)).status_code(),
            axum::http::StatusCode::BAD_REQUEST
        );
        // A stale ring is the client's cue to refetch, so it gets its own status.
        assert_eq!(
            map_zkp_error(ZkpError::RootMismatch).status_code(),
            axum::http::StatusCode::CONFLICT
        );
        assert_eq!(
            map_zkp_error(ZkpError::RingMismatch).status_code(),
            axum::http::StatusCode::CONFLICT
        );
    }
}
