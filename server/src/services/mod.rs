//! # Services Module
//!
//! Reusable background services that are not tied to a single HTTP handler:
//! - [`pow`] – SHA-256 proof-of-work challenge (Hashcash-style) used to gate
//!   entrance into the virtual waiting room (Issue #1187).
//! - [`queue`] – the Redis-backed virtual waiting room queue engine with
//!   token-bucket admission and signed checkout-grant issuance (Issue #1187).
//! - [`indexer`] – high-throughput, re-org resilient Soroban event indexer
//!   with producer-consumer pipeline, ledger cursor checkpoints, and
//!   historical replay (Issue #1174).

pub mod email_dispatch;
pub mod indexer;
pub mod pow;
pub mod queue;
pub mod webhook_dispatcher;
