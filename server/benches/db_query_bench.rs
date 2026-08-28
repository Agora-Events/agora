//! # Database Query Benchmarks (Issue #1178)
//!
//! Benchmarks real PostgreSQL round trips: the ledger-checkpoint upsert that
//! sits on the Soroban indexer's hot path (`CheckpointStore`), and a
//! cursor-paginated ticket listing query against seeded data.
//!
//! Requires a reachable `DATABASE_URL` with migrations applied (same
//! requirement as `server/tests/auth_integration.rs`). When `DATABASE_URL`
//! is unset this binary registers zero benchmarks and exits cleanly instead
//! of failing the build, so `cargo bench` still works in environments
//! without Postgres.
//!
//! ```bash
//! DATABASE_URL=postgres://user:password@localhost:5432/agora cargo bench --bench db_query_bench
//! ```
//!
//! All rows this benchmark writes live under a synthetic organizer created
//! at startup and are removed via `ON DELETE CASCADE` in a teardown step, so
//! it is safe to run repeatedly against a shared dev database. The
//! checkpoint row uses a `bench:` prefixed `chain_key` that can never
//! collide with a real `soroban:*` chain.

use agora_server::models::blockchain_checkpoint::CheckpointStore;
use criterion::{black_box, Criterion};
use sqlx::PgPool;
use uuid::Uuid;

const SEEDED_TICKET_COUNT: i64 = 500;

struct BenchFixture {
    pool: PgPool,
    chain_key: String,
    organizer_id: Uuid,
    event_id: Uuid,
}

impl BenchFixture {
    async fn setup(pool: PgPool) -> Self {
        let organizer_id = Uuid::new_v4();
        let event_id = Uuid::new_v4();
        let chain_key = format!("bench:checkpoint:{organizer_id}");

        sqlx::query(
            "INSERT INTO organizers (id, name, contact_email) VALUES ($1, 'Bench Organizer', 'bench@agora.test')",
        )
        .bind(organizer_id)
        .execute(&pool)
        .await
        .expect("seed organizer");

        sqlx::query(
            "INSERT INTO events (id, organizer_id, title, location, start_time)
             VALUES ($1, $2, 'Bench Event', 'Bench Venue', NOW())",
        )
        .bind(event_id)
        .bind(organizer_id)
        .execute(&pool)
        .await
        .expect("seed event");

        for i in 0..SEEDED_TICKET_COUNT {
            sqlx::query(
                "INSERT INTO tickets (stellar_id, event_id, buyer_wallet, owner_wallet, status)
                 VALUES ($1, $2, $3, $3, 'Unused')",
            )
            .bind(format!("bench-stellar-id-{organizer_id}-{i}"))
            .bind(event_id)
            .bind(format!("GBENCHWALLET{i:06}"))
            .execute(&pool)
            .await
            .expect("seed ticket");
        }

        Self {
            pool,
            chain_key,
            organizer_id,
            event_id,
        }
    }

    /// Deleting the organizer cascades to its event and every seeded ticket
    /// (`events.organizer_id` and `tickets.event_id` are both
    /// `ON DELETE CASCADE`); the checkpoint row is cleaned separately since
    /// it has no FK relationship to organizers.
    async fn teardown(&self) {
        let _ = sqlx::query("DELETE FROM organizers WHERE id = $1")
            .bind(self.organizer_id)
            .execute(&self.pool)
            .await;
        let _ = sqlx::query("DELETE FROM blockchain_checkpoints WHERE chain_key = $1")
            .bind(&self.chain_key)
            .execute(&self.pool)
            .await;
    }
}

fn bench_checkpoint_save_load_roundtrip(c: &mut Criterion, rt: &tokio::runtime::Runtime, fx: &BenchFixture) {
    let mut ledger = 1_000_000i64;
    c.bench_function("checkpoint_save_upsert", |b| {
        b.to_async(rt).iter(|| {
            ledger += 1;
            let pool = fx.pool.clone();
            let chain_key = fx.chain_key.clone();
            async move {
                CheckpointStore::save(&pool, &chain_key, black_box(ledger), None)
                    .await
                    .unwrap();
            }
        })
    });

    c.bench_function("checkpoint_load", |b| {
        b.to_async(rt).iter(|| {
            let pool = fx.pool.clone();
            let chain_key = fx.chain_key.clone();
            async move {
                black_box(CheckpointStore::load(&pool, &chain_key).await.unwrap());
            }
        })
    });
}

fn bench_checkpoint_rollback(c: &mut Criterion, rt: &tokio::runtime::Runtime, fx: &BenchFixture) {
    // Rollback is only meaningfully benchmarked with buffered rows present —
    // seed a small window so the DELETE inside the transaction does real work.
    rt.block_on(async {
        for ledger in 999_900i64..1_000_000 {
            sqlx::query(
                "INSERT INTO indexer_ledger_events (id, ledger, contract_id, topic, value, finalized)
                 VALUES ($1, $2, 'bench-contract', '\"Bench\"'::jsonb, '{}'::jsonb, FALSE)
                 ON CONFLICT (id) DO NOTHING",
            )
            .bind(format!("bench-evt-{}-{ledger}", fx.organizer_id))
            .bind(ledger)
            .execute(&fx.pool)
            .await
            .unwrap();
        }
    });

    c.bench_function("checkpoint_rollback_to_100_ledger_window", |b| {
        b.to_async(rt).iter(|| {
            let pool = fx.pool.clone();
            let chain_key = fx.chain_key.clone();
            async move {
                black_box(
                    CheckpointStore::rollback_to(&pool, &chain_key, 999_900)
                        .await
                        .unwrap(),
                );
            }
        })
    });

    rt.block_on(async {
        let _ = sqlx::query("DELETE FROM indexer_ledger_events WHERE contract_id = 'bench-contract'")
            .execute(&fx.pool)
            .await;
    });
}

fn bench_ticket_pagination_query(c: &mut Criterion, rt: &tokio::runtime::Runtime, fx: &BenchFixture) {
    let mut group = c.benchmark_group("ticket_list_pagination_by_page_size");
    for limit in [10i64, 20, 50, 100] {
        group.bench_with_input(
            criterion::BenchmarkId::from_parameter(limit),
            &limit,
            |b, &limit| {
                b.to_async(rt).iter(|| {
                    let pool = fx.pool.clone();
                    let event_id = fx.event_id;
                    async move {
                        let rows = sqlx::query_as::<_, (Uuid, String)>(
                            "SELECT id, status FROM tickets
                             WHERE event_id = $1
                             ORDER BY created_at ASC, id ASC
                             LIMIT $2",
                        )
                        .bind(event_id)
                        .bind(black_box(limit))
                        .fetch_all(&pool)
                        .await
                        .unwrap();
                        black_box(rows.len());
                    }
                })
            },
        );
    }
    group.finish();
}

fn main() {
    let Ok(database_url) = std::env::var("DATABASE_URL") else {
        eprintln!(
            "db_query_bench: DATABASE_URL not set, skipping (0 benchmarks registered). \
             Set DATABASE_URL to a migrated Postgres instance to run these benchmarks."
        );
        return;
    };

    let rt = tokio::runtime::Runtime::new().expect("tokio runtime");
    let pool = rt
        .block_on(PgPool::connect(&database_url))
        .expect("connect to DATABASE_URL");
    let fixture = rt.block_on(BenchFixture::setup(pool));

    let mut criterion = Criterion::default().configure_from_args();
    bench_checkpoint_save_load_roundtrip(&mut criterion, &rt, &fixture);
    bench_checkpoint_rollback(&mut criterion, &rt, &fixture);
    bench_ticket_pagination_query(&mut criterion, &rt, &fixture);

    rt.block_on(fixture.teardown());
    criterion.final_summary();
}
