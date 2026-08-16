//! # Services Module
//!
//! Reusable background services that are not tied to a single HTTP handler:
//! - [`pow`] – SHA-256 proof-of-work challenge (Hashcash-style) used to gate
//!   entrance into the virtual waiting room (Issue #1187).
//! - [`queue`] – the Redis-backed virtual waiting room queue engine with
//!   token-bucket admission and signed checkout-grant issuance (Issue #1187).

pub mod pow;
pub mod queue;
