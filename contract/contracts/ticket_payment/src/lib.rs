#![no_std]
#![warn(missing_docs)]
//! Ticket Payment smart contract for the Agora platform.
//!
//! This contract handles ticket purchases, escrow management, fee settlement,
//! refunds, transfers, and governance for the Agora event ticketing system.

pub mod contract;
pub mod error;
pub mod events;
pub mod governance;
pub mod interfaces;
pub mod keys;
pub mod payment_types;
pub mod storage;
pub mod types;

#[cfg(test)]
mod test;

#[cfg(test)]
mod test_e2e;
