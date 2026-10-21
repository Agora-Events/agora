//! Slow-query detection helpers.
//!
//! Wrap any async database call with [`timed_query`] and a `WARN` log is
//! emitted whenever elapsed time exceeds `SLOW_QUERY_THRESHOLD_MS` (default 500 ms).
//! An `ERROR` log is emitted at 5x the threshold. Metrics are also incremented.

use std::time::{Duration, Instant};

/// Default threshold for slow query detection (milliseconds).
const DEFAULT_SLOW_QUERY_THRESHOLD_MS: u64 = 500;

/// Multiplier for the error threshold (errors logged at 5x the warn threshold).
const ERROR_THRESHOLD_MULTIPLIER: u64 = 5;

/// Read the configured slow-query threshold from the environment.
pub fn slow_query_threshold() -> Duration {
    let ms = std::env::var("SLOW_QUERY_THRESHOLD_MS")
        .ok()
        .and_then(|v| v.parse::<u64>().ok())
        .unwrap_or(DEFAULT_SLOW_QUERY_THRESHOLD_MS);
    Duration::from_millis(ms)
}

/// Emit structured logs and metrics if `elapsed` exceeds the configured threshold.
///
/// - WARN log when `elapsed >= threshold`
/// - ERROR log when `elapsed >= threshold * 5`
/// - Increments `db_slow_queries_total` metric
///
/// Fields in logs: `query_name`, `duration_ms`, `threshold_ms`
///
/// # Safety
/// Never logs query parameter values (which may contain PII or wallet data).
pub fn log_if_slow(query_name: &str, elapsed: Duration) {
    let threshold = slow_query_threshold();
    let elapsed_ms = elapsed.as_millis() as u64;
    let threshold_ms = threshold.as_millis() as u64;

    if elapsed >= threshold {
        // Increment metric for any slow query
        crate::metrics::increment_slow_query(query_name);

        // Determine if this is an error-level slow query
        let error_threshold_ms = threshold_ms * ERROR_THRESHOLD_MULTIPLIER;
        if elapsed_ms >= error_threshold_ms {
            tracing::error!(
                query_name = query_name,
                duration_ms = elapsed_ms,
                threshold_ms = threshold_ms,
                error_threshold_ms = error_threshold_ms,
                "Very slow database query detected (exceeds error threshold)"
            );
        } else {
            tracing::warn!(
                query_name = query_name,
                duration_ms = elapsed_ms,
                threshold_ms = threshold_ms,
                "Slow database query detected"
            );
        }
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
    use std::collections::HashMap;
    use std::sync::{Arc, Mutex};
    use std::time::Duration;
    use tracing::field::{Field, Visit};
    use tracing::Subscriber;
    use tracing_subscriber::layer::{Context, SubscriberExt};
    use tracing_subscriber::Layer;

    #[derive(Default)]
    struct FieldCollector {
        values: HashMap<&'static str, String>,
    }

    impl Visit for FieldCollector {
        fn record_u64(&mut self, field: &Field, value: u64) {
            self.values.insert(field.name(), value.to_string());
        }

        fn record_i64(&mut self, field: &Field, value: i64) {
            self.values.insert(field.name(), value.to_string());
        }

        fn record_str(&mut self, field: &Field, value: &str) {
            self.values.insert(field.name(), value.to_string());
        }

        fn record_debug(&mut self, field: &Field, value: &dyn std::fmt::Debug) {
            self.values.insert(field.name(), format!("{:?}", value));
        }
    }

    #[derive(Clone, Default)]
    struct CaptureLayer {
        captured: Arc<Mutex<Vec<HashMap<&'static str, String>>>>,
    }

    impl<S> Layer<S> for CaptureLayer
    where
        S: Subscriber,
    {
        fn on_event(&self, event: &tracing::Event<'_>, _ctx: Context<'_, S>) {
            let mut collector = FieldCollector::default();
            event.record(&mut collector);
            self.captured.lock().unwrap().push(collector.values);
        }
    }

    #[test]
    fn test_log_if_slow_does_not_panic_below_threshold() {
        log_if_slow("test_query", Duration::from_millis(0));
    }

    #[test]
    fn test_log_if_slow_does_not_panic_above_threshold() {
        log_if_slow("test_query", Duration::from_millis(600));
    }

    #[test]
    fn test_log_if_slow_emits_structured_fields() {
        let layer = CaptureLayer::default();
        let captured = layer.captured.clone();
        let subscriber = tracing_subscriber::registry().with(layer);

        tracing::subscriber::with_default(subscriber, || {
            temp_env::with_var("SLOW_QUERY_THRESHOLD_MS", Some("10"), || {
                log_if_slow("list_events", Duration::from_millis(25));
            });
        });

        let events = captured.lock().unwrap();
        assert!(
            !events.is_empty(),
            "expected at least one warning event to be emitted"
        );
        let fields = &events[0];
        assert_eq!(
            fields.get("query_name").map(String::as_str),
            Some("list_events")
        );
        assert_eq!(fields.get("duration_ms").map(String::as_str), Some("25"));
        assert_eq!(fields.get("threshold_ms").map(String::as_str), Some("10"));
    }

    #[tokio::test]
    async fn test_timed_query_returns_value() {
        let result = timed_query("test_fast_query", || async { 42u32 }).await;
        assert_eq!(result, 42);
    }

    #[tokio::test]
    async fn test_timed_query_warns_when_threshold_exceeded() {
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
