use tracing_subscriber::{fmt, layer::SubscriberExt, util::SubscriberInitExt, EnvFilter};

/// Initialize a global tracing subscriber for the application.
///
/// This setup uses `EnvFilter` to configure log levels from the `RUST_LOG`
/// environment variable (falling back to `info` if not set) and formats
/// logs for the stdout console.
/// Initialize a global tracing subscriber for the application.
pub fn init_logging() {
    let env_filter = EnvFilter::try_from_default_env().unwrap_or_else(|_| EnvFilter::new("info"));

    // We use `try_init` here to be more resilient in test environments
    // where the initializer might be called multiple times.
    let _ = tracing_subscriber::registry()
        .with(env_filter)
        .with(fmt::layer())
        .try_init();
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_init_logging_idempotent() {
        // Calling it twice should not panic.
        init_logging();
        init_logging();

        // At this point we just want to ensure that no panic occurred
        // and that some logging still works.
        tracing::info!("Test logging works");
    }
}
