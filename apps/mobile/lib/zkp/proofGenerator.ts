/**
 * proofGenerator.ts — Issue #1186: on-device zero-knowledge ticket prover.
 *
 * The mobile half of the protocol implemented in
 * `server/src/utils/zkp_verifier.rs`. That file is the specification; this one
 * must agree with it byte for byte or nothing verifies, so read the two
 * together. The module docs there explain the scheme and, importantly, why it
 * is a ring Chaum–Pedersen sigma protocol rather than the Groth16 pipeline the
 * issue asked for.
 *
 * ## What the attendee proves
 *
 * > "One of the commitments in this published set is mine, and here is a
 * > nullifier proving I have not already walked through the gate — but not
 * > which commitment, and not who I am."
 *
 * ## What never leaves the device
 *
 * `secretBytes`. It is the blinding factor of the Pedersen commitment and the
 * key to the nullifier. Anyone holding it can impersonate the ticket, and
 * anyone holding it *and* the commitment registry can deanonymise every
 * check-in that ticket ever made. It belongs in the same secure storage as the
 * offline ticket vault's signing seed, and it is never sent anywhere.
 *
 * ## Cost
 *
 * Proving is O(ring size): roughly `3n + 2` scalar multiplications, about
 * 1 ms each in Hermes. A ring of 8 is ~30 ms and a ring of 64 is ~250 ms —
 * fine to run when the ticket screen opens, and worth doing there rather than
 * at the turnstile. {@link generateCheckinProof} is synchronous and CPU-bound,
 * so call it off the interaction path for large rings.
 */

import type { ExtendedPoint } from './ristretto255';
import {
  BASEPOINT,
  IDENTITY,
  pointCompress,
  pointDecompress,
  pointEquals,
  pointFromUniformBytes,
  pointMul,
  pointMulAdd,
  pointSub,
  scalarAdd,
  scalarFromBytesWide,
  scalarMod,
  scalarMul,
  scalarSub,
  scalarToBytes,
} from './ristretto255';
import { randomBytes, sha512 } from './sha512';

// ── Protocol constants ───────────────────────────────────────────────────────
//
// These MUST match `server/src/utils/zkp_verifier.rs` exactly. They are
// versioned rather than edited: changing one invalidates every commitment and
// proof in circulation.

const DOMAIN_GENERATOR_H = 'agora/zkp/v1/generator-h';
const DOMAIN_TICKET_SCALAR = 'agora/zkp/v1/ticket-scalar';
const DOMAIN_SECRET_SCALAR = 'agora/zkp/v1/secret-scalar';
const DOMAIN_NULLIFIER_BASE = 'agora/zkp/v1/nullifier-base';
const DOMAIN_MERKLE_LEAF = 'agora/zkp/v1/merkle-leaf';
const DOMAIN_MERKLE_NODE = 'agora/zkp/v1/merkle-node';
const DOMAIN_TRANSCRIPT = 'agora/zkp/v1/fiat-shamir';

/** Wire-format version of the proof encoding. */
export const PROOF_VERSION = 0x01;

/** Default nullifier epoch. One epoch per event makes a ticket single-entry. */
export const DEFAULT_EPOCH = 'checkin';

/** Largest ring the server will verify. Mirrors `MAX_RING_SIZE` in Rust. */
export const MAX_RING_SIZE = 256;

const ZERO = BigInt(0);

// ── Attestation tiers ────────────────────────────────────────────────────────

/** Which attribute set a commitment tree attests to. */
export type AttestationTier = 'general' | 'age21plus' | 'vip';

/** Wire byte for a tier. Bound into the transcript, so it is not cosmetic. */
export function tierByte(tier: AttestationTier): number {
  switch (tier) {
    case 'general':
      return 0;
    case 'age21plus':
      return 1;
    case 'vip':
      return 2;
  }
}

/**
 * Smallest ring the server will accept for a tier.
 *
 * A tiny ring is still zero-knowledge about the secrets but tells the verifier
 * almost exactly which commitment was used, so the server refuses one. Checked
 * here too, to fail on the device rather than at the turnstile.
 */
export function tierMinRing(tier: AttestationTier): number {
  return tier === 'vip' ? 4 : 8;
}

// ── Byte helpers ─────────────────────────────────────────────────────────────

const textEncoder = new TextEncoder();

/** UTF-8 encodes a string. */
function utf8(value: string): Uint8Array {
  return textEncoder.encode(value);
}

/** Concatenates byte arrays. */
function concatBytes(chunks: Uint8Array[]): Uint8Array {
  let total = 0;
  for (const chunk of chunks) total += chunk.length;
  const out = new Uint8Array(total);
  let offset = 0;
  for (const chunk of chunks) {
    out.set(chunk, offset);
    offset += chunk.length;
  }
  return out;
}

/** Encodes a non-negative integer as 8 big-endian bytes. */
function uint64BE(value: number): Uint8Array {
  const out = new Uint8Array(8);
  let v = BigInt(value);
  const BYTE = BigInt(0xff);
  const EIGHT = BigInt(8);
  for (let i = 7; i >= 0; i--) {
    out[i] = Number(v & BYTE);
    v >>= EIGHT;
  }
  return out;
}

/** Lowercase hex encoding. */
export function toHex(bytes: Uint8Array): string {
  let out = '';
  for (const byte of bytes) {
    out += byte.toString(16).padStart(2, '0');
  }
  return out;
}

/** Decodes lowercase or uppercase hex. Throws on malformed input. */
export function fromHex(hex: string): Uint8Array {
  if (hex.length % 2 !== 0) {
    throw new Error(`fromHex: odd-length string (${hex.length})`);
  }
  const out = new Uint8Array(hex.length / 2);
  for (let i = 0; i < out.length; i++) {
    const byte = Number.parseInt(hex.slice(i * 2, i * 2 + 2), 16);
    if (Number.isNaN(byte)) {
      throw new Error('fromHex: string contains non-hex characters');
    }
    out[i] = byte;
  }
  return out;
}

// ── Transcript hashing ───────────────────────────────────────────────────────

/**
 * Length-prefixes each field before hashing, matching `absorb` in Rust.
 *
 * The 8-byte big-endian prefix is what stops `("ab", "c")` and `("a", "bc")`
 * hashing alike, which would let a prover shift bytes between the event id and
 * the epoch and reuse a proof across events.
 */
function absorb(parts: Uint8Array[]): Uint8Array {
  const chunks: Uint8Array[] = [];
  for (const part of parts) {
    chunks.push(uint64BE(part.length));
    chunks.push(part);
  }
  return concatBytes(chunks);
}

/** SHA-512 over a domain tag and a list of length-prefixed fields. */
function transcript(domain: string, parts: Uint8Array[]): Uint8Array {
  return sha512(absorb([utf8(domain), ...parts]));
}

/** Hash-to-group: maps arbitrary input to a group element. */
export function hashToGroup(domain: string, parts: Uint8Array[]): ExtendedPoint {
  return pointFromUniformBytes(transcript(domain, parts));
}

/** Hash-to-scalar, via wide reduction so the result is uniform mod ℓ. */
export function hashToScalar(domain: string, parts: Uint8Array[]): bigint {
  return scalarFromBytesWide(transcript(domain, parts));
}

/** 32-byte domain-separated hash. SHA-512 truncated, matching the verifier. */
export function hash32(domain: string, parts: Uint8Array[]): Uint8Array {
  return transcript(domain, parts).slice(0, 32);
}

/** Reduces a 64-byte digest to a scalar. */
function digestToScalar(digest: Uint8Array): bigint {
  return scalarFromBytesWide(digest);
}

// ── Public parameters ────────────────────────────────────────────────────────

let cachedGeneratorH: ExtendedPoint | null = null;

/**
 * The second Pedersen generator `H`, derived by hash-to-group so nobody knows
 * `log_G H` — which is precisely what makes the commitment binding.
 *
 * Cached because deriving it costs two Elligator maps, and it never changes.
 */
export function generatorH(): ExtendedPoint {
  if (cachedGeneratorH === null) {
    cachedGeneratorH = hashToGroup(DOMAIN_GENERATOR_H, []);
  }
  return cachedGeneratorH;
}

/** The ristretto255 basepoint `G`. */
export function generatorG(): ExtendedPoint {
  return BASEPOINT;
}

/** Derives the ticket scalar `t` bound into a commitment. */
export function ticketScalar(ticketId: string): bigint {
  return hashToScalar(DOMAIN_TICKET_SCALAR, [utf8(ticketId)]);
}

/** Derives the blinding scalar `s` from the attendee's 32 random secret bytes. */
export function secretScalar(secretBytes: Uint8Array): bigint {
  return hashToScalar(DOMAIN_SECRET_SCALAR, [secretBytes]);
}

/** Builds the Pedersen commitment `C = t·G + s·H`. */
export function commit(ticket: bigint, secret: bigint): ExtendedPoint {
  return pointMulAdd(generatorG(), ticket, generatorH(), secret);
}

/**
 * Builds the commitment to register with the issuer at mint time.
 *
 * Call this once per ticket, send the hex to
 * `POST /api/v1/admin/zk/commitments`, and keep `secretBytes` on the device.
 */
export function buildCommitment(
  ticketId: string,
  secretBytes: Uint8Array
): { commitmentHex: string; commitment: ExtendedPoint } {
  const commitment = commit(ticketScalar(ticketId), secretScalar(secretBytes));
  return { commitmentHex: toHex(pointCompress(commitment)), commitment };
}

/** Derives the nullifier base `Ω` for an (event, epoch) pair. */
export function nullifierBase(eventId: string, epoch: string): ExtendedPoint {
  return hashToGroup(DOMAIN_NULLIFIER_BASE, [utf8(eventId), utf8(epoch)]);
}

/** Computes the nullifier `N = s·Ω`. Deterministic per (secret, event, epoch). */
export function computeNullifier(
  secret: bigint,
  eventId: string,
  epoch: string
): ExtendedPoint {
  return pointMul(nullifierBase(eventId, epoch), secret);
}

// ── Merkle tree ──────────────────────────────────────────────────────────────

/** Hashes a compressed commitment into its Merkle leaf. */
export function leafHash(compressedCommitment: Uint8Array): Uint8Array {
  return hash32(DOMAIN_MERKLE_LEAF, [compressedCommitment]);
}

/** Hashes an internal Merkle node from its two children. */
function nodeHash(left: Uint8Array, right: Uint8Array): Uint8Array {
  return hash32(DOMAIN_MERKLE_NODE, [left, right]);
}

/**
 * Computes the Merkle root over a list of leaves.
 *
 * An odd node at any level is **promoted** unchanged rather than duplicated.
 * Duplicating the last leaf is the CVE-2012-2459 shape, where two different
 * leaf lists produce the same root; promotion does not have that property.
 * The Rust verifier does the same, and a mismatch here means every proof is
 * rejected with `RootMismatch`.
 */
export function merkleRoot(leaves: Uint8Array[]): Uint8Array {
  if (leaves.length === 0) return new Uint8Array(32);

  let level = leaves;
  while (level.length > 1) {
    const next: Uint8Array[] = [];
    for (let i = 0; i < level.length; i += 2) {
      if (i + 1 < level.length) {
        next.push(nodeHash(level[i], level[i + 1]));
      } else {
        next.push(level[i]);
      }
    }
    level = next;
  }
  return level[0];
}

/** Computes the Merkle root over a list of compressed commitments. */
export function merkleRootOfCommitments(commitments: Uint8Array[]): Uint8Array {
  return merkleRoot(commitments.map(leafHash));
}

// ── Ring members ─────────────────────────────────────────────────────────────

/** One commitment in the anonymity set, kept in both representations. */
export interface RingMember {
  /** Decompressed commitment, for arithmetic. */
  point: ExtendedPoint;
  /** Canonical 32-byte encoding, for the transcript. */
  compressed: Uint8Array;
}

/** Decodes a hex commitment from `GET /api/v1/zk/ring` into a ring member. */
export function ringMemberFromHex(hex: string): RingMember {
  const compressed = fromHex(hex);
  const point = pointDecompress(compressed);
  if (point === null) {
    throw new Error(`ringMemberFromHex: '${hex}' is not a valid ristretto255 point`);
  }
  return { point, compressed };
}

/** Decodes a whole anonymity set. */
export function ringFromHex(commitments: string[]): RingMember[] {
  return commitments.map(ringMemberFromHex);
}

/**
 * Finds the attendee's own commitment in the ring.
 *
 * Returns −1 when it is absent, which means the ticket was never registered or
 * was registered into a different bucket — a situation worth surfacing to the
 * user, because no proof can be built.
 */
export function findWitnessIndex(ring: RingMember[], commitment: ExtendedPoint): number {
  for (let i = 0; i < ring.length; i++) {
    if (pointEquals(ring[i].point, commitment)) return i;
  }
  return -1;
}

// ── Randomness ───────────────────────────────────────────────────────────────

/**
 * Source of the prover's blinding randomness.
 *
 * Injectable only so the conformance tests can drive the prover
 * deterministically and compare against Rust-generated vectors. Production
 * always uses {@link secureRng}.
 *
 * The randomness here is not decorative: reusing a blinding scalar across two
 * proofs for the same ticket leaks the witness outright.
 */
export interface ProverRng {
  nextScalar(): bigint;
}

/** The production RNG, drawing from the platform CSPRNG. */
export const secureRng: ProverRng = {
  nextScalar(): bigint {
    return scalarFromBytesWide(randomBytes(64));
  },
};

// ── Proof ────────────────────────────────────────────────────────────────────

/** A non-interactive ring Chaum–Pedersen proof. */
export interface RingProof {
  version: number;
  tier: AttestationTier;
  /** Compressed nullifier `N`. */
  nullifier: Uint8Array;
  /** Per-member challenges, summing to the Fiat–Shamir challenge. */
  challenges: bigint[];
  /** Per-member responses for the `G` component. */
  zTicket: bigint[];
  /** Per-member responses for the `H` / `Ω` component. */
  zSecret: bigint[];
}

/**
 * Serialises a proof to the wire encoding the verifier parses.
 *
 * ```text
 *   0    1   version
 *   1    1   tier
 *   2    2   ring_size n (big-endian u16)
 *   4   32   nullifier
 *  36  32n   challenges
 *  ..  32n   z_ticket
 *  ..  32n   z_secret
 * ```
 */
export function encodeProof(proof: RingProof): Uint8Array {
  const n = proof.challenges.length;
  const header = new Uint8Array(4);
  header[0] = proof.version;
  header[1] = tierByte(proof.tier);
  header[2] = (n >> 8) & 0xff;
  header[3] = n & 0xff;

  const chunks: Uint8Array[] = [header, proof.nullifier];
  for (const group of [proof.challenges, proof.zTicket, proof.zSecret]) {
    for (const scalar of group) {
      chunks.push(scalarToBytes(scalar));
    }
  }
  return concatBytes(chunks);
}

/** Everything the proof is bound to. Mirrors `VerificationContext` in Rust. */
export interface ProofContext {
  eventId: string;
  epoch: string;
  tier: AttestationTier;
  /** The published 32-byte Merkle root, as returned by `GET /zk/ring`. */
  merkleRoot: Uint8Array;
  ring: RingMember[];
}

/** The attendee's secret for one ticket. */
export interface Witness {
  index: number;
  ticket: bigint;
  secret: bigint;
}

/**
 * Builds the Fiat–Shamir transcript prefix.
 *
 * Prover and verifier must absorb byte-identical input here, which is the
 * single most common way a hand-rolled sigma protocol silently breaks. The
 * Rust counterpart is `base_transcript`; changes must land in both.
 */
function baseTranscript(
  ctx: ProofContext,
  version: number,
  omegaCompressed: Uint8Array,
  nullifierCompressed: Uint8Array
): Uint8Array[] {
  const parts: Uint8Array[] = [
    utf8(DOMAIN_TRANSCRIPT),
    new Uint8Array([version, tierByte(ctx.tier)]),
    utf8(ctx.eventId),
    utf8(ctx.epoch),
    ctx.merkleRoot,
    omegaCompressed,
    nullifierCompressed,
    uint64BE(ctx.ring.length),
  ];
  for (const member of ctx.ring) {
    parts.push(member.compressed);
  }
  return parts;
}

/**
 * Generates a ring Chaum–Pedersen proof.
 *
 * The construction: every ring member except the real one is **simulated** —
 * pick its challenge and both responses at random, then solve the verification
 * equation backwards for its commitments. A simulated transcript is
 * distributed identically to an honest one, which is exactly why the verifier
 * cannot tell them apart. That is the zero-knowledge property itself, not a
 * shortcut around it.
 *
 * The one degree of freedom left is the real member's challenge, which is
 * forced to make the per-member challenges sum to the Fiat–Shamir value. Only
 * someone who knows `(t, s)` can answer it.
 */
export function prove(
  witness: Witness,
  ctx: ProofContext,
  rng: ProverRng = secureRng
): RingProof {
  const n = ctx.ring.length;
  const minRing = tierMinRing(ctx.tier);
  if (n < minRing || n > MAX_RING_SIZE) {
    throw new Error(
      `prove: ring size ${n} outside the permitted range ${minRing}..${MAX_RING_SIZE} for tier '${ctx.tier}'`
    );
  }
  if (witness.index < 0 || witness.index >= n) {
    throw new Error(`prove: witness index ${witness.index} is outside the ring`);
  }
  if (ctx.merkleRoot.length !== 32) {
    throw new Error(`prove: merkleRoot must be 32 bytes, got ${ctx.merkleRoot.length}`);
  }
  if (!pointEquals(commit(witness.ticket, witness.secret), ctx.ring[witness.index].point)) {
    throw new Error('prove: witness does not open the commitment at its index');
  }

  const omega = nullifierBase(ctx.eventId, ctx.epoch);
  const omegaCompressed = pointCompress(omega);
  const nullifier = pointMul(omega, witness.secret);
  const nullifierCompressed = pointCompress(nullifier);

  const g = generatorG();
  const h = generatorH();
  const l = witness.index;

  const challenges: bigint[] = new Array(n).fill(ZERO);
  const zTicket: bigint[] = new Array(n).fill(ZERO);
  const zSecret: bigint[] = new Array(n).fill(ZERO);
  const aPoints: ExtendedPoint[] = new Array(n).fill(IDENTITY);
  const bPoints: ExtendedPoint[] = new Array(n).fill(IDENTITY);

  // Simulate every member but the real one.
  for (let i = 0; i < n; i++) {
    if (i === l) continue;
    const c = rng.nextScalar();
    const zt = rng.nextScalar();
    const zs = rng.nextScalar();
    challenges[i] = c;
    zTicket[i] = zt;
    zSecret[i] = zs;
    // A_i = z_t·G + z_s·H − c·C_i
    aPoints[i] = pointSub(pointMulAdd(g, zt, h, zs), pointMul(ctx.ring[i].point, c));
    // B_i = z_s·Ω − c·N
    bPoints[i] = pointSub(pointMul(omega, zs), pointMul(nullifier, c));
  }

  // The real member commits honestly.
  const rTicket = rng.nextScalar();
  const rSecret = rng.nextScalar();
  aPoints[l] = pointMulAdd(g, rTicket, h, rSecret);
  bPoints[l] = pointMul(omega, rSecret);

  // Fiat–Shamir over the whole ring.
  const parts = baseTranscript(ctx, PROOF_VERSION, omegaCompressed, nullifierCompressed);
  for (let i = 0; i < n; i++) {
    parts.push(pointCompress(aPoints[i]));
    parts.push(pointCompress(bPoints[i]));
  }
  const challenge = digestToScalar(sha512(absorb(parts)));

  let simulatedSum = ZERO;
  for (let i = 0; i < n; i++) {
    if (i !== l) simulatedSum = scalarAdd(simulatedSum, challenges[i]);
  }
  const cl = scalarSub(challenge, simulatedSum);
  challenges[l] = cl;
  zTicket[l] = scalarAdd(rTicket, scalarMul(cl, witness.ticket));
  zSecret[l] = scalarAdd(rSecret, scalarMul(cl, witness.secret));

  return {
    version: PROOF_VERSION,
    tier: ctx.tier,
    nullifier: nullifierCompressed,
    challenges,
    zTicket,
    zSecret,
  };
}

// ── High-level API ───────────────────────────────────────────────────────────

/** Inputs for {@link generateCheckinProof}. */
export interface CheckinProofRequest {
  /** Event being entered. */
  eventId: string;
  /** Tier being claimed. Must match the ring that was fetched. */
  tier: AttestationTier;
  /** Nullifier epoch. Defaults to {@link DEFAULT_EPOCH}. */
  epoch?: string;
  /** Hex commitments from `GET /api/v1/zk/ring`, in the order returned. */
  ringHex: string[];
  /** Hex Merkle root from the same response. */
  merkleRootHex: string;
  /** The attendee's ticket identifier. */
  ticketId: string;
  /** The attendee's 32 secret bytes. Never transmitted. */
  secretBytes: Uint8Array;
  /** Overridable only for tests. */
  rng?: ProverRng;
}

/** The proof, ready to POST to `/api/v1/zk/checkin`. */
export interface CheckinProofResult {
  /** Hex proof for the request body. */
  proofHex: string;
  /** Hex nullifier this proof burns. Useful for local "already used" state. */
  nullifierHex: string;
  /** Where the attendee sat in the ring. Never sent — for diagnostics only. */
  witnessIndex: number;
  /** Size of the anonymity set the attendee is hiding in. */
  anonymitySetSize: number;
}

/**
 * End-to-end: turn a fetched anonymity set plus the attendee's ticket secret
 * into a proof the gate will accept.
 *
 * Verifies the fetched ring against the fetched root before proving. That
 * check is not ceremony: if the server (or anything between it and the phone)
 * hands over a narrowed ring, proving against it would still succeed and the
 * attendee's anonymity set would be whatever the attacker chose. Catching it
 * here means a tampered ring fails on the device instead of quietly costing
 * the user their privacy.
 */
export function generateCheckinProof(request: CheckinProofRequest): CheckinProofResult {
  const epoch = request.epoch ?? DEFAULT_EPOCH;
  const merkleRoot = fromHex(request.merkleRootHex);
  if (merkleRoot.length !== 32) {
    throw new Error(`generateCheckinProof: merkleRootHex must be 32 bytes, got ${merkleRoot.length}`);
  }

  const ring = ringFromHex(request.ringHex);

  const recomputed = merkleRootOfCommitments(ring.map((m) => m.compressed));
  if (toHex(recomputed) !== toHex(merkleRoot)) {
    throw new Error(
      'generateCheckinProof: anonymity set does not reproduce the published Merkle root; ' +
        'the ring may have been tampered with or is stale — refetch it'
    );
  }

  const ticket = ticketScalar(request.ticketId);
  const secret = secretScalar(request.secretBytes);
  const own = commit(ticket, secret);

  const index = findWitnessIndex(ring, own);
  if (index < 0) {
    throw new Error(
      'generateCheckinProof: this ticket is not in the anonymity set; it may not be ' +
        'registered yet, or it may belong to a different tier or bucket'
    );
  }

  const proof = prove(
    { index, ticket, secret },
    { eventId: request.eventId, epoch, tier: request.tier, merkleRoot, ring },
    request.rng ?? secureRng
  );

  return {
    proofHex: toHex(encodeProof(proof)),
    nullifierHex: toHex(proof.nullifier),
    witnessIndex: index,
    anonymitySetSize: ring.length,
  };
}

/**
 * Local verification of a proof, mirroring the server's `verify`.
 *
 * The gate does the authoritative check; this exists so the device can confirm
 * it built something valid before the attendee is standing at a turnstile, and
 * so the test suite can assert soundness without a running server.
 */
export function verifyLocally(proof: RingProof, ctx: ProofContext): boolean {
  const n = ctx.ring.length;
  if (proof.version !== PROOF_VERSION) return false;
  if (proof.tier !== ctx.tier) return false;
  if (proof.challenges.length !== n || proof.zTicket.length !== n || proof.zSecret.length !== n) {
    return false;
  }

  const recomputedRoot = merkleRootOfCommitments(ctx.ring.map((m) => m.compressed));
  if (toHex(recomputedRoot) !== toHex(ctx.merkleRoot)) return false;

  const nullifier = pointDecompress(proof.nullifier);
  if (nullifier === null) return false;
  if (pointEquals(nullifier, IDENTITY)) return false;

  const omega = nullifierBase(ctx.eventId, ctx.epoch);
  const g = generatorG();
  const h = generatorH();

  const parts = baseTranscript(ctx, proof.version, pointCompress(omega), proof.nullifier);
  let challengeSum = ZERO;

  for (let i = 0; i < n; i++) {
    const c = proof.challenges[i];
    const a = pointSub(
      pointMulAdd(g, proof.zTicket[i], h, proof.zSecret[i]),
      pointMul(ctx.ring[i].point, c)
    );
    const b = pointSub(pointMul(omega, proof.zSecret[i]), pointMul(nullifier, c));
    parts.push(pointCompress(a));
    parts.push(pointCompress(b));
    challengeSum = scalarAdd(challengeSum, c);
  }

  return digestToScalar(sha512(absorb(parts))) === scalarMod(challengeSum);
}

// Re-exported so callers can work with commitments without reaching into the
// group-arithmetic module directly.
export type { ExtendedPoint };
export { pointCompress, pointDecompress };
