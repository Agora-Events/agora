import { xdr, scValToNative, hash as sha256 } from '@stellar/stellar-sdk';
import {
  buildPurchaseOptionsScVal,
  describeContractError,
  formatUsdc,
  generatePaymentId,
  generatePurchaseSecret,
  INSUFFICIENT_ALLOWANCE_ERROR_CODE,
  stroopsToUsdc,
  TICKET_PAYMENT_ERROR_MESSAGES,
  usdcToStroops,
} from '../ticketPaymentContract';

/**
 * Issue #1005 — ticket checkout / Soroban transaction builder.
 *
 * These tests cover the pure, deterministic pieces of the contract-encoding
 * layer (amount conversion, PurchaseOptions XDR shape, error mapping, id/secret
 * generation). The network-dependent orchestration (`purchaseTickets`,
 * `submitUsdcApproval`, `submitProcessPayment`) is exercised indirectly
 * through `useTicketCheckout`'s tests, which mock this module entirely.
 */

describe('usdcToStroops / stroopsToUsdc', () => {
  it('converts a decimal USDC amount to base-unit stroops (7 decimals)', () => {
    expect(usdcToStroops(25)).toBe(250_000_000n);
    expect(usdcToStroops(0.01)).toBe(100_000n);
    expect(usdcToStroops(0)).toBe(0n);
  });

  it('round-trips through stroopsToUsdc', () => {
    expect(stroopsToUsdc(usdcToStroops(12.34))).toBeCloseTo(12.34, 6);
  });

  it('rejects negative or non-finite amounts', () => {
    expect(() => usdcToStroops(-1)).toThrow();
    expect(() => usdcToStroops(NaN)).toThrow();
    expect(() => usdcToStroops(Infinity)).toThrow();
  });
});

describe('formatUsdc', () => {
  it('always shows exactly two decimal places', () => {
    expect(formatUsdc(12.5)).toBe('12.50');
    expect(formatUsdc(0)).toBe('0.00');
    expect(formatUsdc(9.999)).toBe('10.00');
  });
});

describe('buildPurchaseOptionsScVal', () => {
  function decodeMapKeys(scVal: xdr.ScVal): string[] {
    const map = scVal.map();
    if (!map) throw new Error('Expected an ScVal map');
    return map.map((entry) => entry.key().sym().toString());
  }

  it('encodes all three fields, alphabetically ordered, as void when omitted', () => {
    const scVal = buildPurchaseOptionsScVal({});
    expect(decodeMapKeys(scVal)).toEqual(['code_preimage', 'discount_code', 'referrer']);

    const map = scVal.map()!;
    for (const entry of map) {
      expect(entry.val().switch().name).toBe('scvVoid');
    }
  });

  it('encodes a provided discount code as an ScString', () => {
    const scVal = buildPurchaseOptionsScVal({ discountCode: 'SAVE10' });
    const map = scVal.map()!;
    const discountEntry = map.find((entry) => entry.key().sym().toString() === 'discount_code')!;
    expect(scValToNative(discountEntry.val())).toBe('SAVE10');
  });

  it('encodes a provided referrer address as an ScAddress', () => {
    const referrer = 'GA7QYNF7SOWQ3GLR2BGMZEHXAVIRZA4KVWLTJJFC7MGXUA74P7UJVSGZ';
    const scVal = buildPurchaseOptionsScVal({ referrerAddress: referrer });
    const map = scVal.map()!;
    const referrerEntry = map.find((entry) => entry.key().sym().toString() === 'referrer')!;
    expect(referrerEntry.val().switch().name).toBe('scvAddress');
  });
});

describe('contract error table', () => {
  it('maps the InsufficientAllowance code (12) to a readable, non-generic message', () => {
    const message = describeContractError(INSUFFICIENT_ALLOWANCE_ERROR_CODE);
    expect(message).toBe(TICKET_PAYMENT_ERROR_MESSAGES[12]);
    expect(message.toLowerCase()).toContain('approval');
  });

  it('falls back to a generic message for unmapped codes', () => {
    expect(describeContractError(9999)).toMatch(/error code 9999/);
  });

  it('falls back to a generic message when no code was decoded', () => {
    expect(describeContractError(null)).toMatch(/unknown reason/);
  });

  it('has a distinct human message for every documented TicketPaymentError variant', () => {
    // Mirrors contract/contracts/ticket_payment/src/error.rs — every code should
    // read as a sentence, not a code fragment, so buyers never see "Error 42".
    const codes = Object.keys(TICKET_PAYMENT_ERROR_MESSAGES).map(Number);
    expect(codes.length).toBeGreaterThanOrEqual(50);
    for (const code of codes) {
      expect(TICKET_PAYMENT_ERROR_MESSAGES[code].length).toBeGreaterThan(10);
    }
  });
});

describe('generatePaymentId', () => {
  it('is stable in shape and unique across calls', () => {
    const buyer = 'GA7QYNF7SOWQ3GLR2BGMZEHXAVIRZA4KVWLTJJFC7MGXUA74P7UJVSGZ';
    const first = generatePaymentId('evt-1', buyer);
    const second = generatePaymentId('evt-1', buyer);

    expect(first).toMatch(/^pay-evt-1-/);
    expect(first.length).toBeLessThanOrEqual(64);
    expect(first).not.toBe(second);
  });
});

describe('generatePurchaseSecret', () => {
  it('produces a 32-byte secret whose SHA-256 digest is the returned hash', () => {
    const { secretBytes, hash } = generatePurchaseSecret();
    expect(secretBytes.length).toBe(32);
    expect(hash.length).toBe(32);
    expect(Buffer.from(hash).equals(sha256(Buffer.from(secretBytes)))).toBe(true);
  });

  it('never repeats between calls', () => {
    const a = generatePurchaseSecret();
    const b = generatePurchaseSecret();
    expect(Buffer.from(a.secretBytes).equals(Buffer.from(b.secretBytes))).toBe(false);
  });
});
