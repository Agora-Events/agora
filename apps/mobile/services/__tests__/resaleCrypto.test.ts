import nacl from 'tweetnacl';

import {
  BOX_NONCE_BYTES,
  X25519_KEY_BYTES,
  fromBase64,
  openTicketSecret,
  sealTicketSecret,
  toBase64,
} from '../resaleCrypto';

/**
 * Issue #1184 — the resale flow hands a ticket's check-in secret from seller to
 * buyer through a server that must not be able to read it. These tests cover
 * the sealing/opening round trip and the failure modes that matter: a wrong
 * recipient, and tampering in transit.
 *
 * `getOrCreateEncryptionKeyPair` and the ticket-secret helpers are not covered
 * here — they are thin SecureStore wrappers, and SecureStore is a native
 * module with no meaningful behavior to assert under jest.
 */

/** Stands in for a buyer's device: a keypair whose secret never leaves it. */
function makeRecipient() {
  const pair = nacl.box.keyPair();
  return {
    publicKey: toBase64(pair.publicKey),
    secretKey: toBase64(pair.secretKey),
  };
}

const TICKET_SECRET = new Uint8Array(32).fill(7);

describe('sealTicketSecret / openTicketSecret', () => {
  it('round-trips a ticket secret to the intended recipient', () => {
    const buyer = makeRecipient();

    const envelope = sealTicketSecret(TICKET_SECRET, buyer.publicKey);
    const opened = openTicketSecret(envelope, buyer.secretKey);

    expect(Array.from(opened)).toEqual(Array.from(TICKET_SECRET));
  });

  it('produces envelope fields of the sizes the server validates', () => {
    const buyer = makeRecipient();
    const envelope = sealTicketSecret(TICKET_SECRET, buyer.publicKey);

    expect(fromBase64(envelope.ephemeralPublicKey)).toHaveLength(X25519_KEY_BYTES);
    expect(fromBase64(envelope.nonce)).toHaveLength(BOX_NONCE_BYTES);
    // Ciphertext is the plaintext plus a 16-byte Poly1305 tag.
    expect(fromBase64(envelope.ciphertext)).toHaveLength(TICKET_SECRET.length + 16);
  });

  it('never emits the plaintext secret in the envelope', () => {
    const buyer = makeRecipient();
    const envelope = sealTicketSecret(TICKET_SECRET, buyer.publicKey);

    const plaintextB64 = toBase64(TICKET_SECRET);
    expect(envelope.ciphertext).not.toContain(plaintextB64);
    expect(fromBase64(envelope.ciphertext)).not.toEqual(TICKET_SECRET);
  });

  it('uses a fresh ephemeral key and nonce for every envelope', () => {
    const buyer = makeRecipient();

    const first = sealTicketSecret(TICKET_SECRET, buyer.publicKey);
    const second = sealTicketSecret(TICKET_SECRET, buyer.publicKey);

    expect(first.ephemeralPublicKey).not.toEqual(second.ephemeralPublicKey);
    expect(first.nonce).not.toEqual(second.nonce);
    // Same plaintext, different ciphertext — no deterministic leak.
    expect(first.ciphertext).not.toEqual(second.ciphertext);
  });

  it('cannot be opened by a different recipient', () => {
    const buyer = makeRecipient();
    const eavesdropper = makeRecipient();

    const envelope = sealTicketSecret(TICKET_SECRET, buyer.publicKey);

    expect(() => openTicketSecret(envelope, eavesdropper.secretKey)).toThrow(
      /Could not decrypt/
    );
  });

  it('rejects a tampered ciphertext rather than returning garbage', () => {
    const buyer = makeRecipient();
    const envelope = sealTicketSecret(TICKET_SECRET, buyer.publicKey);

    const bytes = fromBase64(envelope.ciphertext);
    bytes[0] ^= 0xff;
    const tampered = { ...envelope, ciphertext: toBase64(bytes) };

    // The Poly1305 tag is what makes this a hard failure instead of a silent
    // corruption the buyer would carry to the gate.
    expect(() => openTicketSecret(tampered, buyer.secretKey)).toThrow(/Could not decrypt/);
  });

  it('rejects a tampered nonce', () => {
    const buyer = makeRecipient();
    const envelope = sealTicketSecret(TICKET_SECRET, buyer.publicKey);

    const nonce = fromBase64(envelope.nonce);
    nonce[0] ^= 0xff;
    const tampered = { ...envelope, nonce: toBase64(nonce) };

    expect(() => openTicketSecret(tampered, buyer.secretKey)).toThrow(/Could not decrypt/);
  });

  it('rejects a buyer public key of the wrong length', () => {
    const shortKey = toBase64(new Uint8Array(31));

    expect(() => sealTicketSecret(TICKET_SECRET, shortKey)).toThrow(
      /must decode to 32 bytes/
    );
  });

  it('rejects a malformed envelope on open', () => {
    const buyer = makeRecipient();
    const envelope = sealTicketSecret(TICKET_SECRET, buyer.publicKey);

    expect(() =>
      openTicketSecret({ ...envelope, nonce: toBase64(new Uint8Array(8)) }, buyer.secretKey)
    ).toThrow(/must decode to 24 bytes/);
  });
});
