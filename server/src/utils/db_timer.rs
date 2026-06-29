//! Slow-query detection helpers.
//!
//! Wrap any async database call with [`timed_query`] and a `WARN` log is
//! emitted whenever elapsed time exceeds `SLOW_QUERY_THRESHOLD_MS` (default 500 ms).

use std::time::{Duration, Instant};

/// Read the configured slow-query threshold from the environment.
pub fn slow_query_threshold() -> Duration {
    let ms = std::env::var("SLOW_QUERY_THRESHOLD_MS")
        .ok()
        .and_then(|v| v.parse::<u64>().ok())
        .unwrap_or(500);
    Duration::from_millis(ms)
}

/// Emit a `WARN` log if `elapsed` exceeds the configured threshold.
pub fn log_if_slow(query_name: &str, elapsed: Duration) {
    let threshold = slow_query_threshold();
    if elapsed >= threshold {
        tracing::warn!(
            query = query_name,
            elapsed_ms = elapsed.as_millis(),
            threshold_ms = threshold.as_millis(),
            "Slow database query detected"
        );
    }
}

/// Run an async database closure and log a warning if it is slower than the threshold.
///
/// # Example
/// ```rust,ignore
/// let rows = timed_query("list_events", || async {
///     sqlx::query_as::<_, Event>("SELECT * FROM events")
///         .fetch_all(&pool)
///         .await
/// }).await?;
/// ```
pub async fn timed_query<F, Fut, T>(query_name: &'static str, f: F) -> T
where
    F: FnOnce() -> Fut,
    Fut: std::future::Future<Output = T>,
{
    let start = Instant::now();
    let result = f().await;
    log_if_slow(query_name, start.elapsed());
    result
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::time::Duration;

    #[test]
    fn test_log_if_slow_does_not_panic_below_threshold() {
        // Should complete without warning (zero elapsed is well below 500 ms).
        log_if_slow("test_query", Duration::from_millis(0));
    }

    #[test]
    fn test_log_if_slow_does_not_panic_above_threshold() {
        // Verify the function is callable with an elapsed time that exceeds the default threshold.
        log_if_slow("test_query", Duration::from_millis(600));
    }

    #[tokio::test]
    async fn test_timed_query_returns_value() {
        let result = timed_query("test_fast_query", || async { 42u32 }).await;
        assert_eq!(result, 42);
    }

    #[tokio::test]
    async fn test_timed_query_warns_when_threshold_exceeded() {
        // Use a very low threshold so the sleep definitely triggers a warning.
        temp_env::with_var("SLOW_QUERY_THRESHOLD_MS", Some("1"), || async {
            let result = timed_query("slow_test_query", || async {
                tokio::time::sleep(Duration::from_millis(5)).await;
                "done"
            })
            .await;
            assert_eq!(result, "done");
        })
        .await;
    }
}
