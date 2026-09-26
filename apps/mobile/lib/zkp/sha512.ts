/**
 * sha512.ts — Issue #1186: the one hash primitive the ZK prover uses.
 *
 * Isolated in its own module for two reasons:
 *
 *   1. `ristretto255.ts` stays dependency-free and therefore trivially
 *      testable in isolation.
 *   2. The protocol hashes with SHA-512 *everywhere*, including where a
 *      32-byte digest is wanted (Merkle nodes), because tweetnacl is the only
 *      hash already in the bundle and it offers SHA-512 alone. The Rust
 *      verifier truncates SHA-512 to 32 bytes to match rather than reaching
 *      for SHA-256 — see `hash32` in `server/src/utils/zkp_verifier.rs`.
 */

import nacl from 'tweetnacl';

/** SHA-512 over a byte string, returning all 64 bytes. */
export function sha512(message: Uint8Array): Uint8Array {
  return nacl.hash(message);
}

/** Cryptographically secure random bytes, from the platform CSPRNG. */
export function randomBytes(length: number): Uint8Array {
  return nacl.randomBytes(length);
}
