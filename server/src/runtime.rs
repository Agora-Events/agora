//! Process runtime state shared across the binary and library.
//!
//! This module lives in the library crate so that handlers (e.g. `health`)
//! can reference uptime without creating a circular dependency on `main`.

use std::sync::LazyLock;
use std::time::Instant;

/// Process start time, captured once at boot (Issue #1428).
static START_TIME: LazyLock<Instant> = LazyLock::new(Instant::now);

/// Return the number of seconds since the server process started.
pub fn uptime_seconds() -> u64 {
    START_TIME.elapsed().as_secs()
}
