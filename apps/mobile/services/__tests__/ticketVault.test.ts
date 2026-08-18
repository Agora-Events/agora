/**
 * ticketVault.test.ts — Issue #1179: Offline Ticket Vault
 *
 * Tests for ticketVault.ts vault operations.
 * expo-secure-store is mocked: this module tests the vault logic only, not the
 * platform-specific Keychain / Keystore integration.
 */

import {
  storeTicketSecret,
  getTicketSecret,
  getTicketPublicKey,
  clearTicketSecret,
  isVaultAvailable,
} from '../../services/ticketVault';
import { deriveTicketKeyPair, toBase64Url, fromBase64Url } from '../../lib/crypto';

// ── Mock expo-secure-store ────────────────────────────────────────────────────

const mockStore = new Map<string, string>();

jest.mock('expo-secure-store', () => ({
  setItemAsync: jest.fn(async (key: string, value: string) => {
    mockStore.set(key, value);
  }),
  getItemAsync: jest.fn(async (key: string) => {
    return mockStore.get(key) ?? null;
  }),
  deleteItemAsync: jest.fn(async (key: string) => {
    mockStore.delete(key);
  }),
  isAvailableAsync: jest.fn(async () => true),
}));

beforeEach(() => {
  mockStore.clear();
  jest.clearAllMocks();
});

// ── Fixtures ──────────────────────────────────────────────────────────────────

function makeSecret(fill: number): Uint8Array {
  return new Uint8Array(32).fill(fill);
}

const PAYMENT_ID = 'pay-event-123-GABCDE-abc123-def456';

// ── isVaultAvailable ──────────────────────────────────────────────────────────

describe('isVaultAvailable', () => {
  it('returns true when SecureStore is available', async () => {
    expect(await isVaultAvailable()).toBe(true);
  });
});

// ── storeTicketSecret ─────────────────────────────────────────────────────────

describe('storeTicketSecret', () => {
  it('writes to SecureStore without throwing', async () => {
    await expect(storeTicketSecret(PAYMENT_ID, makeSecret(0x11))).resolves.toBeUndefined();
  });

  it('stores the secret and public key separately', async () => {
    const secret = makeSecret(0x22);
    await storeTicketSecret(PAYMENT_ID, secret);

    // Should have two entries in the mock store
    const secretKey = `agora.vault.secret.${PAYMENT_ID}`;
    const pubkeyKey = `agora.vault.pubkey.${PAYMENT_ID}`;
    expect(mockStore.has(secretKey)).toBe(true);
    expect(mockStore.has(pubkeyKey)).toBe(true);
  });

  it('throws for a secret shorter than 32 bytes', async () => {
    await expect(
      storeTicketSecret(PAYMENT_ID, new Uint8Array(16))
    ).rejects.toThrow(/32 bytes/);
  });

  it('throws for a secret longer than 32 bytes', async () => {
    await expect(
      storeTicketSecret(PAYMENT_ID, new Uint8Array(64))
    ).rejects.toThrow(/32 bytes/);
  });

  it('never writes plaintext — stored value differs from raw secret', async () => {
    const secret = makeSecret(0x33);
    await storeTicketSecret(PAYMENT_ID, secret);
    const rawSecret = Array.from(secret).join(',');
    // Verify none of the stored values are the raw byte string
    for (const [, value] of mockStore) {
      expect(value).not.toBe(rawSecret);
    }
  });
});

// ── getTicketSecret ───────────────────────────────────────────────────────────

describe('getTicketSecret', () => {
  it('returns the original secret after storeTicketSecret', async () => {
    const secret = makeSecret(0x44);
    await storeTicketSecret(PAYMENT_ID, secret);
    const retrieved = await getTicketSecret(PAYMENT_ID);
    expect(retrieved).not.toBeNull();
    expect(retrieved).toEqual(secret);
  });

  it('returns null when no secret is stored', async () => {
    const result = await getTicketSecret('nonexistent-payment-id');
    expect(result).toBeNull();
  });

  it('retrieved secret reproduces the same keypair as original', async () => {
    const secret = makeSecret(0x55);
    await storeTicketSecret(PAYMENT_ID, secret);
    const retrieved = await getTicketSecret(PAYMENT_ID);
    expect(retrieved).not.toBeNull();

    const originalKp = deriveTicketKeyPair(secret);
    const retrievedKp = deriveTicketKeyPair(retrieved!);
    expect(originalKp.publicKey).toEqual(retrievedKp.publicKey);
  });
});

// ── getTicketPublicKey ────────────────────────────────────────────────────────

describe('getTicketPublicKey', () => {
  it('returns the correct 32-byte public key after storeTicketSecret', async () => {
    const secret = makeSecret(0x66);
    await storeTicketSecret(PAYMENT_ID, secret);

    const storedPubKey = await getTicketPublicKey(PAYMENT_ID);
    expect(storedPubKey).not.toBeNull();
    expect(storedPubKey!.length).toBe(32);

    const expectedPubKey = deriveTicketKeyPair(secret).publicKey;
    expect(storedPubKey).toEqual(expectedPubKey);
  });

  it('returns null when no public key is stored', async () => {
    const result = await getTicketPublicKey('nonexistent');
    expect(result).toBeNull();
  });
});

// ── clearTicketSecret ─────────────────────────────────────────────────────────

describe('clearTicketSecret', () => {
  it('removes both secret and public key from the store', async () => {
    const secret = makeSecret(0x77);
    await storeTicketSecret(PAYMENT_ID, secret);

    await clearTicketSecret(PAYMENT_ID);

    expect(await getTicketSecret(PAYMENT_ID)).toBeNull();
    expect(await getTicketPublicKey(PAYMENT_ID)).toBeNull();
  });

  it('does not throw when called on a non-existent payment ID', async () => {
    await expect(clearTicketSecret('does-not-exist')).resolves.toBeUndefined();
  });
});

// ── Key separation ────────────────────────────────────────────────────────────

describe('key separation', () => {
  it('different paymentIds store different secrets', async () => {
    const secret1 = makeSecret(0x11);
    const secret2 = makeSecret(0x22);
    await storeTicketSecret('payment-A', secret1);
    await storeTicketSecret('payment-B', secret2);

    const r1 = await getTicketSecret('payment-A');
    const r2 = await getTicketSecret('payment-B');
    expect(r1).toEqual(secret1);
    expect(r2).toEqual(secret2);
    expect(r1).not.toEqual(r2);
  });

  it('clearing one paymentId does not affect another', async () => {
    await storeTicketSecret('payment-X', makeSecret(0x0A));
    await storeTicketSecret('payment-Y', makeSecret(0x0B));
    await clearTicketSecret('payment-X');

    expect(await getTicketSecret('payment-X')).toBeNull();
    expect(await getTicketSecret('payment-Y')).not.toBeNull();
  });
});
