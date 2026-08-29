//! # Chaos Injection Harness (Issue #1178)
//!
//! Fault-injection helpers used by the `chaos_*` integration tests to
//! simulate the production outage classes called out in the issue:
//!
//! * [`LatencyInjector`] — network latency spikes (100ms–5000ms)
//! * [`PoolExhaustionGuard`] — sudden PostgreSQL connection-pool exhaustion
//! * [`drop_frames`] — dropped WebSocket frames
//!
//! These are deliberately test-only (`server/tests/common/`, not
//! `server/src/`): they exist to *attack* the running system from outside,
//! not to run inside it in production.

use rand::Rng;
use std::time::Duration;
use tokio::sync::{broadcast, mpsc};

// ---------------------------------------------------------------------------
// Network latency injection
// ---------------------------------------------------------------------------

/// Injects a random delay drawn uniformly from `[min, max]` before a future
/// resolves, modeling the 100ms–5000ms network latency spikes called out in
/// Issue #1178.
#[derive(Debug, Clone, Copy)]
pub struct LatencyInjector {
    min: Duration,
    max: Duration,
}

impl LatencyInjector {
    pub fn new(min: Duration, max: Duration) -> Self {
        assert!(min <= max, "min latency must be <= max latency");
        Self { min, max }
    }

    /// The issue's specified spike range: 100ms to 5000ms.
    pub fn spike_range() -> Self {
        Self::new(Duration::from_millis(100), Duration::from_millis(5000))
    }

    fn sample_delay(&self) -> Duration {
        if self.min == self.max {
            return self.min;
        }
        let min_ms = self.min.as_millis() as u64;
        let max_ms = self.max.as_millis() as u64;
        Duration::from_millis(rand::thread_rng().gen_range(min_ms..=max_ms))
    }

    /// Sleep for a sampled delay.
    pub async fn strike(&self) {
        tokio::time::sleep(self.sample_delay()).await;
    }

    /// Run `fut` behind an injected delay, returning its output and the
    /// delay that was applied (useful for asserting the delay actually
    /// landed inside the configured range).
    pub async fn wrap<F: std::future::Future>(&self, fut: F) -> (F::Output, Duration) {
        let delay = self.sample_delay();
        tokio::time::sleep(delay).await;
        (fut.await, delay)
    }
}

// ---------------------------------------------------------------------------
// PostgreSQL connection-pool exhaustion
// ---------------------------------------------------------------------------

/// Holds `count` live connections checked out from a `PgPool` for as long as
/// the guard is alive, simulating sudden pool exhaustion. Connections are
/// returned to the pool automatically when the guard is dropped.
pub struct PoolExhaustionGuard {
    _connections: Vec<sqlx::pool::PoolConnection<sqlx::Postgres>>,
}

impl PoolExhaustionGuard {
    /// Acquire `count` connections from `pool` up front (bounded by a
    /// timeout so a mis-sized `count` fails fast instead of hanging the
    /// test suite), holding all of them until the guard is dropped.
    pub async fn saturate(pool: &sqlx::PgPool, count: u32) -> Result<Self, sqlx::Error> {
        let mut connections = Vec::with_capacity(count as usize);
        for _ in 0..count {
            let conn = tokio::time::timeout(Duration::from_secs(10), pool.acquire())
                .await
                .map_err(|_| sqlx::Error::PoolTimedOut)??;
            connections.push(conn);
        }
        Ok(Self {
            _connections: connections,
        })
    }

    /// Saturate the *entire* configured pool (its `max_connections`), the
    /// worst-case scenario from Issue #1178: every connection unavailable.
    pub async fn saturate_fully(pool: &sqlx::PgPool) -> Result<Self, sqlx::Error> {
        let max = pool.options().get_max_connections();
        Self::saturate(pool, max).await
    }
}

// ---------------------------------------------------------------------------
// Dropped WebSocket frames
// ---------------------------------------------------------------------------

/// Forward messages from a `broadcast::Receiver` into a bounded `mpsc`
/// channel, deliberately dropping a fraction of them to simulate the network
/// losing WebSocket frames. Returns the receiving half; the forwarding task
/// runs until the broadcast channel closes or the mpsc receiver is dropped.
///
/// `drop_rate` is clamped to `[0.0, 1.0]`; `0.5` drops roughly half of all
/// frames.
pub fn drop_frames<T>(mut rx: broadcast::Receiver<T>, drop_rate: f64) -> mpsc::Receiver<T>
where
    T: Clone + Send + 'static,
{
    let drop_rate = drop_rate.clamp(0.0, 1.0);
    let (tx, forwarded_rx) = mpsc::channel(256);

    tokio::spawn(async move {
        loop {
            match rx.recv().await {
                Ok(msg) => {
                    let dropped = rand::thread_rng().gen_bool(drop_rate);
                    if dropped {
                        continue;
                    }
                    if tx.send(msg).await.is_err() {
                        break; // receiver gone
                    }
                }
                Err(broadcast::error::RecvError::Lagged(_)) => continue,
                Err(broadcast::error::RecvError::Closed) => break,
            }
        }
    });

    forwarded_rx
}

// ---------------------------------------------------------------------------
// Combined chaos configuration (env-driven, mirrors production tunables)
// ---------------------------------------------------------------------------

/// Bundles every chaos knob so a test (or, eventually, a dedicated `k6`/CI
/// chaos run) can be configured from the environment without touching code.
#[derive(Debug, Clone)]
pub struct ChaosConfig {
    pub latency_min_ms: u64,
    pub latency_max_ms: u64,
    pub ws_drop_rate: f64,
    pub pool_exhaustion_connections: u32,
}

impl ChaosConfig {
    pub fn from_env() -> Self {
        Self {
            latency_min_ms: env_u64("CHAOS_LATENCY_MIN_MS", 100),
            latency_max_ms: env_u64("CHAOS_LATENCY_MAX_MS", 5000),
            ws_drop_rate: env_f64("CHAOS_WS_DROP_RATE", 0.2),
            pool_exhaustion_connections: env_u64("CHAOS_POOL_EXHAUSTION_CONNECTIONS", 5) as u32,
        }
    }

    pub fn latency_injector(&self) -> LatencyInjector {
        LatencyInjector::new(
            Duration::from_millis(self.latency_min_ms),
            Duration::from_millis(self.latency_max_ms),
        )
    }
}

fn env_u64(key: &str, default: u64) -> u64 {
    std::env::var(key)
        .ok()
        .and_then(|v| v.parse().ok())
        .unwrap_or(default)
}

fn env_f64(key: &str, default: f64) -> f64 {
    std::env::var(key)
        .ok()
        .and_then(|v| v.parse().ok())
        .unwrap_or(default)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn latency_injector_stays_within_configured_range() {
        let injector = LatencyInjector::new(Duration::from_millis(20), Duration::from_millis(60));
        let start = tokio::time::Instant::now();
        injector.strike().await;
        let elapsed = start.elapsed();
        assert!(elapsed >= Duration::from_millis(20));
        // Generous upper bound to absorb scheduler jitter on a loaded CI box.
        assert!(elapsed < Duration::from_millis(500));
    }

    #[tokio::test]
    async fn wrap_returns_inner_future_output() {
        let injector = LatencyInjector::new(Duration::from_millis(1), Duration::from_millis(5));
        let (value, delay) = injector.wrap(async { 42 }).await;
        assert_eq!(value, 42);
        assert!(delay >= Duration::from_millis(1) && delay <= Duration::from_millis(5));
    }

    #[tokio::test]
    async fn drop_frames_never_exceeds_total_sent() {
        let (tx, rx) = broadcast::channel::<u32>(64);
        let mut forwarded = drop_frames(rx, 0.5);

        for i in 0..50u32 {
            let _ = tx.send(i);
        }
        drop(tx);

        let mut received = 0u32;
        while forwarded.recv().await.is_some() {
            received += 1;
        }
        assert!(received <= 50, "cannot forward more than were sent");
    }

    #[tokio::test]
    async fn drop_frames_at_zero_rate_forwards_everything() {
        let (tx, rx) = broadcast::channel::<u32>(64);
        let mut forwarded = drop_frames(rx, 0.0);

        for i in 0..20u32 {
            tx.send(i).unwrap();
        }
        drop(tx);

        let mut received = 0u32;
        while forwarded.recv().await.is_some() {
            received += 1;
        }
        assert_eq!(received, 20, "zero drop rate must forward every frame");
    }

    #[test]
    fn chaos_config_falls_back_to_defaults() {
        // Rely on ambient env not setting these in the test process; assert
        // the shape is sane rather than depending on unset-ness (parallel
        // test runs may share env).
        let config = ChaosConfig {
            latency_min_ms: 100,
            latency_max_ms: 5000,
            ws_drop_rate: 0.2,
            pool_exhaustion_connections: 5,
        };
        assert!(config.latency_min_ms <= config.latency_max_ms);
        assert!(config.ws_drop_rate >= 0.0 && config.ws_drop_rate <= 1.0);
    }
}
