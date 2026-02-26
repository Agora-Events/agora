#![no_std]
#![allow(clippy::too_many_arguments)]
pub mod contract;
pub mod error;
pub mod events;
pub mod storage;
pub mod types;

#[cfg(test)]
mod test;

#[cfg(test)]
mod test_e2e;
