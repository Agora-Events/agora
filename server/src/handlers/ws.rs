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
    response::{IntoResponse, Response},
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
///
/// Requires a valid JWT in the `Authorization: Bearer <token>` header or
/// `?token=<jwt>` query parameter. Returns 401 if authentication fails.
pub async fn ws_purchases_handler(
    ws: WebSocketUpgrade,
    State(broadcaster): State<PurchaseBroadcaster>,
    axum::extract::Query(params): axum::extract::Query<std::collections::HashMap<String, String>>,
    headers: axum::http::HeaderMap,
) -> Response {
    // Extract JWT from Authorization header or ?token= query param.
    let token = headers
        .get(axum::http::header::AUTHORIZATION)
        .and_then(|v| v.to_str().ok())
        .and_then(|v| v.strip_prefix("Bearer "))
        .or_else(|| params.get("token").map(|s| s.as_str()));

    let _wallet = match token {
        Some(t) => match crate::handlers::auth::verify_jwt(t) {
            Ok(claims) => claims.sub,
            Err(e) => {
                tracing::warn!("WebSocket upgrade rejected: invalid JWT: {:?}", e);
                return (
                    axum::http::StatusCode::UNAUTHORIZED,
                    "Unauthorized: invalid or expired token",
                )
                    .into_response();
            }
        },
        None => {
            tracing::warn!("WebSocket upgrade rejected: no JWT provided");
            return (
                axum::http::StatusCode::UNAUTHORIZED,
                "Unauthorized: authentication required",
            )
                .into_response();
        }
    };

    // Pass the authenticated wallet to the socket handler for per-user scoping.
    ws.on_upgrade(move |socket| handle_socket(socket, broadcaster, _wallet))
}

async fn handle_socket(mut socket: WebSocket, broadcaster: PurchaseBroadcaster, _wallet: String) {
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
                        if socket.send(Message::Text(json)).await.is_err() {
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
