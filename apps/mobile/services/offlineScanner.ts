/**
 * offlineScanner.ts — Issue #1179: Offline Ticket Vault
 *
 * Offline ticket scanner verification module. Runs entirely on the gate
 * device with no network calls required.
 *
 * ## Responsibilities
 *
 *   1. Parse a rotating QR binary payload from `lib/crypto.ts`.
 *   2. Verify the Ed25519 signature against the event's pre-synced public key.
 *   3. Enforce a ±60s timestamp window (SCANNER_CLOCK_DRIFT_S) to tolerate
 *      clock drift between the attendee's phone and the scanner device.
 *   4. Check the ticket ID against a local scan log for double-scan prevention.
 *   5. Record valid scans to the local scan log (in-memory + optional persist).
 *
 * ## Double-scan prevention
 *
 * Scanned ticket IDs are stored in a Map<ticketId, ScanRecord> in memory and
 * optionally persisted to a compact on-device store (see ScanStore interface).
 * A second scan of the same ticketId returns ScanResult.ALREADY_SCANNED with
 * the original scan timestamp, which is a distinct result from
 * INVALID_SIGNATURE or EXPIRED_TIMESTAMP — the gate staff UI must show the
 * right message for each.
 *
 * ## Event public key management
 *
 * The scanner holds one Ed25519 public key per ticket. In the simplest
 * deployment model, the organizer exports a map of { paymentId → publicKeyB64 }
 * at event setup and loads it into the scanner app. The scanner only looks up
 * the public key by the ticketId field embedded in the payload.
 *
 * ## Staleness / revocation
 *
 * See docs/offline-vault-revocation.md. If a ticket is cancelled server-side
 * after the scanner's last sync, it will still scan as VALID here until the
 * scanner pulls a fresh key/revocation list. Scanners should display their
 * last-sync time prominently so staff can make informed decisions.
 *
 * ## Schema (in-memory scan log)
 *
 *   ScanRecord {
 *     ticketId:   string   — ticket identifier from the payload
 *     scannedAtS: number   — Unix seconds when first scanned
 *     payloadTs:  number   — payload's own timestamp (window boundary)
 *   }
 */

import nacl from 'tweetnacl';
import {
  verifyPayload,
  parsePayload,
  fromBase64Url,
  SCANNER_CLOCK_DRIFT_S,
  PAYLOAD_WINDOW_S,
  VerifyResult,
} from '../lib/crypto';

// ── Scan result enum ──────────────────────────────────────────────────────────

export enum ScanResult {
  /** Signature valid, timestamp in window, not a duplicate. */
  VALID = 'VALID',
  /** Ed25519 signature did not verify against the event public key. */
  INVALID_SIGNATURE = 'INVALID_SIGNATURE',
  /** Payload timestamp is outside the ±SCANNER_CLOCK_DRIFT_S window. */
  EXPIRED_TIMESTAMP = 'EXPIRED_TIMESTAMP',
  /** This ticket ID has already been scanned at this event. */
  ALREADY_SCANNED = 'ALREADY_SCANNED',
  /** Payload could not be parsed (wrong format, wrong length, unsupported version). */
  BAD_PAYLOAD = 'BAD_PAYLOAD',
  /** No public key registered for this ticketId in the event key map. */
  UNKNOWN_TICKET = 'UNKNOWN_TICKET',
}

// ── Data types ────────────────────────────────────────────────────────────────

export interface ScanRecord {
  ticketId: string;
  /** Unix seconds when this ticket was first scanned. */
  scannedAtS: number;
  /** The payload's own window-boundary timestamp. */
  payloadTimestampS: number;
}

export interface ScanOutcome {
  result: ScanResult;
  /** Populated for all results where the ticketId could be parsed. */
  ticketId: string | null;
  /** Populated for VALID and ALREADY_SCANNED. */
  record: ScanRecord | null;
  /** Human-readable message for gate staff UI. */
  message: string;
}

/**
 * Interface for persisting the scan log between app restarts.
 * A concrete implementation using AsyncStorage or SQLite can be injected
 * by the scanner app. If omitted, the scanner uses in-memory storage only.
 */
export interface ScanStore {
  has(ticketId: string): Promise<boolean>;
  get(ticketId: string): Promise<ScanRecord | null>;
  set(ticketId: string, record: ScanRecord): Promise<void>;
  /** Returns all scanned records, ordered by scannedAtS ascending. */
  getAll(): Promise<ScanRecord[]>;
  /** Removes all records (for post-event cleanup). */
  clear(): Promise<void>;
}

// ── In-memory scan store ──────────────────────────────────────────────────────

/**
 * Default in-memory scan store. Records are lost when the app restarts.
 * Suitable for short-duration events or when the scanner app is restarted
 * between events. For multi-session events, inject a persistent ScanStore.
 */
export class InMemoryScanStore implements ScanStore {
  private readonly records = new Map<string, ScanRecord>();

  async has(ticketId: string): Promise<boolean> {
    return this.records.has(ticketId);
  }

  async get(ticketId: string): Promise<ScanRecord | null> {
    return this.records.get(ticketId) ?? null;
  }

  async set(ticketId: string, record: ScanRecord): Promise<void> {
    this.records.set(ticketId, record);
  }

  async getAll(): Promise<ScanRecord[]> {
    return Array.from(this.records.values()).sort(
      (a, b) => a.scannedAtS - b.scannedAtS
    );
  }

  async clear(): Promise<void> {
    this.records.clear();
  }

  /** Exposed for testing only. */
  get size(): number {
    return this.records.size;
  }
}

// ── Scanner class ─────────────────────────────────────────────────────────────

/**
 * OfflineScanner manages an event's public key map and scan log.
 *
 * Usage:
 *   const scanner = new OfflineScanner(publicKeyMap);
 *   const outcome = await scanner.scan(qrPayloadString);
 *   switch (outcome.result) { ... }
 */
export class OfflineScanner {
  /**
   * Map from ticketId (string) to 32-byte Ed25519 public key.
   * Pre-synced from the organizer's key export before the event.
   */
  private readonly publicKeys: Map<string, Uint8Array>;
  private readonly store: ScanStore;

  /**
   * @param publicKeyMap  Map<ticketId, publicKeyBase64> — the event's key export.
   *   Keys are base64url-encoded 32-byte Ed25519 public keys.
   * @param store         Optional persistent scan store. Defaults to in-memory.
   */
  constructor(
    publicKeyMap: Map<string, string>,
    store: ScanStore = new InMemoryScanStore()
  ) {
    this.store = store;
    this.publicKeys = new Map();
    for (const [ticketId, pubKeyB64] of publicKeyMap) {
      this.publicKeys.set(ticketId, fromBase64Url(pubKeyB64));
    }
  }

  /**
   * Scans a QR payload string and returns a ScanOutcome.
   *
   * @param encoded   Base64url payload from the QR code.
   * @param nowMs     Current scanner time (defaults to Date.now()).
   */
  async scan(encoded: string, nowMs: number = Date.now()): Promise<ScanOutcome> {
    // Step 1: parse the payload
    let parsed: ReturnType<typeof parsePayload>;
    try {
      parsed = parsePayload(encoded);
    } catch {
      return {
        result: ScanResult.BAD_PAYLOAD,
        ticketId: null,
        record: null,
        message: 'This QR code is not a valid Agora ticket payload.',
      };
    }

    const { ticketId, timestampS } = parsed;

    // Step 2: look up the public key for this ticket
    const publicKey = this.publicKeys.get(ticketId);
    if (!publicKey) {
      return {
        result: ScanResult.UNKNOWN_TICKET,
        ticketId,
        record: null,
        message: `Ticket ${ticketId} is not registered for this event. Sync the scanner before the event.`,
      };
    }

    // Step 3: verify the Ed25519 signature
    const verifyResult: VerifyResult = verifyPayload(encoded, publicKey, nowMs);
    if (!verifyResult.ok) {
      if (verifyResult.reason === 'invalid_signature') {
        return {
          result: ScanResult.INVALID_SIGNATURE,
          ticketId,
          record: null,
          message: `INVALID TICKET: The signature on ticket ${ticketId} could not be verified. Do not admit.`,
        };
      }
      if (verifyResult.reason === 'expired_timestamp') {
        return {
          result: ScanResult.EXPIRED_TIMESTAMP,
          ticketId,
          record: null,
          message: `EXPIRED QR CODE: The QR code for ticket ${ticketId} has expired. Ask the attendee to refresh their ticket screen.`,
        };
      }
      // bad_payload already handled above
      return {
        result: ScanResult.BAD_PAYLOAD,
        ticketId,
        record: null,
        message: 'This QR code could not be verified.',
      };
    }

    // Step 4: double-scan check
    const existing = await this.store.get(ticketId);
    if (existing) {
      return {
        result: ScanResult.ALREADY_SCANNED,
        ticketId,
        record: existing,
        message: `DUPLICATE SCAN: Ticket ${ticketId} was already admitted at ${formatTime(existing.scannedAtS)}. Do not admit again.`,
      };
    }

    // Step 5: record the valid scan
    const nowS = Math.floor(nowMs / 1000);
    const record: ScanRecord = {
      ticketId,
      scannedAtS: nowS,
      payloadTimestampS: timestampS,
    };
    await this.store.set(ticketId, record);

    return {
      result: ScanResult.VALID,
      ticketId,
      record,
      message: `✓ ADMIT: Ticket ${ticketId} is valid.`,
    };
  }

  /**
   * Returns all scan records from the log.
   */
  async getScanLog(): Promise<ScanRecord[]> {
    return this.store.getAll();
  }

  /**
   * Clears the scan log (e.g. between events).
   */
  async clearScanLog(): Promise<void> {
    return this.store.clear();
  }

  /**
   * Returns the number of tickets registered in the event key map.
   */
  get registeredTicketCount(): number {
    return this.publicKeys.size;
  }

  /**
   * Adds or updates a public key in the key map (e.g. when doing a partial
   * sync during the event). Does NOT affect the scan log.
   *
   * @param ticketId     Ticket identifier.
   * @param pubKeyB64    Base64url-encoded 32-byte Ed25519 public key.
   */
  addPublicKey(ticketId: string, pubKeyB64: string): void {
    this.publicKeys.set(ticketId, fromBase64Url(pubKeyB64));
  }
}

// ── Helpers ───────────────────────────────────────────────────────────────────

function formatTime(unixS: number): string {
  return new Date(unixS * 1000).toLocaleTimeString([], {
    hour: '2-digit',
    minute: '2-digit',
    second: '2-digit',
  });
}
