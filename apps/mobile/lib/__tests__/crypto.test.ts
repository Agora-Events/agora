/**
 * crypto.test.ts — Issue #1179: Offline Ticket Vault
 *
 * Tests for:
 *   - Key derivation (deriveTicketKeyPair)
 *   - Payload generation (generateRotatingPayload)
 *   - Payload parsing (parsePayload)
 *   - Payload verification (verifyPayload) — timestamp window, signature validation
 *   - Base64url round-trip
 */

import nacl from 'tweetnacl';
import {
  deriveTicketKeyPair,
  generateRotatingPayload,
  parsePayload,
  verifyPayload,
  toBase64Url,
  fromBase64Url,
  PAYLOAD_OFFSETS,
  PAYLOAD_WINDOW_S,
  SCANNER_CLOCK_DRIFT_S,
  PAYLOAD_VERSION,
} from '../../lib/crypto';

// ── Fixtures ──────────────────────────────────────────────────────────────────

function makeSecret(fillByte = 0x42): Uint8Array {
  return new Uint8Array(32).fill(fillByte);
}

const FIXED_SECRET = makeSecret(0x42);
const FIXED_NOW_MS = 1_700_000_000_000; // arbitrary fixed timestamp

// ── Base64url helpers ─────────────────────────────────────────────────────────

describe('base64url encoding', () => {
  it('round-trips arbitrary bytes', () => {
    const original = new Uint8Array([0, 1, 127, 128, 255, 66, 99]);
    expect(fromBase64Url(toBase64Url(original))).toEqual(original);
  });

  it('produces url-safe characters only', () => {
    const bytes = nacl.randomBytes(100);
    const encoded = toBase64Url(bytes);
    expect(encoded).toMatch(/^[A-Za-z0-9\-_]+$/);
  });

  it('omits padding', () => {
    // Base64url should not contain '='
    const bytes = nacl.randomBytes(10);
    expect(toBase64Url(bytes)).not.toContain('=');
  });

  it('round-trips all 256 byte values in a 256-byte array', () => {
    const all = new Uint8Array(256);
    for (let i = 0; i < 256; i++) all[i] = i;
    expect(fromBase64Url(toBase64Url(all))).toEqual(all);
  });
});

// ── Key derivation ────────────────────────────────────────────────────────────

describe('deriveTicketKeyPair', () => {
  it('returns a 32-byte public key and 64-byte secret key', () => {
    const { publicKey, secretKey } = deriveTicketKeyPair(FIXED_SECRET);
    expect(publicKey.length).toBe(32);
    expect(secretKey.length).toBe(64);
  });

  it('is deterministic: same seed → same keypair', () => {
    const kp1 = deriveTicketKeyPair(FIXED_SECRET);
    const kp2 = deriveTicketKeyPair(FIXED_SECRET);
    expect(kp1.publicKey).toEqual(kp2.publicKey);
    expect(kp1.secretKey).toEqual(kp2.secretKey);
  });

  it('different seeds produce different keypairs', () => {
    const kp1 = deriveTicketKeyPair(makeSecret(0x01));
    const kp2 = deriveTicketKeyPair(makeSecret(0x02));
    expect(kp1.publicKey).not.toEqual(kp2.publicKey);
  });

  it('throws for a seed shorter than 32 bytes', () => {
    expect(() => deriveTicketKeyPair(new Uint8Array(16))).toThrow();
  });

  it('throws for a seed longer than 32 bytes', () => {
    expect(() => deriveTicketKeyPair(new Uint8Array(33))).toThrow();
  });

  it('nacl.sign.detached.verify accepts signatures from the derived keypair', () => {
    const { publicKey, secretKey } = deriveTicketKeyPair(FIXED_SECRET);
    const msg = new Uint8Array([1, 2, 3]);
    const sig = nacl.sign.detached(msg, secretKey);
    expect(nacl.sign.detached.verify(msg, sig, publicKey)).toBe(true);
  });
});

// ── Payload generation ────────────────────────────────────────────────────────

describe('generateRotatingPayload', () => {
  const { secretKey } = deriveTicketKeyPair(FIXED_SECRET);

  it('returns a non-empty string', () => {
    const payload = generateRotatingPayload('ticket-1', secretKey, FIXED_NOW_MS);
    expect(typeof payload).toBe('string');
    expect(payload.length).toBeGreaterThan(0);
  });

  it('decodes to exactly PAYLOAD_OFFSETS.TOTAL bytes', () => {
    const payload = generateRotatingPayload('ticket-1', secretKey, FIXED_NOW_MS);
    const bytes = fromBase64Url(payload);
    expect(bytes.length).toBe(PAYLOAD_OFFSETS.TOTAL);
  });

  it('sets the version byte correctly', () => {
    const payload = generateRotatingPayload('ticket-1', secretKey, FIXED_NOW_MS);
    const bytes = fromBase64Url(payload);
    expect(bytes[PAYLOAD_OFFSETS.VERSION]).toBe(PAYLOAD_VERSION);
  });

  it('embeds the window-aligned timestamp', () => {
    const payload = generateRotatingPayload('ticket-1', secretKey, FIXED_NOW_MS);
    const parsed = parsePayload(payload);
    const expectedWindow =
      Math.floor(FIXED_NOW_MS / 1000 / PAYLOAD_WINDOW_S) * PAYLOAD_WINDOW_S;
    expect(parsed.timestampS).toBe(expectedWindow);
  });

  it('two calls within the same 15s window have the same timestamp', () => {
    // Both calls use timestamps that are in the same window boundary.
    const t1 = 1_700_000_000_000;
    const t2 = t1 + 7_000; // +7s, same window
    const p1 = parsePayload(generateRotatingPayload('ticket-1', secretKey, t1));
    const p2 = parsePayload(generateRotatingPayload('ticket-1', secretKey, t2));
    expect(p1.timestampS).toBe(p2.timestampS);
  });

  it('payloads in adjacent windows have different timestamps', () => {
    const t1 = 1_700_000_000_000;
    const windowMs = PAYLOAD_WINDOW_S * 1000;
    const t2 = t1 + windowMs + 1; // next window
    const p1 = parsePayload(generateRotatingPayload('ticket-1', secretKey, t1));
    const p2 = parsePayload(generateRotatingPayload('ticket-1', secretKey, t2));
    expect(p1.timestampS).not.toBe(p2.timestampS);
  });

  it('embeds the ticketId', () => {
    const ticketId = 'test-ticket-99';
    const payload = generateRotatingPayload(ticketId, secretKey, FIXED_NOW_MS);
    const parsed = parsePayload(payload);
    expect(parsed.ticketId).toBe(ticketId);
  });

  it('truncates ticketId longer than 16 bytes', () => {
    // 'abcdefghijklmnop' is exactly 16 bytes; 'abcdefghijklmnopXYZ' is 19.
    const long = 'abcdefghijklmnopXYZ';
    const payload = generateRotatingPayload(long, secretKey, FIXED_NOW_MS);
    const parsed = parsePayload(payload);
    // The stored ticket ID should be the first 16 bytes of the UTF-8 encoding.
    expect(parsed.ticketId).toBe('abcdefghijklmnop');
  });

  it('two payloads for the same ticket at the same time have different nonces', () => {
    const p1 = fromBase64Url(generateRotatingPayload('t1', secretKey, FIXED_NOW_MS));
    const p2 = fromBase64Url(generateRotatingPayload('t1', secretKey, FIXED_NOW_MS));
    const nonce1 = p1.slice(PAYLOAD_OFFSETS.NONCE, PAYLOAD_OFFSETS.SIGNATURE);
    const nonce2 = p2.slice(PAYLOAD_OFFSETS.NONCE, PAYLOAD_OFFSETS.SIGNATURE);
    // With overwhelming probability the nonces differ (16 random bytes).
    expect(nonce1).not.toEqual(nonce2);
  });

  it('throws if secretKey is not 64 bytes', () => {
    expect(() =>
      generateRotatingPayload('t1', new Uint8Array(32), FIXED_NOW_MS)
    ).toThrow();
  });
});

// ── Payload parsing ───────────────────────────────────────────────────────────

describe('parsePayload', () => {
  const { secretKey } = deriveTicketKeyPair(FIXED_SECRET);

  it('parses a valid payload without throwing', () => {
    const encoded = generateRotatingPayload('ticket-5', secretKey, FIXED_NOW_MS);
    expect(() => parsePayload(encoded)).not.toThrow();
  });

  it('throws on wrong byte length', () => {
    const short = toBase64Url(new Uint8Array(10));
    expect(() => parsePayload(short)).toThrow(/expected \d+ bytes/i);
  });

  it('throws on unsupported version byte', () => {
    const encoded = generateRotatingPayload('ticket-5', secretKey, FIXED_NOW_MS);
    const bytes = fromBase64Url(encoded);
    bytes[0] = 0x99; // wrong version
    expect(() => parsePayload(toBase64Url(bytes))).toThrow(/unsupported version/i);
  });

  it('signedMessage is bytes[0..41]', () => {
    const encoded = generateRotatingPayload('ticket-5', secretKey, FIXED_NOW_MS);
    const parsed = parsePayload(encoded);
    expect(parsed.signedMessage.length).toBe(PAYLOAD_OFFSETS.SIGNATURE);
  });
});

// ── Payload verification ──────────────────────────────────────────────────────

describe('verifyPayload', () => {
  const { publicKey, secretKey } = deriveTicketKeyPair(FIXED_SECRET);

  it('returns ok=true for a fresh valid payload', () => {
    const encoded = generateRotatingPayload('ticket-1', secretKey, FIXED_NOW_MS);
    const result = verifyPayload(encoded, publicKey, FIXED_NOW_MS);
    expect(result.ok).toBe(true);
    if (result.ok) {
      expect(result.ticketId).toBe('ticket-1');
    }
  });

  it('returns ok=false reason=invalid_signature for a tampered payload', () => {
    const encoded = generateRotatingPayload('ticket-1', secretKey, FIXED_NOW_MS);
    const bytes = fromBase64Url(encoded);
    // Flip one bit in the signature
    bytes[PAYLOAD_OFFSETS.SIGNATURE] ^= 0x01;
    const result = verifyPayload(toBase64Url(bytes), publicKey, FIXED_NOW_MS);
    expect(result.ok).toBe(false);
    if (!result.ok) expect(result.reason).toBe('invalid_signature');
  });

  it('returns ok=false reason=invalid_signature for the wrong public key', () => {
    const { secretKey: sk2 } = deriveTicketKeyPair(makeSecret(0x99));
    const encoded = generateRotatingPayload('ticket-1', sk2, FIXED_NOW_MS);
    // Verify with the original (different) public key
    const result = verifyPayload(encoded, publicKey, FIXED_NOW_MS);
    expect(result.ok).toBe(false);
    if (!result.ok) expect(result.reason).toBe('invalid_signature');
  });

  it('returns ok=false reason=invalid_signature when ticketId is tampered', () => {
    const encoded = generateRotatingPayload('ticket-1', secretKey, FIXED_NOW_MS);
    const bytes = fromBase64Url(encoded);
    // Overwrite the ticketId field
    bytes[PAYLOAD_OFFSETS.TICKET_ID] = 0xff;
    const result = verifyPayload(toBase64Url(bytes), publicKey, FIXED_NOW_MS);
    expect(result.ok).toBe(false);
    if (!result.ok) expect(result.reason).toBe('invalid_signature');
  });

  it('accepts a payload within the ±60s clock drift window', () => {
    // Payload generated SCANNER_CLOCK_DRIFT_S - 1 seconds ago (still valid)
    const oldNow = FIXED_NOW_MS - (SCANNER_CLOCK_DRIFT_S - 1) * 1000;
    const encoded = generateRotatingPayload('ticket-1', secretKey, oldNow);
    const result = verifyPayload(encoded, publicKey, FIXED_NOW_MS);
    expect(result.ok).toBe(true);
  });

  it('rejects a payload just outside the timestamp window', () => {
    // Payload from (PAYLOAD_WINDOW_S + SCANNER_CLOCK_DRIFT_S + 1)s ago
    const tooOldDeltaS = PAYLOAD_WINDOW_S + SCANNER_CLOCK_DRIFT_S + 1;
    // The payload timestamp is at the window boundary before the generation time,
    // so we need to go back far enough that the boundary itself is outside the window.
    const oldNow = FIXED_NOW_MS - (tooOldDeltaS + PAYLOAD_WINDOW_S) * 1000;
    const encoded = generateRotatingPayload('ticket-1', secretKey, oldNow);
    const result = verifyPayload(encoded, publicKey, FIXED_NOW_MS);
    expect(result.ok).toBe(false);
    if (!result.ok) expect(result.reason).toBe('expired_timestamp');
  });

  it('returns ok=false reason=bad_payload for garbage input', () => {
    const result = verifyPayload('not-a-valid-payload', publicKey, FIXED_NOW_MS);
    expect(result.ok).toBe(false);
    if (!result.ok) expect(result.reason).toBe('bad_payload');
  });

  it('returns ok=false reason=bad_payload for empty string', () => {
    const result = verifyPayload('', publicKey, FIXED_NOW_MS);
    expect(result.ok).toBe(false);
  });

  // Legacy/stale-key handling: a payload signed with an old key (e.g. before
  // a ticket transfer) must not verify against the new owner's public key.
  it('stale-key: old keypair payload does not verify against new keypair public key', () => {
    const oldSecret = makeSecret(0xAA);
    const newSecret = makeSecret(0xBB);
    const { secretKey: oldSk } = deriveTicketKeyPair(oldSecret);
    const { publicKey: newPk } = deriveTicketKeyPair(newSecret);

    const encoded = generateRotatingPayload('ticket-transfer', oldSk, FIXED_NOW_MS);
    const result = verifyPayload(encoded, newPk, FIXED_NOW_MS);
    expect(result.ok).toBe(false);
    if (!result.ok) expect(result.reason).toBe('invalid_signature');
  });
});
