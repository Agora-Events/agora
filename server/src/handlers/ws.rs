//! WebSocket handler for real-time ticket purchase updates.
//!
//! Dashboard clients connect to `GET /api/v1/ws/purchases` and receive
//! [`PurchaseEvent`] messages as JSON whenever a ticket sale is processed.
//!
//! # Architecture
//!
//! A single [`PurchaseBroadcaster`] (held in shared app state) owns a
//! `tokio::sync::broadcast` channel. The ticket purchase handler calls
//! [`PurchaseBroadcaster::publish`] after a successful sale; every connected
//! WebSocket client receives the message independently.

use axum::{
    extract::{
        ws::{Message, WebSocket, WebSocketUpgrade},
        State,
    },
    response::Response,
};
use serde::{Deserialize, Serialize};
use std::sync::Arc;
use tokio::sync::broadcast;
use uuid::Uuid;

/// The capacity of the broadcast channel (number of buffered messages).
const CHANNEL_CAPACITY: usize = 128;

/// A live ticket purchase event pushed to dashboard WebSocket clients.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PurchaseEvent {
    pub event_id: Uuid,
    pub ticket_tier_id: Uuid,
    pub quantity: u32,
    /// Total sale amount in the event's currency.
    pub amount: f64,
    pub currency: String,
    /// ISO-8601 timestamp of the purchase.
    pub purchased_at: String,
}

/// Shared broadcaster — create once at startup and clone the `Arc` into state.
#[derive(Clone)]
pub struct PurchaseBroadcaster {
    sender: Arc<broadcast::Sender<PurchaseEvent>>,
}

impl PurchaseBroadcaster {
    pub fn new() -> Self {
        let (sender, _) = broadcast::channel(CHANNEL_CAPACITY);
        Self {
            sender: Arc::new(sender),
        }
    }

    /// Publish a purchase event to all connected clients.
    /// Returns the number of active receivers (0 if no clients are connected).
    pub fn publish(&self, event: PurchaseEvent) -> usize {
        self.sender.send(event).unwrap_or(0)
    }

    /// Subscribe to the broadcast stream.
    pub fn subscribe(&self) -> broadcast::Receiver<PurchaseEvent> {
        self.sender.subscribe()
    }
}

impl Default for PurchaseBroadcaster {
    fn default() -> Self {
        Self::new()
    }
}

/// HTTP upgrade handler — clients call this endpoint to open a WebSocket.
///
/// Route: `GET /api/v1/ws/purchases`
pub async fn ws_purchases_handler(
    ws: WebSocketUpgrade,
    State(broadcaster): State<PurchaseBroadcaster>,
) -> Response {
    ws.on_upgrade(move |socket| handle_socket(socket, broadcaster))
}

async fn handle_socket(mut socket: WebSocket, broadcaster: PurchaseBroadcaster) {
    let mut rx = broadcaster.subscribe();

    loop {
        tokio::select! {
            // Forward broadcast messages to the WebSocket client.
            result = rx.recv() => {
                match result {
                    Ok(event) => {
                        let json = match serde_json::to_string(&event) {
                            Ok(j) => j,
                            Err(e) => {
                                tracing::error!(error = %e, "Failed to serialize PurchaseEvent");
                                continue;
                            }
                        };
                        if socket.send(Message::Text(json.into())).await.is_err() {
                            // Client disconnected.
                            break;
                        }
                    }
                    Err(broadcast::error::RecvError::Lagged(n)) => {
                        tracing::warn!(skipped = n, "WebSocket client lagged behind broadcast");
                    }
                    Err(broadcast::error::RecvError::Closed) => break,
                }
            }
            // Handle incoming client messages (ping/close frames).
            msg = socket.recv() => {
                match msg {
                    Some(Ok(Message::Close(_))) | None => break,
                    Some(Ok(Message::Ping(payload))) => {
                        let _ = socket.send(Message::Pong(payload)).await;
                    }
                    _ => {}
                }
            }
        }
    }
}
