//! # Chaos Test: Network Latency Spikes (Issue #1178)
//!
//! Exercises [`common::chaos::LatencyInjector`] — the harness for the
//! "network latency spikes (100ms–5000ms)" fault class called out in the
//! issue — both in isolation and wrapped around a real database round trip,
//! to prove injected latency neither corrupts results nor can hang forever.

mod common;

use common::chaos::LatencyInjector;
use common::test_pool;
use std::time::Duration;

#[tokio::test]
async fn injected_latency_falls_within_configured_range() {
    // A tight, fast range for routine CI runs — the full issue-specified
    // 100ms-5000ms spike is covered by the #[ignore] test below.
    let injector = LatencyInjector::new(Duration::from_millis(30), Duration::from_millis(90));

    for _ in 0..5 {
        let start = tokio::time::Instant::now();
        injector.strike().await;
        let elapsed = start.elapsed();
        assert!(
            elapsed >= Duration::from_millis(30),
            "strike() must never resolve faster than the configured minimum, got {elapsed:?}"
        );
        // Generous slack for scheduler jitter on a loaded CI runner.
        assert!(
            elapsed < Duration::from_secs(2),
            "strike() took implausibly long ({elapsed:?}) — possible harness hang"
        );
    }
}

/// Validates the exact 100ms-5000ms spike range from Issue #1178 end to end.
/// Marked `#[ignore]` because a full pass can take several seconds; run
/// explicitly with `cargo test --test chaos_network_latency -- --ignored`.
#[tokio::test]
#[ignore]
async fn full_issue_specified_spike_range_100ms_to_5000ms() {
    let injector = LatencyInjector::spike_range();
    let mut min_seen = Duration::from_secs(10);
    let mut max_seen = Duration::ZERO;

    for _ in 0..15 {
        let start = tokio::time::Instant::now();
        injector.strike().await;
        let elapsed = start.elapsed();
        assert!(elapsed >= Duration::from_millis(100));
        assert!(elapsed <= Duration::from_millis(6000), "generous slack above the 5000ms cap");
        min_seen = min_seen.min(elapsed);
        max_seen = max_seen.max(elapsed);
    }

    eprintln!("spike range observed over 15 draws: {min_seen:?}..={max_seen:?}");
}

#[tokio::test]
async fn wrap_preserves_inner_future_result_under_latency() {
    let injector = LatencyInjector::new(Duration::from_millis(10), Duration::from_millis(40));
    let (result, delay) = injector.wrap(async { 2 + 2 }).await;
    assert_eq!(result, 4, "injected latency must never alter the wrapped computation's outcome");
    assert!(delay >= Duration::from_millis(10) && delay <= Duration::from_millis(40));
}

/// Regression guard: the harness itself must be boundable. A chaos tool that
/// can hang forever is worse than no chaos tool at all.
#[tokio::test]
async fn latency_injection_cannot_hang_indefinitely() {
    let injector = LatencyInjector::spike_range(); // up to 5000ms
    let outcome = tokio::time::timeout(Duration::from_secs(8), injector.wrap(async { "done" })).await;
    assert!(outcome.is_ok(), "a single spike-range strike must resolve within 8s");
}

/// Wraps a real Postgres round trip in injected latency and asserts the
/// query result is unaffected — proving latency injection is a pure delay,
/// not a source of flakiness in what gets returned. Skips gracefully when
/// `DATABASE_URL` is unset (same convention as `auth_integration.rs`).
#[tokio::test]
async fn latency_injected_db_round_trip_still_returns_correct_result() {
    let Some(pool) = test_pool("latency_injected_db_round_trip_still_returns_correct_result").await
    else {
        return;
    };

    let injector = LatencyInjector::new(Duration::from_millis(50), Duration::from_millis(200));
    let start = tokio::time::Instant::now();

    let (row, delay): (Result<(i32,), sqlx::Error>, Duration) = injector
        .wrap(async { sqlx::query_as::<_, (i32,)>("SELECT 41 + 1").fetch_one(&pool).await })
        .await;

    let elapsed = start.elapsed();
    assert_eq!(row.unwrap().0, 42, "the query result must be exact despite injected latency");
    assert!(elapsed >= delay, "wall-clock elapsed must be at least the injected delay");
}
