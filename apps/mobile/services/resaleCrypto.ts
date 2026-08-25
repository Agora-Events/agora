/**
 * resaleCrypto.ts
 *
 * End-to-end encryption for handing a ticket's check-in secret from seller to
 * buyer during a resale (issue #1184).
 *
 * ## Why this exists
 *
 * Buying a ticket on-chain moves ownership, but it does not get the buyer
 * through the gate. Check-in requires the raw secret whose SHA-256 digest the
 * `ticket_payment` contract stored as the payment's `ValidationHash` (see
 * `generatePurchaseSecret` in `ticketPaymentContract.ts`). That secret lives
 * only on the original buyer's device, so a resale has to move it too — and it
 * must not be readable by the server that relays it.
 *
 * ## Scheme
 *
 * NaCl `box`: X25519 key agreement over Curve25519, then XSalsa20-Poly1305
 * authenticated encryption. The seller generates a fresh ephemeral keypair per
 * envelope, derives a shared secret against the buyer's published X25519
 * public key, and seals the ticket secret under it. Only the holder of the
 * buyer's X25519 secret key can open the result.
 *
 * A fresh ephemeral keypair per envelope means a seller's long-term key is
 * never the thing protecting a past handover.
 *
 * ## Key separation
 *
 * The buyer's encryption keypair is generated independently of their Stellar
 * signing keypair. Reusing an Ed25519 signing key for X25519 key agreement is
 * possible via birational mapping, but sharing key material across two
 * primitives is a well-known footgun, so the app keeps them separate and
 * stores the encryption secret in the device keychain via SecureStore.
 *
 * ## Randomness
 *
 * Ephemeral keys and nonces come from tweetnacl's own `randomBytes`. That is
 * the same generator the app already trusts for ticket secrets: `Keypair.random()`
 * in `ticketPaymentContract.ts` is `@stellar/stellar-base` calling into this
 * very tweetnacl instance. Do not install a custom PRNG via `nacl.setPRNG`
 * here — because the SDK shares this instance, a shim that sources entropy
 * from `Keypair.random()` recurses into itself and never terminates.
 */

import nacl from 'tweetnacl';
import * as SecureStore from 'expo-secure-store';

/** SecureStore key holding the device's base64 X25519 secret key. */
const X25519_SECRET_STORE_KEY = 'agora.resale.x25519.secret';

/** SecureStore key prefix for a ticket's check-in secret, by payment id. */
const TICKET_SECRET_STORE_PREFIX = 'agora.ticket.secret.';

/** Raw byte lengths fixed by the NaCl box construction. */
export const X25519_KEY_BYTES = 32;
export const BOX_NONCE_BYTES = 24;

// ── Base64 helpers ──────────────────────────────────────────────────────────

export function toBase64(bytes: Uint8Array): string {
  return Buffer.from(bytes).toString('base64');
}

export function fromBase64(value: string): Uint8Array {
  return new Uint8Array(Buffer.from(value, 'base64'));
}

/** Decodes a base64 field and asserts the exact raw length the scheme requires. */
function decodeExact(field: string, value: string, expectedLength: number): Uint8Array {
  const bytes = fromBase64(value);
  if (bytes.length !== expectedLength) {
    throw new Error(
      `${field} must decode to ${expectedLength} bytes, got ${bytes.length}.`
    );
  }
  return bytes;
}

// ── Buyer's long-lived encryption keypair ───────────────────────────────────

export interface EncryptionKeyPair {
  /** Base64 X25519 public key — safe to publish with an offer. */
  publicKey: string;
  /** Base64 X25519 secret key — never leaves the device. */
  secretKey: string;
}

/**
 * Returns this device's X25519 keypair, generating and persisting one on first
 * use. The secret is written to SecureStore (Keychain / Keystore), never to
 * ordinary storage.
 */
export async function getOrCreateEncryptionKeyPair(): Promise<EncryptionKeyPair> {
  const stored = await SecureStore.getItemAsync(X25519_SECRET_STORE_KEY);
  if (stored) {
    const secretKey = decodeExact('stored encryption key', stored, X25519_KEY_BYTES);
    const pair = nacl.box.keyPair.fromSecretKey(secretKey);
    return { publicKey: toBase64(pair.publicKey), secretKey: stored };
  }

  const pair = nacl.box.keyPair();
  const secretKey = toBase64(pair.secretKey);
  await SecureStore.setItemAsync(X25519_SECRET_STORE_KEY, secretKey);

  return { publicKey: toBase64(pair.publicKey), secretKey };
}

/**
 * Discards the device's encryption keypair. Any envelope still sealed to the
 * old key becomes permanently unreadable, so this is only for sign-out.
 */
export async function clearEncryptionKeyPair(): Promise<void> {
  await SecureStore.deleteItemAsync(X25519_SECRET_STORE_KEY);
}

// ── Ticket secret storage ───────────────────────────────────────────────────

/**
 * Reads the locally-held check-in secret for a ticket. Returns `null` when
 * this device never held it — a seller in that position cannot complete a
 * resale, because there is nothing to hand over.
 */
export async function getTicketSecret(paymentId: string): Promise<Uint8Array | null> {
  const stored = await SecureStore.getItemAsync(
    `${TICKET_SECRET_STORE_PREFIX}${paymentId}`
  );
  return stored ? fromBase64(stored) : null;
}

/** Persists a ticket's check-in secret for later check-in or resale. */
export async function storeTicketSecret(
  paymentId: string,
  secret: Uint8Array
): Promise<void> {
  await SecureStore.setItemAsync(
    `${TICKET_SECRET_STORE_PREFIX}${paymentId}`,
    toBase64(secret)
  );
}

/**
 * Drops a ticket's secret from this device. Called after a completed sale:
 * the seller no longer owns the ticket, and keeping the secret around only
 * widens the window in which it could leak.
 *
 * This is hygiene, not enforcement — a determined seller could have copied it
 * beforehand. Double-spend is prevented at the gate, where check-in is
 * single-use per ticket.
 */
export async function clearTicketSecret(paymentId: string): Promise<void> {
  await SecureStore.deleteItemAsync(`${TICKET_SECRET_STORE_PREFIX}${paymentId}`);
}

// ── Sealing / opening ───────────────────────────────────────────────────────

/** The wire form of a sealed ticket secret. Every field is base64. */
export interface KeyEnvelope {
  /** Ephemeral X25519 public key of the sender, needed to derive the shared secret. */
  ephemeralPublicKey: string;
  /** 24-byte XSalsa20 nonce. */
  nonce: string;
  /** NaCl box ciphertext (secret + 16-byte Poly1305 tag). */
  ciphertext: string;
}

/**
 * Seals `secret` so that only the holder of the X25519 secret key matching
 * `buyerPublicKeyB64` can read it.
 *
 * Called on the seller's device after the on-chain purchase settles. The
 * returned envelope is safe to hand to the server, which stores it verbatim
 * and cannot open it.
 */
export function sealTicketSecret(
  secret: Uint8Array,
  buyerPublicKeyB64: string
): KeyEnvelope {
  const buyerPublicKey = decodeExact(
    'buyer_public_key',
    buyerPublicKeyB64,
    X25519_KEY_BYTES
  );

  // Fresh per envelope: the sending key is discarded as soon as this returns.
  const ephemeral = nacl.box.keyPair();
  const nonce = nacl.randomBytes(BOX_NONCE_BYTES);
  const ciphertext = nacl.box(secret, nonce, buyerPublicKey, ephemeral.secretKey);

  if (!ciphertext) {
    throw new Error('Failed to encrypt the ticket key for this buyer.');
  }

  return {
    ephemeralPublicKey: toBase64(ephemeral.publicKey),
    nonce: toBase64(nonce),
    ciphertext: toBase64(ciphertext),
  };
}

/**
 * Opens an envelope sealed to this device's encryption key.
 *
 * Returns the raw ticket secret. A `null` from `nacl.box.open` means the
 * Poly1305 tag did not verify — the envelope was tampered with, addressed to a
 * different key, or the buyer's key has been rotated since the offer — so this
 * throws rather than returning something the caller might treat as a secret.
 */
export function openTicketSecret(
  envelope: KeyEnvelope,
  recipientSecretKeyB64: string
): Uint8Array {
  const ephemeralPublicKey = decodeExact(
    'ephemeral_public_key',
    envelope.ephemeralPublicKey,
    X25519_KEY_BYTES
  );
  const nonce = decodeExact('nonce', envelope.nonce, BOX_NONCE_BYTES);
  const recipientSecretKey = decodeExact(
    'recipient secret key',
    recipientSecretKeyB64,
    X25519_KEY_BYTES
  );
  const ciphertext = fromBase64(envelope.ciphertext);

  const opened = nacl.box.open(
    ciphertext,
    nonce,
    ephemeralPublicKey,
    recipientSecretKey
  );

  if (!opened) {
    throw new Error(
      'Could not decrypt the ticket key. It may have been sealed to a different device, or tampered with in transit.'
    );
  }

  return opened;
}
