/**
 * ticketVault.ts — Issue #1179: Offline Ticket Vault
 *
 * Encrypted offline storage of per-ticket signing secrets and their derived
 * public keys, gated behind biometric authentication.
 *
 * ## Storage model
 *
 * Ticket secrets are stored in expo-secure-store (iOS Keychain /
 * Android Keystore), which provides hardware-backed encryption on supported
 * devices. SecureStore is the correct storage layer here because:
 *
 *   - iOS: values are stored in the Keychain, which can be configured with
 *     kSecAttrAccessibleWhenPasscodeSetThisDeviceOnly to require device
 *     passcode/biometric before reading. expo-secure-store supports this via
 *     the `requireAuthentication` option (iOS only, requires biometric
 *     enrollment to set the item).
 *   - Android: values are encrypted with AES-256-GCM using a key from the
 *     Android Keystore hardware-backed key store.
 *
 * On iOS, we set requireAuthentication: true to delegate biometric prompting
 * to the OS. On Android, biometric authentication is handled explicitly in the
 * UI layer via expo-local-authentication before calling getTicketSecret().
 *
 * ## What is stored
 *
 * Key: `agora.vault.secret.<paymentId>`  (per-ticket, biometric-gated)
 * Value: base64url-encoded raw 32-byte purchase secret
 *
 * Key: `agora.vault.pubkey.<paymentId>`  (per-ticket public key, no biometric needed)
 * Value: base64url-encoded 32-byte Ed25519 public key (safe to read freely)
 *
 * The private key is never written to disk; it is re-derived from the secret
 * on demand using deriveTicketKeyPair() and used only in memory.
 *
 * ## Plaintext-key guarantee
 *
 * This module never writes plaintext ticket secrets to:
 *   - AsyncStorage (unencrypted)
 *   - SQLite (unencrypted)
 *   - The filesystem via RNFS or similar
 *   - React state that persists across app backgrounding
 *
 * The only write path is SecureStore.setItemAsync(), which encrypts on write.
 *
 * ## Revocation / staleness note
 *
 * See docs/offline-vault-revocation.md for the full tradeoff analysis.
 * Short version: a refunded or cancelled ticket synced to the server *after*
 * the device's last sync will still scan as valid offline until the scanner
 * pulls a fresh public-key / revocation list. Gate staff should be informed
 * of the last-sync time so they can decide whether to accept borderline cases.
 */

import * as SecureStore from 'expo-secure-store';
import { deriveTicketKeyPair, toBase64Url, fromBase64Url } from '../lib/crypto';

// ── Key name helpers ──────────────────────────────────────────────────────────

const SECRET_KEY_PREFIX = 'agora.vault.secret.';
const PUBKEY_KEY_PREFIX = 'agora.vault.pubkey.';

function secretStoreKey(paymentId: string): string {
  return `${SECRET_KEY_PREFIX}${paymentId}`;
}
function pubkeyStoreKey(paymentId: string): string {
  return `${PUBKEY_KEY_PREFIX}${paymentId}`;
}

// ── Platform options ──────────────────────────────────────────────────────────

/**
 * SecureStore options for biometric-gated items (iOS only: requireAuthentication).
 * On Android, authentication must be performed externally via
 * expo-local-authentication before calling getTicketSecret().
 */
const BIOMETRIC_STORE_OPTIONS: SecureStore.SecureStoreOptions = {
  keychainService: 'agora_ticket_vault',
  // On iOS: this causes the Keychain item to require biometric/passcode before
  // being read. The OS presents its own native prompt; we don't draw our own UI
  // for this path.
  requireAuthentication: true,
};

/** Options for the public-key store — no biometric required, freely readable. */
const PUBLIC_STORE_OPTIONS: SecureStore.SecureStoreOptions = {
  keychainService: 'agora_ticket_vault_pub',
};

// ── Vault write ───────────────────────────────────────────────────────────────

/**
 * Stores a ticket secret in the hardware-backed vault.
 *
 * Also pre-computes and stores the derived Ed25519 public key so it can be
 * retrieved freely (e.g. to send to the event organizer for scanner sync)
 * without requiring biometric authentication.
 *
 * This is called once, right after `purchaseTickets()` resolves successfully.
 * The plaintext `secretBytes` must never be stored elsewhere; after this call
 * the caller should zero the local reference if possible.
 *
 * @param paymentId   Unique payment identifier returned by the contract.
 * @param secretBytes Raw 32-byte purchase secret from `generatePurchaseSecret()`.
 */
export async function storeTicketSecret(
  paymentId: string,
  secretBytes: Uint8Array
): Promise<void> {
  if (secretBytes.length !== 32) {
    throw new Error(`storeTicketSecret: secret must be 32 bytes, got ${secretBytes.length}.`);
  }

  // Derive the public key before writing so both writes succeed atomically (or
  // the first write hasn't happened yet if derivation throws).
  const { publicKey } = deriveTicketKeyPair(secretBytes);

  await SecureStore.setItemAsync(
    secretStoreKey(paymentId),
    toBase64Url(secretBytes),
    BIOMETRIC_STORE_OPTIONS
  );

  // Public key is stored separately without biometric so it can be read at
  // any time (e.g. during scanner-sync export).
  await SecureStore.setItemAsync(
    pubkeyStoreKey(paymentId),
    toBase64Url(publicKey),
    PUBLIC_STORE_OPTIONS
  );
}

// ── Vault read ────────────────────────────────────────────────────────────────

/**
 * Retrieves the raw 32-byte purchase secret for a ticket.
 *
 * On iOS, the OS presents a biometric / passcode prompt before returning the
 * value (because the item was stored with requireAuthentication: true).
 *
 * On Android, the caller MUST have already called
 * `authenticateWithBiometrics()` and received a successful result before
 * calling this function; expo-secure-store on Android does not perform its
 * own biometric prompt.
 *
 * Returns `null` if no secret is stored for this paymentId (e.g. the ticket
 * was purchased on a different device, or the vault was cleared after a
 * sign-out).
 *
 * @throws if the biometric prompt is dismissed or fails (iOS), or if
 *   SecureStore is unavailable.
 */
export async function getTicketSecret(paymentId: string): Promise<Uint8Array | null> {
  const stored = await SecureStore.getItemAsync(
    secretStoreKey(paymentId),
    BIOMETRIC_STORE_OPTIONS
  );
  if (!stored) return null;
  const bytes = fromBase64Url(stored);
  if (bytes.length !== 32) {
    throw new Error(
      `getTicketSecret: stored value for ${paymentId} has wrong length: ${bytes.length}.`
    );
  }
  return bytes;
}

/**
 * Retrieves the Ed25519 public key for a ticket *without* requiring biometrics.
 *
 * This is used to export the public key to the event organizer for scanner
 * pre-sync, and by the scanner verification flow on a gate device.
 *
 * Returns `null` if no public key is stored for this paymentId.
 */
export async function getTicketPublicKey(paymentId: string): Promise<Uint8Array | null> {
  const stored = await SecureStore.getItemAsync(
    pubkeyStoreKey(paymentId),
    PUBLIC_STORE_OPTIONS
  );
  if (!stored) return null;
  return fromBase64Url(stored);
}

// ── Vault delete ──────────────────────────────────────────────────────────────

/**
 * Permanently removes a ticket's secret and public key from the vault.
 *
 * Call this after a successful resale (the new owner needs to receive the
 * secret via `resaleCrypto.sealTicketSecret` before the seller calls this)
 * or after the event ends and the device no longer needs to show the ticket.
 *
 * This does NOT require biometric authentication — deletion is a less
 * sensitive operation than reading.
 */
export async function clearTicketSecret(paymentId: string): Promise<void> {
  await Promise.all([
    SecureStore.deleteItemAsync(secretStoreKey(paymentId), BIOMETRIC_STORE_OPTIONS),
    SecureStore.deleteItemAsync(pubkeyStoreKey(paymentId), PUBLIC_STORE_OPTIONS),
  ]);
}

// ── Vault availability check ──────────────────────────────────────────────────

/**
 * Returns true if SecureStore is available and usable on this device.
 * Call this before attempting vault operations to surface a clear error
 * rather than a cryptic SecureStore exception.
 */
export async function isVaultAvailable(): Promise<boolean> {
  return SecureStore.isAvailableAsync();
}
