//! # Re-org Resilient Indexer Benchmarks (Issue #1178)
//!
//! Two halves:
//!
//! 1. **Pure decode benchmarks** (`decode_topic_symbol`, `event_kind_for_topic`,
//!    `backoff_with_jitter`, `RpcNodePool` rotation) — no I/O, always run.
//! 2. **DB-gated throughput benchmarks** — buffered-event insert throughput
//!    and the ledger-rollback path exercised by
//!    `server/tests/chaos_reorg_simulation.rs` when simulating a 5-ledger
//!    Stellar re-org, run only when `DATABASE_URL` is set.
//!
//! ```bash
//! cargo bench --bench reorg_indexer_bench                     # decode-only
//! DATABASE_URL=postgres://... cargo bench --bench reorg_indexer_bench  # full suite
//! ```

use agora_server::models::indexer_event::{decode_topic_symbol, event_kind_for_topic, IndexedEvent};
use agora_server::models::blockchain_checkpoint::CheckpointStore;
use agora_server::services::indexer::{backoff_with_jitter, buffer_insert, RpcNodePool};
use criterion::{black_box, BenchmarkId, Criterion};
use serde_json::json;
use sqlx::PgPool;
use std::time::Duration;
use stellar_xdr::curr::{Limits, ScSymbol, ScVal, WriteXdr};
use uuid::Uuid;

// ---------------------------------------------------------------------------
// Pure decode benchmarks (no I/O)
// ---------------------------------------------------------------------------

fn bench_decode_topic_symbol(c: &mut Criterion) {
    let topic = ScVal::Symbol(ScSymbol("PaymentProcessed".parse().unwrap()))
        .to_xdr_base64(Limits::none())
        .unwrap();

    c.bench_function("decode_topic_symbol", |b| {
        b.iter(|| decode_topic_symbol(Some(black_box(&topic))))
    });
}

fn bench_event_kind_for_topic(c: &mut Criterion) {
    let topics = [
        "PaymentProcessed",
        "BulkRefundProcessed",
        "TicketTransferred",
        "EventRegistered",
        "CollateralStaked",
        "TotallyUnknownEvent",
    ];
    let mut group = c.benchmark_group("event_kind_for_topic");
    for topic in topics {
        group.bench_with_input(BenchmarkId::from_parameter(topic), &topic, |b, topic| {
            b.iter(|| event_kind_for_topic(black_box(topic)))
        });
    }
    group.finish();
}

fn bench_indexed_event_decode_end_to_end(c: &mut Criterion) {
    let topic = ScVal::Symbol(ScSymbol("PaymentProcessed".parse().unwrap()))
        .to_xdr_base64(Limits::none())
        .unwrap();
    let value = json!({
        "payment_id": "pay-1",
        "event_id": Uuid::new_v4().to_string(),
        "buyer": "GABC",
        "owner": "GDEF",
        "amount": 5_000_000
    });

    c.bench_function("indexed_event_decode_purchase", |b| {
        b.iter(|| {
            IndexedEvent::decode(
                black_box("evt-1".to_string()),
                black_box(100),
                black_box("CPAY".to_string()),
                black_box(std::slice::from_ref(&topic)),
                black_box(&value),
                black_box(105),
            )
        })
    });
}

fn bench_backoff_with_jitter(c: &mut Criterion) {
    let mut group = c.benchmark_group("backoff_with_jitter");
    for attempt in [0u32, 4, 8, 16] {
        group.bench_with_input(
            BenchmarkId::from_parameter(attempt),
            &attempt,
            |b, &attempt| {
                b.iter(|| {
                    backoff_with_jitter(
                        black_box(attempt),
                        Duration::from_secs(5),
                        Duration::from_secs(300),
                    )
                })
            },
        );
    }
    group.finish();
}

fn bench_rpc_node_pool_rotation(c: &mut Criterion) {
    c.bench_function("rpc_node_pool_next_url_3_nodes", |b| {
        let mut pool = RpcNodePool::new(
            vec![
                "https://rpc-a.example".to_string(),
                "https://rpc-b.example".to_string(),
                "https://rpc-c.example".to_string(),
            ],
            Duration::from_secs(60),
        );
        b.iter(|| black_box(pool.next_url()))
    });
}

// ---------------------------------------------------------------------------
// DB-gated: re-org rollback & buffer throughput
// ---------------------------------------------------------------------------

/// Ledger base far above any plausible real Stellar ledger sequence (mainnet
/// sits in the tens of millions as of 2026) so this benchmark's synthetic
/// rows can never be confused with — or accidentally purge — real indexer
/// data if pointed at a populated database. See the equivalent guard in
/// `server/tests/chaos_reorg_simulation.rs` for the full rationale.
const SYNTHETIC_LEDGER_BASE: i64 = 3_900_000_000;

async fn seed_buffer_window(pool: &PgPool, contract_id: &str, count: i64) {
    for i in 0..count {
        let ledger = SYNTHETIC_LEDGER_BASE + i;
        let event = IndexedEvent::decode(
            format!("{contract_id}-evt-{i}"),
            ledger as u32,
            contract_id.to_string(),
            &[],
            &json!({}),
            ledger as u32 + 10,
        );
        buffer_insert(pool, &event).await.expect("buffer_insert");
    }
}

fn bench_buffer_insert_throughput(c: &mut Criterion, rt: &tokio::runtime::Runtime, pool: &PgPool) {
    let contract_id = format!("bench-buffer-{}", Uuid::new_v4());
    let mut i = 0i64;

    c.bench_function("buffer_insert_single_event", |b| {
        b.to_async(rt).iter(|| {
            i += 1;
            let pool = pool.clone();
            let contract_id = contract_id.clone();
            let ledger = SYNTHETIC_LEDGER_BASE + i;
            async move {
                let event = IndexedEvent::decode(
                    format!("{contract_id}-{ledger}-{i}"),
                    ledger as u32,
                    contract_id,
                    &[],
                    &json!({}),
                    ledger as u32 + 10,
                );
                black_box(buffer_insert(&pool, &event).await.unwrap());
            }
        })
    });

    rt.block_on(async {
        let _ = sqlx::query("DELETE FROM indexer_ledger_events WHERE contract_id = $1")
            .bind(&contract_id)
            .execute(pool)
            .await;
    });
}

fn bench_rollback_by_window_size(c: &mut Criterion, rt: &tokio::runtime::Runtime, pool: &PgPool) {
    let mut group = c.benchmark_group("rollback_to_by_buffered_window");
    for window in [5i64, 50, 500] {
        let contract_id = format!("bench-rollback-{}", Uuid::new_v4());
        let chain_key = format!("bench:reorg:{contract_id}");

        group.bench_with_input(
            BenchmarkId::from_parameter(window),
            &window,
            |b, &window| {
                b.to_async(rt).iter_batched(
                    || {
                        let pool = pool.clone();
                        let contract_id = contract_id.clone();
                        let chain_key = chain_key.clone();
                        async move {
                            seed_buffer_window(&pool, &contract_id, window).await;
                            CheckpointStore::save(
                                &pool,
                                &chain_key,
                                SYNTHETIC_LEDGER_BASE + window,
                                None,
                            )
                            .await
                            .unwrap();
                        }
                    },
                    |setup| {
                        let pool = pool.clone();
                        let chain_key = chain_key.clone();
                        async move {
                            setup.await;
                            black_box(
                                CheckpointStore::rollback_to(&pool, &chain_key, SYNTHETIC_LEDGER_BASE)
                                    .await
                                    .unwrap(),
                            );
                        }
                    },
                    criterion::BatchSize::SmallInput,
                )
            },
        );

        rt.block_on(async {
            let _ = sqlx::query("DELETE FROM indexer_ledger_events WHERE contract_id = $1")
                .bind(&contract_id)
                .execute(pool)
                .await;
            let _ = sqlx::query("DELETE FROM blockchain_checkpoints WHERE chain_key = $1")
                .bind(&chain_key)
                .execute(pool)
                .await;
        });
    }
    group.finish();
}

fn main() {
    let mut criterion = Criterion::default().configure_from_args();

    bench_decode_topic_symbol(&mut criterion);
    bench_event_kind_for_topic(&mut criterion);
    bench_indexed_event_decode_end_to_end(&mut criterion);
    bench_backoff_with_jitter(&mut criterion);
    bench_rpc_node_pool_rotation(&mut criterion);

    match std::env::var("DATABASE_URL") {
        Ok(database_url) => {
            let rt = tokio::runtime::Runtime::new().expect("tokio runtime");
            let pool = rt
                .block_on(PgPool::connect(&database_url))
                .expect("connect to DATABASE_URL");
            bench_buffer_insert_throughput(&mut criterion, &rt, &pool);
            bench_rollback_by_window_size(&mut criterion, &rt, &pool);
        }
        Err(_) => {
            eprintln!(
                "reorg_indexer_bench: DATABASE_URL not set, skipping DB-gated benchmarks \
                 (decode benchmarks above still ran)."
            );
        }
    }

    criterion.final_summary();
}
