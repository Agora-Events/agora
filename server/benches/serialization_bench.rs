//! # Payload Serialization Benchmarks (Issue #1178)
//!
//! Measures the cost of the JSON/XDR encode-decode paths that sit on the hot
//! path of every request: WebSocket telemetry frames, cursor-paginated list
//! responses, and Soroban contract-event decoding in the re-org resilient
//! indexer (`agora_server::services::indexer`, `agora_server::models::indexer_event`).
//!
//! None of these benchmarks touch the network or the database — they isolate
//! CPU-bound (de)serialization cost so a regression here can't be masked by
//! I/O noise. Run with:
//!
//! ```bash
//! cargo bench --bench serialization_bench
//! ```

use agora_server::handlers::ws::{
    GateHeatmapEvent, GateMetrics, OrganizerEvent, PurchaseEvent, ScanVelocityEvent,
};
use agora_server::models::blockchain_checkpoint::BlockchainCheckpoint;
use agora_server::models::indexer_event::{scval_to_json, IndexedEvent};
use agora_server::utils::cursor_pagination::{encode_cursor, decode_cursor, CursorResponse, EventCursor, ValidatedCursorParams};
use chrono::Utc;
use criterion::{black_box, criterion_group, criterion_main, BenchmarkId, Criterion, Throughput};
use serde_json::json;
use stellar_xdr::curr::{Limits, ReadXdr, ScMap, ScMapEntry, ScString, ScSymbol, ScVal, WriteXdr};
use uuid::Uuid;

fn sym(name: &str) -> ScSymbol {
    ScSymbol(name.parse().unwrap())
}

fn str_val(s: &str) -> ScVal {
    ScVal::String(ScString(s.parse().unwrap()))
}

// ---------------------------------------------------------------------------
// WebSocket telemetry payloads
// ---------------------------------------------------------------------------

fn sample_purchase_event() -> PurchaseEvent {
    PurchaseEvent {
        event_id: Uuid::new_v4(),
        ticket_tier_id: Uuid::new_v4(),
        quantity: 2,
        amount: 149.99,
        currency: "USDC".to_string(),
        purchased_at: Utc::now().to_rfc3339(),
    }
}

fn sample_gate_heatmap(gate_count: usize) -> OrganizerEvent {
    let gates = (0..gate_count)
        .map(|i| GateMetrics {
            gate_id: format!("gate-{i}"),
            gate_name: format!("North Entrance {i}"),
            current_wait_time_minutes: 4.5,
            throughput_per_minute: 62.0,
            staff_count: 3,
            congestion_level: "medium".to_string(),
        })
        .collect();

    OrganizerEvent::GateHeatmap(GateHeatmapEvent {
        event_id: Uuid::new_v4(),
        gates,
        timestamp: Utc::now().to_rfc3339(),
    })
}

fn bench_purchase_event_roundtrip(c: &mut Criterion) {
    let event = sample_purchase_event();
    let json = serde_json::to_string(&event).unwrap();

    let mut group = c.benchmark_group("ws_purchase_event");
    group.throughput(Throughput::Bytes(json.len() as u64));
    group.bench_function("serialize", |b| {
        b.iter(|| serde_json::to_string(black_box(&event)).unwrap())
    });
    group.bench_function("deserialize", |b| {
        b.iter(|| serde_json::from_str::<PurchaseEvent>(black_box(&json)).unwrap())
    });
    group.finish();
}

fn bench_organizer_heatmap_by_gate_count(c: &mut Criterion) {
    let mut group = c.benchmark_group("ws_organizer_gate_heatmap");
    for gate_count in [1usize, 10, 50, 200] {
        let event = sample_gate_heatmap(gate_count);
        let json = serde_json::to_string(&event).unwrap();
        group.throughput(Throughput::Elements(gate_count as u64));
        group.bench_with_input(
            BenchmarkId::new("serialize", gate_count),
            &event,
            |b, event| b.iter(|| serde_json::to_string(black_box(event)).unwrap()),
        );
        group.bench_with_input(
            BenchmarkId::new("deserialize", gate_count),
            &json,
            |b, json| b.iter(|| serde_json::from_str::<OrganizerEvent>(black_box(json)).unwrap()),
        );
    }
    group.finish();
}

fn bench_scan_velocity_event(c: &mut Criterion) {
    let event = ScanVelocityEvent {
        event_id: Uuid::new_v4(),
        gate_id: "gate-3".to_string(),
        scans_per_minute: 87.2,
        total_scans: 4211,
        timestamp: Utc::now().to_rfc3339(),
    };
    c.bench_function("ws_scan_velocity_event_serialize", |b| {
        b.iter(|| serde_json::to_string(black_box(&event)).unwrap())
    });
}

// ---------------------------------------------------------------------------
// Cursor pagination
// ---------------------------------------------------------------------------

fn bench_cursor_encode_decode(c: &mut Criterion) {
    let cursor = EventCursor {
        start_time: Utc::now(),
        id: Uuid::new_v4(),
        created_at: Some(Utc::now()),
        minted_tickets: Some(4200),
        count_of_ratings: Some(318),
        min_ticket_price: Some(49.5),
    };
    let encoded = encode_cursor(&cursor).unwrap();

    let mut group = c.benchmark_group("cursor_pagination");
    group.bench_function("encode", |b| {
        b.iter(|| encode_cursor(black_box(&cursor)).unwrap())
    });
    group.bench_function("decode", |b| {
        b.iter(|| decode_cursor::<EventCursor>(black_box(&encoded)).unwrap())
    });
    group.finish();
}

fn bench_cursor_response_wrap_by_page_size(c: &mut Criterion) {
    let mut group = c.benchmark_group("cursor_response_wrap");
    for page_size in [10u32, 20, 50, 100] {
        let params = ValidatedCursorParams {
            limit: page_size,
            cursor: None,
            include_count: true,
        };
        let items: Vec<Uuid> = (0..page_size).map(|_| Uuid::new_v4()).collect();
        group.throughput(Throughput::Elements(page_size as u64));
        group.bench_with_input(
            BenchmarkId::from_parameter(page_size),
            &(items, params),
            |b, (items, params)| {
                b.iter(|| {
                    let response =
                        CursorResponse::new(black_box(items.clone()), params, Some("next".into()))
                            .with_total(10_000, true);
                    serde_json::to_string(&response).unwrap()
                })
            },
        );
    }
    group.finish();
}

// ---------------------------------------------------------------------------
// Soroban XDR event decoding (indexer hot path)
// ---------------------------------------------------------------------------

fn purchase_processed_xdr(fields: usize) -> String {
    let mut entries = vec![
        ScMapEntry {
            key: ScVal::Symbol(sym("payment_id")),
            val: str_val("pay-bench-0001"),
        },
        ScMapEntry {
            key: ScVal::Symbol(sym("event_id")),
            val: str_val(&Uuid::new_v4().to_string()),
        },
        ScMapEntry {
            key: ScVal::Symbol(sym("buyer")),
            val: str_val("GABCDEF1234567890"),
        },
        ScMapEntry {
            key: ScVal::Symbol(sym("amount")),
            val: ScVal::I64(12_500_000),
        },
    ];
    // Pad the map with extra fields to model larger real-world event
    // payloads (metadata, memo fields, discount codes, etc.).
    for i in 0..fields {
        entries.push(ScMapEntry {
            key: ScVal::Symbol(sym(&format!("extra_{i}"))),
            val: ScVal::U32(i as u32),
        });
    }
    let scval = ScVal::Map(Some(ScMap(entries.try_into().unwrap())));
    scval.to_xdr_base64(Limits::none()).unwrap()
}

fn bench_indexed_event_decode_by_payload_size(c: &mut Criterion) {
    let topic = ScVal::Symbol(sym("PaymentProcessed"))
        .to_xdr_base64(Limits::none())
        .unwrap();

    let mut group = c.benchmark_group("indexer_event_decode");
    for extra_fields in [0usize, 8, 32, 128] {
        let xdr = purchase_processed_xdr(extra_fields);
        let value = json!({ "xdr": xdr });
        group.throughput(Throughput::Elements(1));
        group.bench_with_input(
            BenchmarkId::from_parameter(extra_fields),
            &(topic.clone(), value),
            |b, (topic, value)| {
                b.iter(|| {
                    IndexedEvent::decode(
                        black_box("evt-id-bench".to_string()),
                        black_box(1_000_000),
                        black_box("CPAYCONTRACT".to_string()),
                        black_box(std::slice::from_ref(topic)),
                        black_box(value),
                        black_box(1_000_010),
                    )
                })
            },
        );
    }
    group.finish();
}

fn bench_scval_to_json(c: &mut Criterion) {
    let xdr = purchase_processed_xdr(32);
    let scval = ScVal::from_xdr_base64(&xdr, Limits::none()).expect("encoded above");
    c.bench_function("scval_to_json_32_field_map", |b| {
        b.iter(|| scval_to_json(black_box(&scval)))
    });
}

// ---------------------------------------------------------------------------
// Checkpoint model
// ---------------------------------------------------------------------------

fn bench_checkpoint_serialize(c: &mut Criterion) {
    let cp = BlockchainCheckpoint {
        chain_key: "soroban:mainnet".to_string(),
        ledger_sequence: 68_432_991,
        event_cursor: Some("cursor-opaque-token-abc123".to_string()),
        created_at: Utc::now(),
        updated_at: Utc::now(),
    };
    c.bench_function("blockchain_checkpoint_serialize", |b| {
        b.iter(|| serde_json::to_string(black_box(&cp)).unwrap())
    });
}

criterion_group!(
    benches,
    bench_purchase_event_roundtrip,
    bench_organizer_heatmap_by_gate_count,
    bench_scan_velocity_event,
    bench_cursor_encode_decode,
    bench_cursor_response_wrap_by_page_size,
    bench_indexed_event_decode_by_payload_size,
    bench_scval_to_json,
    bench_checkpoint_serialize,
);
criterion_main!(benches);
