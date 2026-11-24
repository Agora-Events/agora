//! # Auth Service Module
//!
//! Dedicated service for passwordless / magic-link style auth token
//! generation (Issue #1554).
//!
//! Extracted from the router handler so nonce and CSRF token generation
//! is testable and reusable outside [`crate::handlers::auth`].

use rand::RngCore;

/// Stateless auth token generation helpers.
pub struct AuthService;

impl AuthService {
    /// Generate a 32-byte random nonce encoded as hex.
    pub fn generate_nonce() -> String {
        let mut bytes = [0u8; 32];
        rand::thread_rng().fill_bytes(&mut bytes);
        hex::encode(bytes)
    }

    /// Generate a 32-byte random CSRF token encoded as hex.
    pub fn generate_csrf_token() -> String {
        let mut bytes = [0u8; 32];
        rand::thread_rng().fill_bytes(&mut bytes);
        hex::encode(bytes)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_generate_nonce_is_unique_hex() {
        let a = AuthService::generate_nonce();
        let b = AuthService::generate_nonce();
        assert_eq!(a.len(), 64);
        assert_ne!(a, b);
    }

    #[test]
    fn test_generate_csrf_token_length() {
        assert_eq!(AuthService::generate_csrf_token().len(), 64);
    }
}
