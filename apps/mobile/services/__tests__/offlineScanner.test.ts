/**
 * offlineScanner.test.ts — Issue #1179: Offline Ticket Vault
 *
 * Tests for:
 *   - VALID scan path
 *   - INVALID_SIGNATURE rejection
 *   - EXPIRED_TIMESTAMP rejection
 *   - ALREADY_SCANNED duplicate-scan rejection (distinct from invalid/expired)
 *   - BAD_PAYLOAD rejection
 *   - UNKNOWN_TICKET rejection
 *   - Scan log recording and retrieval
 *   - Partial sync (addPublicKey)
 *   - Legacy/stale-key handling
 */

import {
  OfflineScanner,
  InMemoryScanStore,
  ScanResult,
  ScanRecord,
} from '../../services/offlineScanner';
import {
  deriveTicketKeyPair,
  generateRotatingPayload,
  toBase64Url,
  fromBase64Url,
  PAYLOAD_OFFSETS,
  PAYLOAD_WINDOW_S,
  SCANNER_CLOCK_DRIFT_S,
} from '../../lib/crypto';

// ── Fixtures ──────────────────────────────────────────────────────────────────

function makeSecret(fill: number): Uint8Array {
  return new Uint8Array(32).fill(fill);
}

const FIXED_NOW_MS = 1_750_000_000_000;

function buildScanner(
  tickets: Array<{ ticketId: string; secretFill: number }>,
  store = new InMemoryScanStore()
): { scanner: OfflineScanner; keypairs: Map<string, ReturnType<typeof deriveTicketKeyPair>> } {
  const keyMap = new Map<string, string>();
  const keypairs = new Map<string, ReturnType<typeof deriveTicketKeyPair>>();

  for (const { ticketId, secretFill } of tickets) {
    const kp = deriveTicketKeyPair(makeSecret(secretFill));
    keypairs.set(ticketId, kp);
    keyMap.set(ticketId, toBase64Url(kp.publicKey));
  }

  return { scanner: new OfflineScanner(keyMap, store), keypairs };
}

// ── VALID scan ────────────────────────────────────────────────────────────────

describe('VALID scan', () => {
  it('returns VALID for a fresh signed payload', async () => {
    const { scanner, keypairs } = buildScanner([{ ticketId: 'ticket-1', secretFill: 0x11 }]);
    const { secretKey } = keypairs.get('ticket-1')!;
    const payload = generateRotatingPayload('ticket-1', secretKey, FIXED_NOW_MS);

    const outcome = await scanner.scan(payload, FIXED_NOW_MS);
    expect(outcome.result).toBe(ScanResult.VALID);
    expect(outcome.ticketId).toBe('ticket-1');
    expect(outcome.record).not.toBeNull();
    expect(outcome.record?.scannedAtS).toBe(Math.floor(FIXED_NOW_MS / 1000));
  });

  it('records the scan in the log', async () => {
    const { scanner, keypairs } = buildScanner([{ ticketId: 'ticket-2', secretFill: 0x22 }]);
    const { secretKey } = keypairs.get('ticket-2')!;
    const payload = generateRotatingPayload('ticket-2', secretKey, FIXED_NOW_MS);

    await scanner.scan(payload, FIXED_NOW_MS);
    const log = await scanner.getScanLog();
    expect(log.length).toBe(1);
    expect(log[0].ticketId).toBe('ticket-2');
  });

  it('produces a message mentioning the ticketId', async () => {
    const { scanner, keypairs } = buildScanner([{ ticketId: 'my-ticket', secretFill: 0x33 }]);
    const { secretKey } = keypairs.get('my-ticket')!;
    const payload = generateRotatingPayload('my-ticket', secretKey, FIXED_NOW_MS);
    const outcome = await scanner.scan(payload, FIXED_NOW_MS);
    expect(outcome.message).toContain('my-ticket');
  });
});

// ── INVALID_SIGNATURE ─────────────────────────────────────────────────────────

describe('INVALID_SIGNATURE', () => {
  it('rejects a payload with a tampered signature byte', async () => {
    const { scanner, keypairs } = buildScanner([{ ticketId: 'ticket-3', secretFill: 0x44 }]);
    const { secretKey } = keypairs.get('ticket-3')!;
    const payload = generateRotatingPayload('ticket-3', secretKey, FIXED_NOW_MS);
    const bytes = fromBase64Url(payload);
    bytes[PAYLOAD_OFFSETS.SIGNATURE] ^= 0xff;
    const tampered = toBase64Url(bytes);

    const outcome = await scanner.scan(tampered, FIXED_NOW_MS);
    expect(outcome.result).toBe(ScanResult.INVALID_SIGNATURE);
    expect(outcome.record).toBeNull();
  });

  it('rejects a payload signed by the wrong keypair', async () => {
    const { scanner } = buildScanner([{ ticketId: 'ticket-4', secretFill: 0x55 }]);
    const wrongKp = deriveTicketKeyPair(makeSecret(0x99));
    const payload = generateRotatingPayload('ticket-4', wrongKp.secretKey, FIXED_NOW_MS);

    const outcome = await scanner.scan(payload, FIXED_NOW_MS);
    expect(outcome.result).toBe(ScanResult.INVALID_SIGNATURE);
  });

  it('does not record the ticket in the scan log on invalid signature', async () => {
    const { scanner, keypairs } = buildScanner([{ ticketId: 'ticket-5', secretFill: 0x56 }]);
    const { secretKey } = keypairs.get('ticket-5')!;
    const payload = generateRotatingPayload('ticket-5', secretKey, FIXED_NOW_MS);
    const bytes = fromBase64Url(payload);
    bytes[PAYLOAD_OFFSETS.SIGNATURE] ^= 0x01;

    await scanner.scan(toBase64Url(bytes), FIXED_NOW_MS);
    const log = await scanner.getScanLog();
    expect(log.length).toBe(0);
  });

  it('INVALID_SIGNATURE is distinct from ALREADY_SCANNED', async () => {
    const { scanner, keypairs } = buildScanner([{ ticketId: 'ticket-6', secretFill: 0x66 }]);
    const { secretKey } = keypairs.get('ticket-6')!;
    const validPayload = generateRotatingPayload('ticket-6', secretKey, FIXED_NOW_MS);
    await scanner.scan(validPayload, FIXED_NOW_MS);

    // Now send a tampered payload for the same ticket
    const bytes = fromBase64Url(validPayload);
    bytes[PAYLOAD_OFFSETS.SIGNATURE] ^= 0x01;
    const outcome = await scanner.scan(toBase64Url(bytes), FIXED_NOW_MS);
    // Should still be INVALID_SIGNATURE (signature check happens before dup check)
    expect(outcome.result).toBe(ScanResult.INVALID_SIGNATURE);
  });
});

// ── EXPIRED_TIMESTAMP ─────────────────────────────────────────────────────────

describe('EXPIRED_TIMESTAMP', () => {
  it('rejects a payload outside the clock drift window', async () => {
    const { scanner, keypairs } = buildScanner([{ ticketId: 'ticket-7', secretFill: 0x77 }]);
    const { secretKey } = keypairs.get('ticket-7')!;

    // Generate a payload that is (PAYLOAD_WINDOW_S + SCANNER_CLOCK_DRIFT_S + 2) seconds
    // in the past relative to the scanner's clock.
    const pastNow =
      FIXED_NOW_MS - (PAYLOAD_WINDOW_S + SCANNER_CLOCK_DRIFT_S + 2) * 1000 -
      PAYLOAD_WINDOW_S * 1000;
    const payload = generateRotatingPayload('ticket-7', secretKey, pastNow);

    const outcome = await scanner.scan(payload, FIXED_NOW_MS);
    expect(outcome.result).toBe(ScanResult.EXPIRED_TIMESTAMP);
    expect(outcome.record).toBeNull();
  });

  it('accepts a payload within the drift window', async () => {
    const { scanner, keypairs } = buildScanner([{ ticketId: 'ticket-8', secretFill: 0x88 }]);
    const { secretKey } = keypairs.get('ticket-8')!;

    // Payload generated (SCANNER_CLOCK_DRIFT_S - 5) seconds ago — still within window
    const slightlyOldNow = FIXED_NOW_MS - (SCANNER_CLOCK_DRIFT_S - 5) * 1000;
    const payload = generateRotatingPayload('ticket-8', secretKey, slightlyOldNow);

    const outcome = await scanner.scan(payload, FIXED_NOW_MS);
    expect(outcome.result).toBe(ScanResult.VALID);
  });

  it('EXPIRED_TIMESTAMP is distinct from ALREADY_SCANNED', async () => {
    const { scanner, keypairs } = buildScanner([{ ticketId: 'ticket-9', secretFill: 0x99 }]);
    const { secretKey } = keypairs.get('ticket-9')!;

    const oldNow = FIXED_NOW_MS - (PAYLOAD_WINDOW_S + SCANNER_CLOCK_DRIFT_S + 100) * 1000;
    const payload = generateRotatingPayload('ticket-9', secretKey, oldNow);
    const first = await scanner.scan(payload, FIXED_NOW_MS);
    expect(first.result).toBe(ScanResult.EXPIRED_TIMESTAMP);

    // A second scan of the same expired payload must still be EXPIRED_TIMESTAMP,
    // not ALREADY_SCANNED, because it was never recorded.
    const second = await scanner.scan(payload, FIXED_NOW_MS);
    expect(second.result).toBe(ScanResult.EXPIRED_TIMESTAMP);
  });
});

// ── ALREADY_SCANNED (double-scan prevention) ──────────────────────────────────

describe('ALREADY_SCANNED', () => {
  it('rejects a second scan of the same ticketId', async () => {
    const { scanner, keypairs } = buildScanner([{ ticketId: 'dup-ticket', secretFill: 0xAA }]);
    const { secretKey } = keypairs.get('dup-ticket')!;
    const payload1 = generateRotatingPayload('dup-ticket', secretKey, FIXED_NOW_MS);
    const payload2 = generateRotatingPayload('dup-ticket', secretKey, FIXED_NOW_MS + 16_000);

    const first = await scanner.scan(payload1, FIXED_NOW_MS);
    expect(first.result).toBe(ScanResult.VALID);

    const second = await scanner.scan(payload2, FIXED_NOW_MS + 16_000);
    expect(second.result).toBe(ScanResult.ALREADY_SCANNED);
  });

  it('includes the original scan time in the duplicate result', async () => {
    const { scanner, keypairs } = buildScanner([{ ticketId: 'dup-time', secretFill: 0xAB }]);
    const { secretKey } = keypairs.get('dup-time')!;
    const payload = generateRotatingPayload('dup-time', secretKey, FIXED_NOW_MS);
    await scanner.scan(payload, FIXED_NOW_MS);

    const later = FIXED_NOW_MS + 30_000;
    const laterPayload = generateRotatingPayload('dup-time', secretKey, later);
    const outcome = await scanner.scan(laterPayload, later);
    expect(outcome.result).toBe(ScanResult.ALREADY_SCANNED);
    expect(outcome.record?.scannedAtS).toBe(Math.floor(FIXED_NOW_MS / 1000));
  });

  it('ALREADY_SCANNED message is distinct from INVALID_SIGNATURE message', async () => {
    const { scanner, keypairs } = buildScanner([{ ticketId: 'msg-test', secretFill: 0xCC }]);
    const { secretKey } = keypairs.get('msg-test')!;
    const payload1 = generateRotatingPayload('msg-test', secretKey, FIXED_NOW_MS);
    const payload2 = generateRotatingPayload('msg-test', secretKey, FIXED_NOW_MS + 20_000);

    await scanner.scan(payload1, FIXED_NOW_MS);
    const dup = await scanner.scan(payload2, FIXED_NOW_MS + 20_000);
    const invalidPayload = (() => {
      const b = fromBase64Url(payload2);
      b[PAYLOAD_OFFSETS.SIGNATURE] ^= 1;
      return toBase64Url(b);
    })();

    const invalid = await scanner.scan(invalidPayload, FIXED_NOW_MS);
    expect(dup.message).not.toBe(invalid.message);
    expect(dup.result).toBe(ScanResult.ALREADY_SCANNED);
    expect(invalid.result).toBe(ScanResult.INVALID_SIGNATURE);
  });

  it('does not add a second record on duplicate scan', async () => {
    const { scanner, keypairs } = buildScanner([{ ticketId: 'no-double', secretFill: 0xDD }]);
    const { secretKey } = keypairs.get('no-double')!;
    const p1 = generateRotatingPayload('no-double', secretKey, FIXED_NOW_MS);
    const p2 = generateRotatingPayload('no-double', secretKey, FIXED_NOW_MS + 20_000);

    await scanner.scan(p1, FIXED_NOW_MS);
    await scanner.scan(p2, FIXED_NOW_MS + 20_000);
    const log = await scanner.getScanLog();
    expect(log.filter(r => r.ticketId === 'no-double')).toHaveLength(1);
  });

  it('two different ticketIds can both be VALID', async () => {
    const { scanner, keypairs } = buildScanner([
      { ticketId: 'ticket-A', secretFill: 0x01 },
      { ticketId: 'ticket-B', secretFill: 0x02 },
    ]);
    const p1 = generateRotatingPayload('ticket-A', keypairs.get('ticket-A')!.secretKey, FIXED_NOW_MS);
    const p2 = generateRotatingPayload('ticket-B', keypairs.get('ticket-B')!.secretKey, FIXED_NOW_MS);

    const r1 = await scanner.scan(p1, FIXED_NOW_MS);
    const r2 = await scanner.scan(p2, FIXED_NOW_MS);
    expect(r1.result).toBe(ScanResult.VALID);
    expect(r2.result).toBe(ScanResult.VALID);
  });
});

// ── BAD_PAYLOAD ───────────────────────────────────────────────────────────────

describe('BAD_PAYLOAD', () => {
  it('rejects an empty string', async () => {
    const { scanner } = buildScanner([]);
    const outcome = await scanner.scan('', FIXED_NOW_MS);
    expect(outcome.result).toBe(ScanResult.BAD_PAYLOAD);
    expect(outcome.ticketId).toBeNull();
  });

  it('rejects a non-base64url string', async () => {
    const { scanner } = buildScanner([]);
    const outcome = await scanner.scan('not-a-ticket!!!', FIXED_NOW_MS);
    expect(outcome.result).toBe(ScanResult.BAD_PAYLOAD);
  });

  it('rejects a valid base64url string of wrong length', async () => {
    const { scanner } = buildScanner([]);
    const outcome = await scanner.scan(toBase64Url(new Uint8Array(50)), FIXED_NOW_MS);
    expect(outcome.result).toBe(ScanResult.BAD_PAYLOAD);
  });
});

// ── UNKNOWN_TICKET ────────────────────────────────────────────────────────────

describe('UNKNOWN_TICKET', () => {
  it('rejects a valid payload for a ticket not in the key map', async () => {
    // Scanner has no registered keys
    const { scanner: emptyScanner } = buildScanner([]);
    const kp = deriveTicketKeyPair(makeSecret(0x50));
    const payload = generateRotatingPayload('unregistered', kp.secretKey, FIXED_NOW_MS);

    const outcome = await emptyScanner.scan(payload, FIXED_NOW_MS);
    expect(outcome.result).toBe(ScanResult.UNKNOWN_TICKET);
    expect(outcome.ticketId).toBe('unregistered');
  });
});

// ── Scan log ──────────────────────────────────────────────────────────────────

describe('scan log', () => {
  it('returns an empty log initially', async () => {
    const { scanner } = buildScanner([]);
    expect(await scanner.getScanLog()).toEqual([]);
  });

  it('records multiple valid scans', async () => {
    const { scanner, keypairs } = buildScanner([
      { ticketId: 'log-1', secretFill: 0x10 },
      { ticketId: 'log-2', secretFill: 0x20 },
    ]);
    const p1 = generateRotatingPayload('log-1', keypairs.get('log-1')!.secretKey, FIXED_NOW_MS);
    const p2 = generateRotatingPayload('log-2', keypairs.get('log-2')!.secretKey, FIXED_NOW_MS + 5000);

    await scanner.scan(p1, FIXED_NOW_MS);
    await scanner.scan(p2, FIXED_NOW_MS + 5000);

    const log = await scanner.getScanLog();
    expect(log.length).toBe(2);
    expect(log.map(r => r.ticketId).sort()).toEqual(['log-1', 'log-2']);
  });

  it('clearScanLog empties the log', async () => {
    const { scanner, keypairs } = buildScanner([{ ticketId: 'clear-me', secretFill: 0x30 }]);
    const p = generateRotatingPayload('clear-me', keypairs.get('clear-me')!.secretKey, FIXED_NOW_MS);
    await scanner.scan(p, FIXED_NOW_MS);
    await scanner.clearScanLog();
    expect(await scanner.getScanLog()).toEqual([]);
  });
});

// ── addPublicKey (partial sync) ───────────────────────────────────────────────

describe('addPublicKey', () => {
  it('allows scanning a ticket added after construction', async () => {
    const { scanner } = buildScanner([]);
    const kp = deriveTicketKeyPair(makeSecret(0x60));
    scanner.addPublicKey('late-ticket', toBase64Url(kp.publicKey));

    const payload = generateRotatingPayload('late-ticket', kp.secretKey, FIXED_NOW_MS);
    const outcome = await scanner.scan(payload, FIXED_NOW_MS);
    expect(outcome.result).toBe(ScanResult.VALID);
  });
});

// ── Legacy / stale key handling ───────────────────────────────────────────────

describe('legacy/stale-key handling', () => {
  // NOTE: ticketId in the payload is truncated to 16 bytes. Keys must match.
  const TICKET_ID = 'xfer-ticket-01'; // 14 bytes — safely under the 16-byte limit

  it('old keypair payload returns INVALID_SIGNATURE when scanner has new key for the same ticketId', async () => {
    const oldKp = deriveTicketKeyPair(makeSecret(0x70));
    const newKp = deriveTicketKeyPair(makeSecret(0x71));

    // Scanner loaded with the NEW public key for TICKET_ID.
    const keyMap = new Map([[TICKET_ID, toBase64Url(newKp.publicKey)]]);
    const scanner = new OfflineScanner(keyMap);

    // Attendee presents a payload signed with the OLD key.
    // ticketId IS in the key map (so UNKNOWN_TICKET is not returned),
    // but the signature does not verify against the new public key.
    const oldPayload = generateRotatingPayload(TICKET_ID, oldKp.secretKey, FIXED_NOW_MS);
    const outcome = await scanner.scan(oldPayload, FIXED_NOW_MS);
    expect(outcome.result).toBe(ScanResult.INVALID_SIGNATURE);
  });

  it('new keypair payload verifies after scanner is loaded with new key', async () => {
    const newKp = deriveTicketKeyPair(makeSecret(0x71));
    const keyMap = new Map([[TICKET_ID, toBase64Url(newKp.publicKey)]]);
    const scanner = new OfflineScanner(keyMap);

    const newPayload = generateRotatingPayload(TICKET_ID, newKp.secretKey, FIXED_NOW_MS);
    const outcome = await scanner.scan(newPayload, FIXED_NOW_MS);
    expect(outcome.result).toBe(ScanResult.VALID);
  });

  it('UNKNOWN_TICKET is distinct from INVALID_SIGNATURE: unregistered ticket returns UNKNOWN_TICKET', async () => {
    const kp = deriveTicketKeyPair(makeSecret(0x72));
    const scanner = new OfflineScanner(new Map()); // no registered keys
    const payload = generateRotatingPayload('unreg-2', kp.secretKey, FIXED_NOW_MS);
    const outcome = await scanner.scan(payload, FIXED_NOW_MS);
    expect(outcome.result).toBe(ScanResult.UNKNOWN_TICKET);
  });
});

// ── InMemoryScanStore ─────────────────────────────────────────────────────────

describe('InMemoryScanStore', () => {
  it('has/get/set/getAll/clear work correctly', async () => {
    const store = new InMemoryScanStore();
    expect(await store.has('x')).toBe(false);
    expect(await store.get('x')).toBeNull();

    const record: ScanRecord = { ticketId: 'x', scannedAtS: 100, payloadTimestampS: 90 };
    await store.set('x', record);
    expect(await store.has('x')).toBe(true);
    expect(await store.get('x')).toEqual(record);

    const all = await store.getAll();
    expect(all).toHaveLength(1);

    await store.clear();
    expect(await store.has('x')).toBe(false);
    expect(store.size).toBe(0);
  });
});
