//! # Handler & Middleware Stack Benchmarks (Issue #1178)
//!
//! Drives real Axum handlers through the same `tower::ServiceExt::oneshot`
//! path used by the existing integration tests (see
//! `server/tests/health_integration.rs`) — no TCP listener, no external
//! services — so these benchmarks isolate in-process request handling cost:
//! request-id propagation, tracing, panic catching, and per-IP rate limiting.
//!
//! Also benchmarks the SHA-256 proof-of-work gate
//! (`agora_server::services::pow`) that guards entry into the virtual
//! waiting room, since its cost directly bounds how fast the `queue` phase
//! of the k6 stress script (`scripts/stress_test.js`) can move.
//!
//! ```bash
//! cargo bench --bench handler_bench
//! ```

use agora_server::config::request_id::{propagate_request_id_layer, set_request_id_layer};
use agora_server::middleware::catch_panic::catch_panic_layer;
use agora_server::middleware::request_id_tracing::{propagate_request_id, trace_request_id};
use agora_server::services::pow::{generate_challenge, solve_pow, verify_pow, DEFAULT_DIFFICULTY};
use agora_server::utils::rate_limit::RateLimitLayer;
use axum::{body::Body, http::Request, middleware, routing::get, Router};
use criterion::{black_box, criterion_group, criterion_main, BenchmarkId, Criterion};
use tower::ServiceExt;

// ---------------------------------------------------------------------------
// Router builders
// ---------------------------------------------------------------------------

/// The same middleware stack `routes::create_routes` applies, minus CORS /
/// security headers / DB-backed layers so the benchmark stays hermetic.
fn full_stack_router() -> Router {
    Router::new()
        .route("/ping", get(|| async { "pong" }))
        .layer(middleware::from_fn(trace_request_id))
        .layer(middleware::from_fn(propagate_request_id))
        .layer(propagate_request_id_layer())
        .layer(set_request_id_layer())
        .layer(catch_panic_layer())
}

fn rate_limited_router(max: usize) -> Router {
    full_stack_router().layer(RateLimitLayer::new(max, std::time::Duration::from_secs(60)))
}

fn get_request(path: &str) -> Request<Body> {
    Request::builder().uri(path).body(Body::empty()).unwrap()
}

// ---------------------------------------------------------------------------
// Benchmarks
// ---------------------------------------------------------------------------

fn bench_middleware_stack_roundtrip(c: &mut Criterion) {
    let rt = tokio::runtime::Runtime::new().unwrap();
    let router = full_stack_router();

    c.bench_function("middleware_stack_oneshot_roundtrip", |b| {
        b.to_async(&rt).iter(|| {
            let router = router.clone();
            async move {
                let resp = router.oneshot(get_request("/ping")).await.unwrap();
                black_box(resp.status());
            }
        })
    });
}

fn bench_rate_limited_roundtrip_below_capacity(c: &mut Criterion) {
    let rt = tokio::runtime::Runtime::new().unwrap();
    // A fresh router per invocation (not per-iteration) with headroom well
    // above criterion's sample count so every request in the sample stays
    // in the "allowed" branch of the token bucket — we're isolating the
    // bucket's steady-state overhead, not its rejection path.
    let router = rate_limited_router(1_000_000);

    c.bench_function("rate_limit_layer_allowed_request", |b| {
        b.to_async(&rt).iter(|| {
            let router = router.clone();
            async move {
                let resp = router.oneshot(get_request("/ping")).await.unwrap();
                black_box(resp.status());
            }
        })
    });
}

fn bench_pow_challenge_generation(c: &mut Criterion) {
    c.bench_function("pow_generate_challenge", |b| {
        b.iter(generate_challenge)
    });
}

fn bench_pow_verify(c: &mut Criterion) {
    let challenge = generate_challenge();
    let nonce = solve_pow(&challenge, DEFAULT_DIFFICULTY).expect("solvable at default difficulty");

    c.bench_function("pow_verify_single_hash", |b| {
        b.iter(|| verify_pow(black_box(&challenge), black_box(&nonce), DEFAULT_DIFFICULTY))
    });
}

/// The waiting-room `join` step actually pays this cost client-side, but
/// benchmarking it here quantifies the CPU budget the k6 stress script
/// (which solves the same puzzle in JS) needs to model per admitted client.
fn bench_pow_solve_by_difficulty(c: &mut Criterion) {
    let mut group = c.benchmark_group("pow_solve");
    group.sample_size(20); // solving is expensive; keep wall-clock reasonable
    for difficulty in [1u32, 2, 3, DEFAULT_DIFFICULTY] {
        group.bench_with_input(
            BenchmarkId::from_parameter(difficulty),
            &difficulty,
            |b, &difficulty| {
                b.iter_batched(
                    generate_challenge,
                    |challenge| solve_pow(black_box(&challenge), difficulty),
                    criterion::BatchSize::SmallInput,
                )
            },
        );
    }
    group.finish();
}

criterion_group!(
    benches,
    bench_middleware_stack_roundtrip,
    bench_rate_limited_roundtrip_below_capacity,
    bench_pow_challenge_generation,
    bench_pow_verify,
    bench_pow_solve_by_difficulty,
);
criterion_main!(benches);
