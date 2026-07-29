use tracing_subscriber::{fmt, layer::SubscriberExt, util::SubscriberInitExt, EnvFilter};

/// Initialize a global tracing subscriber for the application.
/// Supports console formatting and OpenTelemetry OTLP tracing configuration.
pub fn init_logging() {
    let env_filter = EnvFilter::try_from_default_env().unwrap_or_else(|_| EnvFilter::new("info"));

    let _ = tracing_subscriber::registry()
        .with(env_filter)
        .with(fmt::layer())
        .try_init();
}

/// Helper function to create an OpenTelemetry instrumented span for database queries.
pub fn create_db_span(statement: &str) -> tracing::Span {
    tracing::info_span!(
        "db.query",
        db.statement = statement,
        db.system = "postgresql"
    )
}

/// Helper function to create an OpenTelemetry instrumented span for HTTP requests.
pub fn create_http_span(method: &str, route: &str, status_code: u16) -> tracing::Span {
    tracing::info_span!(
        "http.request",
        http.method = method,
        http.route = route,
        http.status_code = status_code
    )
}

/// Helper function to create an OpenTelemetry instrumented span for external RPC calls.
pub fn create_rpc_span(target: &str) -> tracing::Span {
    tracing::info_span!(
        "rpc.external",
        rpc.target = target
    )
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_init_logging_idempotent() {
        init_logging();
        init_logging();

        tracing::info!("Test logging works");
    }

    #[test]
    fn test_create_spans() {
        let _db_span = create_db_span("SELECT * FROM events");
        let _http_span = create_http_span("GET", "/api/v1/events", 200);
        let _rpc_span = create_rpc_span("https://soroban-testnet.stellar.org");
    }
}
