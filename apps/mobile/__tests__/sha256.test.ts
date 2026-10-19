import { sha256Hex } from '@/utils/sha256';
import { solvePow } from '@/services/waitingRoom';

/**
 * SHA-256 is verified against the standard NIST test vectors so a regression
 * in the pure-TS implementation is caught immediately (Issue #1187).
 */
describe('sha256Hex', () => {
  it('matches the empty-string test vector', () => {
    expect(sha256Hex('')).toBe('e3b0c44298fc1c149afbf4c8996fb92427ae41e4649b934ca495991b7852b855');
  });

  it('matches the "abc" test vector', () => {
    expect(sha256Hex('abc')).toBe('ba7816bf8f01cfea414140de5dae2223b00361a396177a9cb410ff61f20015ad');
  });

  it('matches the "The quick brown fox..." test vector', () => {
    expect(sha256Hex('The quick brown fox jumps over the lazy dog')).toBe(
      'd7a8fbb307d7809469ca9abcb0082e4f8d5651e46d3cdb762d02d0bf37c9e592'
    );
  });

  it('handles messages that exercise the padding boundary (55, 56, 64 bytes)', () => {
    // 55 bytes → single block
    expect(sha256Hex('a'.repeat(55))).toBe('9f4390f8d30c2dd92ec9f095b65e2b9ae9b0a925a5258e241c9f1e910f734318');
    // 56 bytes → padding spills into a second block
    expect(sha256Hex('a'.repeat(56))).toBe('b35439a4ac6f0948b6d6f9e3c6af0f5f590ce20f1bde7090ef7970686ec6738a');
    // 64 bytes → exactly one full block
    expect(sha256Hex('a'.repeat(64))).toBe('ffe054fe7ae0cb6dc65c3af9b61d5209f439851db43d0ba5997337df154668eb');
  });

  it('handles non-ASCII UTF-8 input', () => {
    // "¡Hola, mundo!" — multi-byte characters exercise UTF-8 encoding.
    expect(sha256Hex('¡Hola, mundo!')).toBe('89cbd75bc4d8136ac5a1b54619f73933a8cd55db7b686b470b3a5db083ccd527');
  });
});

describe('solvePow', () => {
  it('finds a nonce whose hash starts with the required number of zeros', () => {
    const challenge = 'deadbeefdeadbeefdeadbeefdeadbeef';
    const nonce = solvePow(challenge, 2, 100_000);
    const digest = sha256Hex(`${challenge}${nonce}`);
    expect(digest.startsWith('00')).toBe(true);
  });

  it('throws when the search space is exhausted', () => {
    expect(() => solvePow('challenge', 30, 10)).toThrow(/proof-of-work/);
  });
});
