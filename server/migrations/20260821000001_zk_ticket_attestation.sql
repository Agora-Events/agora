-- Zero-knowledge ticket attestation (Issue #1186)
--
-- Lets an attendee prove at the gate that they hold a valid ticket — and
-- optionally that they were age-verified as 21+ — without revealing which
-- ticket, their name, their email, or their Stellar address.
--
-- Three tables, one per stage of the lifecycle:
--
--   1. `zk_ticket_commitments` — the Merkle leaves. One row per ticket, holding
--      a Pedersen commitment C = t·G + s·H computed on the attendee's device.
--      The server never sees `s`, so these rows are not a re-identification
--      database: a commitment is unlinkable to the ticket it came from without
--      the secret.
--
--   2. `zk_anonymity_buckets` — frozen anonymity sets. A proof is made against
--      a fixed Merkle root, so the commitment set behind that root must stop
--      changing before anyone can prove against it. Sealing a bucket publishes
--      its root and closes it to new commitments.
--
--   3. `zk_spent_nullifiers` — double-spend guard. Each valid proof discloses a
--      nullifier that is deterministic per (ticket secret, event, epoch) but
--      unlinkable to the commitment. The primary key is the enforcement: a
--      second check-in of the same ticket is a unique violation, not a race.
--
-- See `server/src/utils/zkp_verifier.rs` for the protocol these tables serve.

-- ── 1. Commitment registry (Merkle leaves) ───────────────────────────────────

CREATE TABLE IF NOT EXISTS zk_ticket_commitments (
    id              BIGSERIAL PRIMARY KEY,
    -- Event ids are on-chain / platform strings, matching `resale_listings`.
    event_id        TEXT NOT NULL,
    -- The attribute set this commitment attests to. Membership in the
    -- `age21plus` tree *is* the age proof — the issuer only writes a row here
    -- after verifying age out of band, once, at mint time. No DOB is stored.
    tier            TEXT NOT NULL
                        CHECK (tier IN ('general', 'age21plus', 'vip')),
    -- Anonymity bucket. Verification is O(ring size), so events larger than
    -- MAX_RING_SIZE are split; an attendee proves against their own bucket.
    bucket_index    INTEGER NOT NULL DEFAULT 0 CHECK (bucket_index >= 0),
    -- Compressed ristretto255 point, exactly 32 bytes.
    commitment      BYTEA NOT NULL CHECK (octet_length(commitment) = 32),
    -- Domain-separated hash of `commitment`; the Merkle leaf. Stored rather
    -- than recomputed so root assembly is a single ordered scan.
    leaf_hash       BYTEA NOT NULL CHECK (octet_length(leaf_hash) = 32),
    created_at      TIMESTAMPTZ NOT NULL DEFAULT NOW(),

    -- A commitment may appear once per (event, tier). Re-registering is an
    -- idempotent no-op rather than a way to inflate a ring with duplicates.
    CONSTRAINT zk_ticket_commitments_unique UNIQUE (event_id, tier, commitment)
);

-- Ring assembly reads every commitment in one bucket, in insertion order.
-- Order matters: the Merkle root is order-sensitive, and `id` is what fixes it.
CREATE INDEX IF NOT EXISTS idx_zk_commitments_bucket
    ON zk_ticket_commitments (event_id, tier, bucket_index, id);

-- ── 2. Frozen anonymity sets ─────────────────────────────────────────────────

CREATE TABLE IF NOT EXISTS zk_anonymity_buckets (
    event_id            TEXT NOT NULL,
    tier                TEXT NOT NULL
                            CHECK (tier IN ('general', 'age21plus', 'vip')),
    bucket_index        INTEGER NOT NULL CHECK (bucket_index >= 0),
    -- Number of commitments at seal time. Also the anonymity-set size: the
    -- honest denominator of the privacy guarantee, kept for auditing.
    commitment_count    INTEGER NOT NULL DEFAULT 0 CHECK (commitment_count >= 0),
    -- Published Merkle root, set at seal. NULL means the bucket is still
    -- filling and cannot be proven against.
    merkle_root         BYTEA CHECK (merkle_root IS NULL OR octet_length(merkle_root) = 32),
    sealed_at           TIMESTAMPTZ,
    created_at          TIMESTAMPTZ NOT NULL DEFAULT NOW(),

    PRIMARY KEY (event_id, tier, bucket_index),

    -- A bucket is sealed exactly when it has a root. Without this the two
    -- columns could disagree and a proof could be accepted against a root
    -- that was never published.
    CONSTRAINT zk_bucket_sealed_iff_rooted
        CHECK ((sealed_at IS NULL) = (merkle_root IS NULL))
);

-- ── 3. Nullifier store (double-spend guard) ──────────────────────────────────

CREATE TABLE IF NOT EXISTS zk_spent_nullifiers (
    -- Compressed ristretto255 point N = s·Ω, disclosed by the proof.
    nullifier           BYTEA NOT NULL CHECK (octet_length(nullifier) = 32),
    event_id            TEXT NOT NULL,
    -- Nullifier epoch. One epoch per event makes a ticket single-entry;
    -- a rotating epoch would allow controlled re-entry.
    epoch               TEXT NOT NULL,
    tier                TEXT NOT NULL
                            CHECK (tier IN ('general', 'age21plus', 'vip')),
    -- Proof system that produced this row, so a future migration to a
    -- different scheme can tell the generations apart.
    scheme              TEXT NOT NULL,
    -- Ring size the holder hid in, recorded for privacy auditing.
    anonymity_set_size  INTEGER NOT NULL CHECK (anonymity_set_size > 0),
    -- Which gate device accepted it. Operational only — it says nothing about
    -- *who* walked through, which is the entire point of this feature.
    scanner_id          TEXT,
    spent_at            TIMESTAMPTZ NOT NULL DEFAULT NOW(),

    -- THE double-spend guard. Two scanners racing the same ticket both try to
    -- insert this key; exactly one wins and the other gets a unique violation.
    PRIMARY KEY (event_id, epoch, nullifier)
);

-- Check-in throughput reporting, and the "how many are inside" query.
CREATE INDEX IF NOT EXISTS idx_zk_nullifiers_event_time
    ON zk_spent_nullifiers (event_id, spent_at DESC);
