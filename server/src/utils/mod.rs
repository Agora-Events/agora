pub mod cursor_pagination;
pub mod db_timer;
pub mod error;
pub mod ics;
pub mod logging;
pub mod pagination;
pub mod pdf;
pub mod rate_limit;
pub mod response;
pub mod zkp_verifier;

// Utility helpers (hashing, validation) will be added here

#[cfg(test)]
mod docker_compose_tests;
