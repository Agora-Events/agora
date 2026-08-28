//! # Chaos Test: Dropped WebSocket Frames (Issue #1178)
//!
//! Exercises the real broadcaster types dashboard clients connect to
//! (`agora_server::handlers::ws::{PurchaseBroadcaster, OrganizerBroadcaster}`)
//! under simulated frame loss, using [`common::chaos::drop_frames`] to model
//! a lossy network dropping WebSocket frames in transit.
//!
//! No database or network is required — these tests exercise the in-process
//! `tokio::sync::broadcast` channels directly.

mod common;

use agora_server::handlers::ws::{
    GateHeatmapEvent, OrganizerBroadcaster, OrganizerEvent, PurchaseBroadcaster, PurchaseEvent,
};
use common::chaos::drop_frames;
use chrono::Utc;
use tokio::sync::broadcast::error::RecvError;
use uuid::Uuid;

fn sample_purchase(i: u32) -> PurchaseEvent {
    PurchaseEvent {
        event_id: Uuid::new_v4(),
        ticket_tier_id: Uuid::new_v4(),
        quantity: 1,
        amount: 10.0 + i as f64,
        currency: "USDC".to_string(),
        purchased_at: Utc::now().to_rfc3339(),
    }
}

/// The broadcaster's real client-facing lag handling (mirrored from
/// `handle_socket` in `handlers::ws`): a slow/absent reader must observe
/// `RecvError::Lagged(n)`, never a silent, undetectable gap in the stream.
#[tokio::test]
async fn slow_consumer_observes_lagged_not_silent_loss() {
    let broadcaster = PurchaseBroadcaster::new();
    let mut rx = broadcaster.subscribe();

    // Publish well beyond the broadcaster's internal channel capacity (128)
    // without ever reading, forcing the receiver to fall behind.
    for i in 0..400u32 {
        broadcaster.publish(sample_purchase(i));
    }

    match rx.recv().await {
        Err(RecvError::Lagged(skipped)) => {
            assert!(skipped > 0, "lag must report a nonzero skipped count");
        }
        other => panic!(
            "expected the broadcast channel to report Lagged after being overrun, got {other:?}"
        ),
    }
}

/// A reader that keeps up (reads as fast as frames are published) must never
/// observe drops purely from broadcaster overflow — loss should only come
/// from the chaos-injected network simulation, not from the broadcaster
/// itself under normal load.
#[tokio::test]
async fn keeping_up_consumer_sees_every_frame_broadcaster_side() {
    let broadcaster = PurchaseBroadcaster::new();
    let mut rx = broadcaster.subscribe();

    let publisher = {
        let broadcaster = broadcaster.clone();
        tokio::spawn(async move {
            for i in 0..50u32 {
                broadcaster.publish(sample_purchase(i));
                tokio::task::yield_now().await;
            }
        })
    };

    let mut received = 0u32;
    while received < 50 {
        match rx.recv().await {
            Ok(_) => received += 1,
            Err(RecvError::Lagged(n)) => panic!("unexpected lag of {n} while keeping up"),
            Err(RecvError::Closed) => break,
        }
    }
    publisher.await.unwrap();
    assert_eq!(received, 50);
}

/// [`drop_frames`] simulates a lossy socket on top of a broadcaster that is
/// itself delivering every frame — the majority of frames must still arrive,
/// and the consumer must complete instead of hanging when the sender side
/// closes.
#[tokio::test]
async fn dropped_frames_still_deliver_the_majority_of_purchase_events() {
    let broadcaster = PurchaseBroadcaster::new();
    let rx = broadcaster.subscribe();
    let mut lossy_rx = drop_frames(rx, 0.3);

    const TOTAL: u32 = 500;
    for i in 0..TOTAL {
        broadcaster.publish(sample_purchase(i));
    }
    drop(broadcaster); // close the channel so the forwarder task can exit

    let mut received = 0u32;
    while lossy_rx.recv().await.is_some() {
        received += 1;
    }

    assert!(received > 0, "a 30% drop rate must not eliminate every frame");
    assert!(
        received < TOTAL,
        "a 30% drop rate over {TOTAL} frames should statistically drop at least one"
    );
    // Expected ~350 survivors; a wide band keeps this non-flaky.
    assert!(
        received > (TOTAL as f64 * 0.5) as u32,
        "received {received}/{TOTAL} — drop rate behaved far outside its configured 30%"
    );
}

#[tokio::test]
async fn organizer_broadcaster_tolerates_dropped_gate_heatmap_frames() {
    let broadcaster = OrganizerBroadcaster::new();
    let rx = broadcaster.subscribe();
    let mut lossy_rx = drop_frames(rx, 0.25);

    const TOTAL: u32 = 300;
    for i in 0..TOTAL {
        broadcaster.publish(OrganizerEvent::GateHeatmap(GateHeatmapEvent {
            event_id: Uuid::new_v4(),
            gates: vec![],
            timestamp: format!("frame-{i}"),
        }));
    }
    drop(broadcaster);

    let mut received = 0u32;
    while lossy_rx.recv().await.is_some() {
        received += 1;
    }
    assert!(received > (TOTAL as f64 * 0.5) as u32, "received {received}/{TOTAL}");
}

/// Zero drop rate is the harness's own regression guard: with no chaos
/// applied, frame delivery must be exact.
#[tokio::test]
async fn zero_drop_rate_delivers_every_frame_exactly() {
    let broadcaster = PurchaseBroadcaster::new();
    let rx = broadcaster.subscribe();
    let mut lossy_rx = drop_frames(rx, 0.0);

    for i in 0..40u32 {
        broadcaster.publish(sample_purchase(i));
    }
    drop(broadcaster);

    let mut received = 0u32;
    while lossy_rx.recv().await.is_some() {
        received += 1;
    }
    assert_eq!(received, 40);
}
