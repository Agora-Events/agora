use axum::{
    extract::Request,
    http::StatusCode,
    middleware::Next,
    response::{IntoResponse, Response},
};
use once_cell::sync::Lazy;
use prometheus::{
    register_counter_vec_with_registry, register_histogram_vec_with_registry, CounterVec, Encoder,
    HistogramOpts, HistogramVec, Opts, Registry, TextEncoder,
};
use std::time::Instant;

static REGISTRY: Lazy<Registry> = Lazy::new(Registry::new);

pub static HTTP_REQUESTS_TOTAL: Lazy<CounterVec> = Lazy::new(|| {
    register_counter_vec_with_registry!(
        Opts::new("http_requests_total", "Total number of HTTP requests"),
        &["method", "path", "status"],
        REGISTRY
    )
    .unwrap()
});

pub static HTTP_REQUESTS_DURATION_SECONDS: Lazy<HistogramVec> = Lazy::new(|| {
    register_histogram_vec_with_registry!(
        HistogramOpts::new(
            "http_requests_duration_seconds",
            "HTTP request latency in seconds"
        ),
        &["method", "path", "status"],
        REGISTRY
    )
    .unwrap()
});

pub static CACHE_HITS_TOTAL: Lazy<CounterVec> = Lazy::new(|| {
    register_counter_vec_with_registry!(
        Opts::new("cache_hits_total", "Total number of cache hits"),
        &["cache_key"],
        REGISTRY
    )
    .unwrap()
});

pub static CACHE_MISSES_TOTAL: Lazy<CounterVec> = Lazy::new(|| {
    register_counter_vec_with_registry!(
        Opts::new("cache_misses_total", "Total number of cache misses"),
        &["cache_key"],
        REGISTRY
    )
    .unwrap()
});

pub async fn metrics_handler() -> Response {
    let encoder = TextEncoder::new();
    let mut buffer = Vec::new();
    encoder.encode(&REGISTRY.gather(), &mut buffer).unwrap();
    (
        StatusCode::OK,
        [("content-type", "text/plain; version=0.0.4; charset=utf-8")],
        buffer,
    )
        .into_response()
}

pub async fn track_metrics(request: Request, next: Next) -> Response {
    let start = Instant::now();
    let method = request.method().to_string();
    let path = request.uri().path().to_string();

    let response = next.run(request).await;

    let duration = start.elapsed().as_secs_f64();
    let status = response.status().as_u16().to_string();

    HTTP_REQUESTS_TOTAL
        .with_label_values(&[&method, &path, &status])
        .inc();
    HTTP_REQUESTS_DURATION_SECONDS
        .with_label_values(&[&method, &path, &status])
        .observe(duration);

    response
}
