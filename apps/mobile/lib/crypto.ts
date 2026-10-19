/**
 * crypto.ts — Issue #1179: Offline Ticket Vault
 *
 * ## Crypto design decision: Ed25519 (asymmetric), not HMAC
 *
 * The issue spec mentions "HMAC-SHA256" but then says scanners verify "against
 * pre-synced event public keys." These are incompatible:
 *
 *   - HMAC is symmetric: the same secret generates *and* verifies the tag.
 *     Distributing that secret to every gate-scanner device means a single
 *     stolen or compromised scanner leaks the ability to forge valid tickets
 *     for the whole event.
 *
 *   - The "public key" language implies asymmetric signing, where scanners
 *     only hold a public key that can *verify* but never *forge* signatures.
 *
 * We use Ed25519 (tweetnacl `nacl.sign`, already a dependency via
 * `resaleCrypto.ts`) because:
 *
 *   1. tweetnacl is already in package.json — no new dependency.
 *   2. The existing `ticket/[id].tsx` already uses `Keypair.sign()` (Ed25519,
 *      from @stellar/stellar-sdk) for rotating QR payloads. This module
 *      replaces that ad-hoc approach with a well-structured, testable API.
 *   3. `generatePurchaseSecret()` in `ticketPaymentContract.ts` already
 *      produces 32 random bytes (`Keypair.random().rawSecretKey()`), which is
 *      exactly the seed tweetnacl needs for `nacl.sign.keyPair.fromSeed()`.
 *   4. Only the 32-byte public key is synced to scanner devices. A lost
 *      scanner cannot forge tickets.
 *
 * ## Payload wire format (binary, base64url-encoded for QR)
 *
 * All multi-byte integers are big-endian.
 *
 *   Offset  Size  Field
 *   ------  ----  -----
 *        0     1  version (0x01)
 *        1     8  timestamp_s (uint64): Unix seconds, truncated to 15s window
 *        9    16  ticket_id   (UTF-8, zero-padded or trimmed to 16 bytes)
 *       25    16  nonce       (random, 16 bytes)
 *       41    64  signature   (Ed25519 over bytes[0..41])
 *   Total: 105 bytes → ~140 base64url chars → comfortably fits a QR code
 *
 * The scanner reconstructs bytes[0..41], re-verifies the signature, checks
 * the timestamp against its own clock, and checks the ticket ID against its
 * local scan log.
 */

import nacl from 'tweetnacl';

// ── Constants ─────────────────────────────────────────────────────────────────

export const PAYLOAD_VERSION = 0x01;

/** Rotation window: payload is valid for two windows either side of generation. */
export const PAYLOAD_WINDOW_S = 15;

/** Scanner clock-drift tolerance (±60s). */
export const SCANNER_CLOCK_DRIFT_S = 60;

/** Byte lengths of each payload field. */
export const PAYLOAD_OFFSETS = {
  VERSION: 0,
  TIMESTAMP: 1,
  TICKET_ID: 9,
  NONCE: 25,
  SIGNATURE: 41,
  TOTAL: 105,
} as const;

export const TICKET_ID_FIELD_BYTES = 16;
export const NONCE_BYTES = 16;
export const SIGNATURE_BYTES = 64; // Ed25519

// ── Key derivation ─────────────────────────────────────────────────────────────

export interface TicketKeyPair {
  /** 32-byte Ed25519 public key — safe to distribute to scanners. */
  publicKey: Uint8Array;
  /**
   * 64-byte nacl.sign secret key (seed ‖ public key).
   * NEVER distribute this; it stays on the attendee's device in the vault.
   */
  secretKey: Uint8Array;
}

/**
 * Derives the per-ticket Ed25519 signing keypair from the raw purchase secret
 * (the `secretBytes` returned by `generatePurchaseSecret()` in
 * `ticketPaymentContract.ts`).
 *
 * The secret is the seed for `nacl.sign.keyPair.fromSeed()`. The 32-byte
 * public key is what gets pre-synced to scanner devices.
 *
 * This derivation is deterministic: the same secretBytes always produces the
 * same keypair, so the vault can re-derive the keypair from the stored secret
 * rather than storing two separate keys.
 */
export function deriveTicketKeyPair(secretBytes: Uint8Array): TicketKeyPair {
  if (secretBytes.length !== 32) {
    throw new Error(
      `deriveTicketKeyPair: seed must be 32 bytes, got ${secretBytes.length}.`
    );
  }
  const kp = nacl.sign.keyPair.fromSeed(secretBytes);
  return { publicKey: kp.publicKey, secretKey: kp.secretKey };
}

// ── Helpers ──────────────────────────────────────────────────────────────────

/** Encodes a string to exactly `length` bytes (UTF-8 truncated or zero-padded). */
function encodeFixedString(s: string, length: number): Uint8Array {
  const te = new TextEncoder();
  const encoded = te.encode(s);
  const out = new Uint8Array(length);
  out.set(encoded.subarray(0, length));
  return out;
}

/** Decodes a fixed-length zero-padded UTF-8 field back to a string. */
function decodeFixedString(bytes: Uint8Array): string {
  const td = new TextDecoder();
  // Strip trailing zero bytes before decoding
  let end = bytes.length;
  while (end > 0 && bytes[end - 1] === 0) end--;
  return td.decode(bytes.subarray(0, end));
}

/**
 * Writes a 64-bit unsigned integer into a DataView at the given byte offset,
 * as two 32-bit big-endian words (JS has no native 64-bit integer support).
 */
function writeUint64BE(view: DataView, offset: number, value: number): void {
  const hi = Math.floor(value / 0x100000000);
  const lo = value >>> 0;
  view.setUint32(offset, hi);
  view.setUint32(offset + 4, lo);
}

/**
 * Reads a 64-bit unsigned integer from a DataView at the given byte offset.
 */
function readUint64BE(view: DataView, offset: number): number {
  const hi = view.getUint32(offset);
  const lo = view.getUint32(offset + 4);
  return hi * 0x100000000 + lo;
}

// ── Base64url encoding ────────────────────────────────────────────────────────

const BASE64URL_CHARS =
  'ABCDEFGHIJKLMNOPQRSTUVWXYZabcdefghijklmnopqrstuvwxyz0123456789-_';

export function toBase64Url(bytes: Uint8Array): string {
  let result = '';
  for (let i = 0; i < bytes.length; i += 3) {
    const b0 = bytes[i];
    const b1 = i + 1 < bytes.length ? bytes[i + 1] : 0;
    const b2 = i + 2 < bytes.length ? bytes[i + 2] : 0;
    result += BASE64URL_CHARS[b0 >> 2];
    result += BASE64URL_CHARS[((b0 & 3) << 4) | (b1 >> 4)];
    result += i + 1 < bytes.length ? BASE64URL_CHARS[((b1 & 15) << 2) | (b2 >> 6)] : '=';
    result += i + 2 < bytes.length ? BASE64URL_CHARS[b2 & 63] : '=';
  }
  return result.replace(/=+$/, '');
}

const BASE64URL_LOOKUP: Record<string, number> = {};
for (let i = 0; i < BASE64URL_CHARS.length; i++) {
  BASE64URL_LOOKUP[BASE64URL_CHARS[i]] = i;
}

export function fromBase64Url(s: string): Uint8Array {
  // Restore padding
  const padded = s + '=='.slice(0, (4 - (s.length % 4)) % 4);
  const out: number[] = [];
  for (let i = 0; i < padded.length; i += 4) {
    const c0 = BASE64URL_LOOKUP[padded[i]] ?? 0;
    const c1 = BASE64URL_LOOKUP[padded[i + 1]] ?? 0;
    const c2 = BASE64URL_LOOKUP[padded[i + 2]] ?? 0;
    const c3 = BASE64URL_LOOKUP[padded[i + 3]] ?? 0;
    out.push((c0 << 2) | (c1 >> 4));
    if (padded[i + 2] !== '=') out.push(((c1 & 15) << 4) | (c2 >> 2));
    if (padded[i + 3] !== '=') out.push(((c2 & 3) << 6) | c3);
  }
  return new Uint8Array(out);
}

// ── Payload generation ────────────────────────────────────────────────────────

/**
 * Generates a rotating QR payload for offline ticket verification.
 *
 * The payload is a 105-byte binary structure (see header for layout) encoded
 * as base64url so it can be rendered into a QR code. It is valid for at most
 * PAYLOAD_WINDOW_S × 2 seconds (current window ± scanner clock drift).
 *
 * @param ticketId   Human-readable ticket identifier (max 16 UTF-8 bytes).
 * @param secretKey  64-byte Ed25519 secret key from `deriveTicketKeyPair()`.
 * @param nowMs      Current time in milliseconds (defaults to `Date.now()`).
 */
export function generateRotatingPayload(
  ticketId: string,
  secretKey: Uint8Array,
  nowMs: number = Date.now()
): string {
  if (secretKey.length !== 64) {
    throw new Error(
      `generateRotatingPayload: secretKey must be 64 bytes, got ${secretKey.length}.`
    );
  }

  // Truncate to the current 15-second window boundary so payloads are stable
  // within the window and predictably advance every PAYLOAD_WINDOW_S seconds.
  const timestampS = Math.floor(nowMs / 1000 / PAYLOAD_WINDOW_S) * PAYLOAD_WINDOW_S;

  const buf = new Uint8Array(PAYLOAD_OFFSETS.TOTAL);
  const view = new DataView(buf.buffer);

  buf[PAYLOAD_OFFSETS.VERSION] = PAYLOAD_VERSION;
  writeUint64BE(view, PAYLOAD_OFFSETS.TIMESTAMP, timestampS);
  buf.set(encodeFixedString(ticketId, TICKET_ID_FIELD_BYTES), PAYLOAD_OFFSETS.TICKET_ID);
  buf.set(nacl.randomBytes(NONCE_BYTES), PAYLOAD_OFFSETS.NONCE);

  // Sign the header (everything before the signature field).
  const message = buf.subarray(0, PAYLOAD_OFFSETS.SIGNATURE);
  const signature = nacl.sign.detached(message, secretKey);
  buf.set(signature, PAYLOAD_OFFSETS.SIGNATURE);

  return toBase64Url(buf);
}

// ── Payload parsing ───────────────────────────────────────────────────────────

export interface ParsedPayload {
  version: number;
  timestampS: number;
  ticketId: string;
  nonce: Uint8Array;
  signature: Uint8Array;
  /** The 41 bytes over which the signature was computed. */
  signedMessage: Uint8Array;
}

/**
 * Parses a base64url QR payload into its constituent fields.
 * Throws if the binary length or version byte is wrong.
 */
export function parsePayload(encoded: string): ParsedPayload {
  const buf = fromBase64Url(encoded);
  if (buf.length !== PAYLOAD_OFFSETS.TOTAL) {
    throw new Error(
      `parsePayload: expected ${PAYLOAD_OFFSETS.TOTAL} bytes, got ${buf.length}.`
    );
  }
  const view = new DataView(buf.buffer);

  const version = buf[PAYLOAD_OFFSETS.VERSION];
  if (version !== PAYLOAD_VERSION) {
    throw new Error(`parsePayload: unsupported version ${version}.`);
  }

  return {
    version,
    timestampS: readUint64BE(view, PAYLOAD_OFFSETS.TIMESTAMP),
    ticketId: decodeFixedString(
      buf.subarray(PAYLOAD_OFFSETS.TICKET_ID, PAYLOAD_OFFSETS.NONCE)
    ),
    nonce: buf.slice(PAYLOAD_OFFSETS.NONCE, PAYLOAD_OFFSETS.SIGNATURE),
    signature: buf.slice(PAYLOAD_OFFSETS.SIGNATURE),
    signedMessage: buf.slice(0, PAYLOAD_OFFSETS.SIGNATURE),
  };
}

// ── Verification ──────────────────────────────────────────────────────────────

export type VerifyResult =
  | { ok: true; ticketId: string; timestampS: number }
  | { ok: false; reason: 'invalid_signature' | 'expired_timestamp' | 'bad_payload' };

/**
 * Verifies a rotating QR payload against a pre-synced Ed25519 public key.
 *
 * This runs entirely offline on the scanner device. No network call is made.
 *
 * @param encoded    Base64url payload string from the QR code.
 * @param publicKey  32-byte Ed25519 public key synced to the scanner.
 * @param nowMs      Current scanner time in milliseconds (defaults to Date.now()).
 */
export function verifyPayload(
  encoded: string,
  publicKey: Uint8Array,
  nowMs: number = Date.now()
): VerifyResult {
  let parsed: ParsedPayload;
  try {
    parsed = parsePayload(encoded);
  } catch {
    return { ok: false, reason: 'bad_payload' };
  }

  // Verify Ed25519 signature first (fast rejection of tampered payloads).
  const sigValid = nacl.sign.detached.verify(
    parsed.signedMessage,
    parsed.signature,
    publicKey
  );
  if (!sigValid) {
    return { ok: false, reason: 'invalid_signature' };
  }

  // Enforce ±SCANNER_CLOCK_DRIFT_S timestamp window.
  const nowS = Math.floor(nowMs / 1000);
  const delta = Math.abs(nowS - parsed.timestampS);
  if (delta > PAYLOAD_WINDOW_S + SCANNER_CLOCK_DRIFT_S) {
    return { ok: false, reason: 'expired_timestamp' };
  }

  return { ok: true, ticketId: parsed.ticketId, timestampS: parsed.timestampS };
}
