/**
 * ristretto255.test.ts — Issue #1186.
 *
 * Group-arithmetic tests. The encodings here are fixed by RFC 9496 and by the
 * Rust verifier; the values asserted below were produced by
 * `curve25519-dalek`, so a failure means this implementation has drifted from
 * the one that will actually check the proofs.
 */

import {
  BASEPOINT,
  bigIntToBytesLE,
  bytesToBigIntLE,
  D,
  feAbs,
  feInv,
  feIsNegative,
  feMul,
  feSqrt,
  IDENTITY,
  INVSQRT_A_MINUS_D,
  L,
  P,
  pointAdd,
  pointCompress,
  pointDecompress,
  pointDouble,
  pointEquals,
  pointIsIdentity,
  pointMul,
  pointMulAdd,
  pointNeg,
  pointSub,
  scalarFromBytes,
  scalarFromBytesWide,
  scalarToBytes,
  SQRT_AD_MINUS_ONE,
  SQRT_M1,
} from '../ristretto255';

const ZERO = BigInt(0);
const ONE = BigInt(1);
const TWO = BigInt(2);

const toHex = (b: Uint8Array): string =>
  Array.from(b)
    .map((x) => x.toString(16).padStart(2, '0'))
    .join('');

describe('field constants', () => {
  it('uses the curve25519 prime', () => {
    expect(P).toBe(BigInt(2) ** BigInt(255) - BigInt(19));
  });

  it('derives d = -121665/121666', () => {
    expect(feMul(D, BigInt(121666))).toBe(
      ((BigInt(-121665) % P) + P) % P
    );
  });

  it('derives a square root of -1', () => {
    expect(feMul(SQRT_M1, SQRT_M1)).toBe(P - ONE);
  });

  /**
   * The two ad-related constants use *opposite* roots in the reference
   * implementation — their product is −1, not 1. Deriving one as the inverse
   * of the other is a bug that round-trip tests cannot see, because encode and
   * decode stay self-consistent; only the Elligator output diverges. This test
   * exists specifically to pin that down.
   */
  it('pairs SQRT_AD_MINUS_ONE and INVSQRT_A_MINUS_D with opposite signs', () => {
    expect(feMul(SQRT_AD_MINUS_ONE, INVSQRT_A_MINUS_D)).toBe(P - ONE);
    expect(feIsNegative(SQRT_AD_MINUS_ONE)).toBe(true);
    expect(feIsNegative(INVSQRT_A_MINUS_D)).toBe(false);
  });

  it('matches the reference values for both ad constants', () => {
    expect(SQRT_AD_MINUS_ONE.toString()).toBe(
      '25063068953384623474111414158702152701244531502492656460079210482610430750235'
    );
    expect(INVSQRT_A_MINUS_D.toString()).toBe(
      '54469307008909316920995813868745141605393597292927456921205312896311721017578'
    );
  });

  it('uses the prime group order', () => {
    expect(L).toBe(
      BigInt(2) ** BigInt(252) + BigInt('27742317777372353535851937790883648493')
    );
  });
});

describe('field arithmetic', () => {
  it('inverts', () => {
    for (const v of [ONE, TWO, BigInt(12345), D, SQRT_M1]) {
      expect(feMul(v, feInv(v))).toBe(ONE);
    }
  });

  it('computes square roots when they exist', () => {
    const root = feSqrt(feMul(BigInt(7), BigInt(7)));
    expect(root).not.toBeNull();
    expect(feAbs(root as bigint)).toBe(BigInt(7));
  });

  it('returns null for non-squares', () => {
    // Exactly half the field is non-square; at least one of these must be.
    const nonSquares = [BigInt(2), BigInt(3), BigInt(5), BigInt(7), BigInt(11)]
      .map(feSqrt)
      .filter((r) => r === null);
    expect(nonSquares.length).toBeGreaterThan(0);
  });

  it('treats odd canonical representatives as negative', () => {
    expect(feIsNegative(ONE)).toBe(true);
    expect(feIsNegative(TWO)).toBe(false);
    expect(feIsNegative(ZERO)).toBe(false);
  });
});

describe('byte conversion', () => {
  it('round-trips little-endian', () => {
    const value = BigInt('0x0123456789abcdef');
    expect(bytesToBigIntLE(bigIntToBytesLE(value, 32))).toBe(value);
  });

  it('is little-endian, not big-endian', () => {
    expect(toHex(bigIntToBytesLE(ONE, 4))).toBe('01000000');
  });
});

describe('the basepoint', () => {
  /** The canonical ristretto255 basepoint encoding from RFC 9496. */
  it('compresses to the published encoding', () => {
    expect(toHex(pointCompress(BASEPOINT))).toBe(
      'e2f2ae0a6abc4e71a884a961c500515f58e30b6aa582dd8db6a65945e08d2d76'
    );
  });

  it('decompresses back to itself', () => {
    const decoded = pointDecompress(pointCompress(BASEPOINT));
    expect(decoded).not.toBeNull();
    expect(pointEquals(decoded as never, BASEPOINT)).toBe(true);
  });
});

describe('group law', () => {
  it('treats the identity as neutral', () => {
    expect(pointEquals(pointAdd(BASEPOINT, IDENTITY), BASEPOINT)).toBe(true);
    expect(pointIsIdentity(IDENTITY)).toBe(true);
    expect(pointIsIdentity(BASEPOINT)).toBe(false);
  });

  it('cancels a point against its negation', () => {
    expect(pointIsIdentity(pointAdd(BASEPOINT, pointNeg(BASEPOINT)))).toBe(true);
    expect(pointIsIdentity(pointSub(BASEPOINT, BASEPOINT))).toBe(true);
  });

  it('agrees between doubling and addition', () => {
    expect(pointEquals(pointDouble(BASEPOINT), pointAdd(BASEPOINT, BASEPOINT))).toBe(true);
  });

  it('agrees between scalar multiplication and repeated addition', () => {
    let sum = IDENTITY;
    for (let i = 0; i < 9; i++) sum = pointAdd(sum, BASEPOINT);
    expect(pointEquals(pointMul(BASEPOINT, BigInt(9)), sum)).toBe(true);
  });

  it('is distributive over scalar addition', () => {
    const a = BigInt(1234567);
    const b = BigInt(7654321);
    expect(
      pointEquals(
        pointMul(BASEPOINT, a + b),
        pointAdd(pointMul(BASEPOINT, a), pointMul(BASEPOINT, b))
      )
    ).toBe(true);
  });

  it('sends the group order to the identity', () => {
    expect(pointIsIdentity(pointMul(BASEPOINT, L))).toBe(true);
  });

  it('computes combined multiplications correctly', () => {
    const a = BigInt(99991);
    const b = BigInt(11117);
    const combined = pointMulAdd(BASEPOINT, a, pointDouble(BASEPOINT), b);
    const expected = pointAdd(pointMul(BASEPOINT, a), pointMul(pointDouble(BASEPOINT), b));
    expect(pointEquals(combined, expected)).toBe(true);
  });

  it('handles multiples 1 through 16, spanning the 4-bit window boundary', () => {
    let acc = IDENTITY;
    for (let i = 1; i <= 16; i++) {
      acc = pointAdd(acc, BASEPOINT);
      expect(pointEquals(pointMul(BASEPOINT, BigInt(i)), acc)).toBe(true);
    }
  });
});

describe('ristretto encoding', () => {
  it('round-trips a range of points', () => {
    for (let i = 1; i <= 12; i++) {
      const point = pointMul(BASEPOINT, BigInt(i * 7919));
      const decoded = pointDecompress(pointCompress(point));
      expect(decoded).not.toBeNull();
      expect(pointEquals(decoded as never, point)).toBe(true);
    }
  });

  it('encodes the identity as all zeros', () => {
    expect(toHex(pointCompress(IDENTITY))).toBe('00'.repeat(32));
  });

  it('gives equal encodings to equal points', () => {
    // Same group element via two different coordinate representations.
    const viaDouble = pointDouble(BASEPOINT);
    const viaAdd = pointAdd(BASEPOINT, BASEPOINT);
    expect(toHex(pointCompress(viaDouble))).toBe(toHex(pointCompress(viaAdd)));
  });

  it('rejects wrong-length input', () => {
    expect(pointDecompress(new Uint8Array(31))).toBeNull();
    expect(pointDecompress(new Uint8Array(33))).toBeNull();
  });

  it('rejects non-canonical field encodings', () => {
    // p itself, and values above it, must not decode.
    expect(pointDecompress(bigIntToBytesLE(P, 32))).toBeNull();
    expect(pointDecompress(bigIntToBytesLE(P + ONE, 32))).toBeNull();
  });

  it('rejects encodings whose s is negative', () => {
    // s = 1 is odd, therefore "negative" under the sign convention.
    expect(pointDecompress(bigIntToBytesLE(ONE, 32))).toBeNull();
  });

  it('rejects bytes that do not lie on the curve', () => {
    // 0xff.. is above p and must be refused rather than silently reduced.
    expect(pointDecompress(new Uint8Array(32).fill(0xff))).toBeNull();
  });
});

describe('scalar arithmetic', () => {
  it('round-trips canonical scalars', () => {
    const s = BigInt('123456789012345678901234567890');
    expect(scalarFromBytes(scalarToBytes(s))).toBe(s);
  });

  it('rejects scalars at or above the group order', () => {
    expect(scalarFromBytes(bigIntToBytesLE(L, 32))).toBeNull();
    expect(scalarFromBytes(bigIntToBytesLE(L + ONE, 32))).toBeNull();
    expect(scalarFromBytes(bigIntToBytesLE(L - ONE, 32))).toBe(L - ONE);
  });

  it('reduces wide input below the group order', () => {
    const wide = new Uint8Array(64).fill(0xff);
    const reduced = scalarFromBytesWide(wide);
    expect(reduced).toBeLessThan(L);
    expect(reduced).toBeGreaterThanOrEqual(ZERO);
  });

  it('rejects wide input of the wrong length', () => {
    expect(() => scalarFromBytesWide(new Uint8Array(32))).toThrow();
  });
});
