//! # Chaos Test: Sudden PostgreSQL Connection-Pool Exhaustion (Issue #1178)
//!
//! Uses [`common::chaos::PoolExhaustionGuard`] to hold every connection in a
//! small, dedicated pool and asserts the application degrades *predictably*
//! (a bounded, typed timeout) instead of hanging forever — then verifies the
//! pool recovers cleanly the instant connections are released.
//!
//! Runs against its own tiny `max_connections` pool (not the shared test
//! pool other suites use) so saturating it can't starve unrelated tests
//! running concurrently in the same `cargo test` invocation.

mod common;

use common::chaos::PoolExhaustionGuard;
use sqlx::postgres::PgPoolOptions;
use std::time::Duration;

async fn dedicated_pool(max_connections: u32) -> Option<sqlx::PgPool> {
    let Ok(database_url) = std::env::var("DATABASE_URL") else {
        eprintln!("skipping: DATABASE_URL not set");
        return None;
    };
    match PgPoolOptions::new()
        .max_connections(max_connections)
        .acquire_timeout(Duration::from_secs(5))
        .connect(&database_url)
        .await
    {
        Ok(pool) => Some(pool),
        Err(e) => {
            eprintln!("skipping: failed to connect a dedicated pool: {e}");
            None
        }
    }
}

#[tokio::test]
async fn saturated_pool_times_out_predictably_instead_of_hanging() {
    const MAX_CONNECTIONS: u32 = 3;
    let Some(pool) = dedicated_pool(MAX_CONNECTIONS).await else {
        return;
    };

    let guard = PoolExhaustionGuard::saturate(&pool, MAX_CONNECTIONS)
        .await
        .expect("saturate the whole pool");

    // Every connection is held by `guard` — a further acquire must not hang
    // the test suite. `pool.acquire_timeout` (5s, set above) is the
    // production-facing bound; wrap it in an outer timeout too so a harness
    // bug can't turn this into a suite-wide hang even if that bound is
    // somehow not respected.
    let start = tokio::time::Instant::now();
    let outcome = tokio::time::timeout(
        Duration::from_secs(10),
        sqlx::query("SELECT 1").execute(&pool),
    )
    .await;
    let elapsed = start.elapsed();

    match outcome {
        Ok(query_result) => {
            assert!(
                query_result.is_err(),
                "a query against a fully saturated pool must fail, not silently succeed"
            );
            assert!(
                matches!(query_result.unwrap_err(), sqlx::Error::PoolTimedOut),
                "the failure mode under exhaustion should be a typed PoolTimedOut error"
            );
        }
        Err(_elapsed) => panic!(
            "query hung past the 10s outer bound instead of respecting the pool's own \
             5s acquire_timeout — this is the exact failure mode chaos testing exists to catch"
        ),
    }
    assert!(
        elapsed < Duration::from_secs(9),
        "must fail close to the pool's configured acquire_timeout (5s), not linger near the outer bound"
    );

    drop(guard);

    // Once connections are released, the pool must recover immediately —
    // proving exhaustion is a transient backpressure signal, not a
    // permanently wedged state.
    let recovered = tokio::time::timeout(
        Duration::from_secs(5),
        sqlx::query_as::<_, (i32,)>("SELECT 1").fetch_one(&pool),
    )
    .await;
    assert!(
        recovered.is_ok() && recovered.unwrap().is_ok(),
        "pool must recover promptly once exhausting connections are released"
    );
}

#[tokio::test]
async fn saturate_fully_acquires_exactly_max_connections() {
    const MAX_CONNECTIONS: u32 = 4;
    let Some(pool) = dedicated_pool(MAX_CONNECTIONS).await else {
        return;
    };

    assert_eq!(pool.options().get_max_connections(), MAX_CONNECTIONS);

    let guard = PoolExhaustionGuard::saturate_fully(&pool)
        .await
        .expect("saturate_fully should acquire every configured connection");

    let blocked = tokio::time::timeout(
        Duration::from_millis(500),
        sqlx::query("SELECT 1").execute(&pool),
    )
    .await;
    assert!(
        blocked.is_err() || blocked.unwrap().is_err(),
        "with every connection held, a fast-timeout probe must not succeed"
    );

    drop(guard);
}

#[tokio::test]
async fn partial_saturation_leaves_headroom_for_other_queries() {
    const MAX_CONNECTIONS: u32 = 5;
    let Some(pool) = dedicated_pool(MAX_CONNECTIONS).await else {
        return;
    };

    // Hold all but one connection — the classic "nearly exhausted" state
    // that's easy to miss in testing but common in production incidents.
    let guard = PoolExhaustionGuard::saturate(&pool, MAX_CONNECTIONS - 1)
        .await
        .expect("saturate all-but-one connection");

    let result = tokio::time::timeout(
        Duration::from_secs(3),
        sqlx::query_as::<_, (i32,)>("SELECT 7").fetch_one(&pool),
    )
    .await;
    assert!(
        result.is_ok() && result.unwrap().unwrap().0 == 7,
        "the one remaining connection must still serve a request correctly"
    );

    drop(guard);
}
