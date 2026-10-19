//! # SHA-256 Proof-of-Work (Issue #1187)
//!
//! Hashcash-style proof-of-work used to gate entrance into the virtual
//! waiting room. Clients are issued a random challenge and must find a `nonce`
//! such that `SHA-256(challenge || nonce)` starts with `difficulty` hex `0`
//! characters (i.e. `difficulty * 4` leading zero bits). This makes automated
//! queue spam expensive while remaining trivially fast for real users on any
//! device (a difficulty of 4 requires ~65k hash attempts on average).
//!
//! The challenge itself is a random 128-bit value so answers cannot be
//! pre-computed, and handlers additionally make challenges single-use and
//! time-limited so a solved nonce cannot be replayed.

use rand::RngCore;
use sha2::{Digest, Sha256};

/// Default number of leading hex zeros required (16 bits of work).
pub const DEFAULT_DIFFICULTY: u32 = 4;

/// Default lifetime of an issued challenge in seconds.
pub const DEFAULT_CHALLENGE_TTL_SECONDS: u64 = 300;

/// Generate a fresh random challenge string (32 hex chars / 16 random bytes).
pub fn generate_challenge() -> String {
    let mut bytes = [0u8; 16];
    rand::thread_rng().fill_bytes(&mut bytes);
    hex::encode(bytes)
}

/// Verify that `nonce` satisfies the challenge at the given difficulty.
///
/// Returns `true` when `SHA-256(challenge || nonce)` begins with `difficulty`
/// hex zeros. A difficulty of 0 always passes (no work required).
pub fn verify_pow(challenge: &str, nonce: &str, difficulty: u32) -> bool {
    if difficulty == 0 {
        return true;
    }
    let mut hasher = Sha256::new();
    hasher.update(challenge.as_bytes());
    hasher.update(nonce.as_bytes());
    let digest = hasher.finalize();
    hex::encode(digest)
        .chars()
        .take(difficulty as usize)
        .all(|c| c == '0')
}

/// Brute-force a nonce that satisfies the challenge (used by clients and tests).
///
/// Iterates nonces from `0` upwards; statistically expects `2^(4*difficulty)`
/// attempts. Returns `None` only if the search space is exhausted, which is
/// effectively impossible at the difficulties used in practice.
pub fn solve_pow(challenge: &str, difficulty: u32) -> Option<String> {
    for nonce in 0..u64::MAX {
        let candidate = nonce.to_string();
        if verify_pow(challenge, &candidate, difficulty) {
            return Some(candidate);
        }
    }
    None
}

// ---------------------------------------------------------------------------
// Tests
// ---------------------------------------------------------------------------

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_generate_challenge_is_random_hex() {
        let a = generate_challenge();
        let b = generate_challenge();
        assert_eq!(a.len(), 32);
        assert!(a.chars().all(|c| c.is_ascii_hexdigit()));
        assert_ne!(a, b);
    }

    #[test]
    fn test_verify_pow_accepts_solved_nonce() {
        let challenge = generate_challenge();
        let nonce = solve_pow(&challenge, 4).expect("should find a nonce at difficulty 4");
        assert!(verify_pow(&challenge, &nonce, 4));
    }

    #[test]
    fn test_verify_pow_rejects_bad_nonce() {
        let challenge = generate_challenge();
        // Nonce "0" will virtually never satisfy difficulty 4 — assert the
        // obvious failure path and also that a difficulty bump invalidates a
        // previously-valid nonce.
        assert!(!verify_pow(&challenge, "0", 4));
        let nonce = solve_pow(&challenge, 3).expect("should find a nonce at difficulty 3");
        assert!(verify_pow(&challenge, &nonce, 3));
        // The same nonce fails at a higher difficulty (unless the hash is
        // spectacularly lucky — astronomically unlikely at 3 -> 4).
        assert!(!verify_pow(&challenge, &nonce, 6));
    }

    #[test]
    fn test_verify_pow_zero_difficulty_always_passes() {
        assert!(verify_pow("challenge", "any-nonce", 0));
        assert!(verify_pow("", "", 0));
    }

    #[test]
    fn test_verify_pow_deterministic() {
        let challenge = generate_challenge();
        let nonce = solve_pow(&challenge, 4).unwrap();
        assert_eq!(
            verify_pow(&challenge, &nonce, 4),
            verify_pow(&challenge, &nonce, 4)
        );
        // A different challenge must not accept the same nonce.
        let other = generate_challenge();
        assert_ne!(challenge, other);
        assert!(!verify_pow(&other, &nonce, 4));
    }

    #[test]
    fn test_solve_pow_expected_difficulty() {
        // Difficulty 2 (8 bits) — quick sanity check that solved nonces carry
        // exactly the required number of leading zeros.
        let challenge = generate_challenge();
        let nonce = solve_pow(&challenge, 2).unwrap();
        assert!(verify_pow(&challenge, &nonce, 2));
    }
}
