/**
 * Sync handler for CRDT delta synchronization
 * Implements POST /api/v1/sync/delta endpoint for offline-first sync
 */

use crate::crdt::{VectorClock, VectorClockUtils};
use crate::utils::error::{ApiError, AppError};
use axum::{extract::State, Json};
use serde::{Deserialize, Serialize};
use sqlx::PgPool;
use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::RwLock;

/// Delta operation type
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum DeltaOperation {
    Add,
    Remove,
    Update,
}

/// A delta entry representing a local mutation
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DeltaEntry {
    pub id: String,
    pub entity_type: String,
    pub entity_id: String,
    pub operation: DeltaOperation,
    pub value: serde_json::Value,
    pub vector_clock: HashMap<String, u64>,
    pub timestamp: i64,
    pub synced: bool,
}

/// Sync request from client
#[derive(Debug, Serialize, Deserialize)]
pub struct SyncRequest {
    pub node_id: String,
    pub vector_clock: HashMap<String, u64>,
    pub deltas: Vec<DeltaEntry>,
}

/// Sync response to client
#[derive(Debug, Serialize, Deserialize)]
pub struct SyncResponse {
    pub server_vector_clock: HashMap<String, u64>,
    pub server_deltas: Vec<DeltaEntry>,
    pub merged_vector_clock: HashMap<String, u64>,
}

/// Server-side sync state
#[derive(Clone)]
pub struct SyncState {
    pub pool: PgPool,
    pub server_vector_clock: Arc<RwLock<VectorClock>>,
    pub delta_store: Arc<RwLock<HashMap<String, Vec<DeltaEntry>>>>,
}

impl SyncState {
    pub fn new(pool: PgPool) -> Self {
        Self {
            pool,
            server_vector_clock: Arc::new(RwLock::new(VectorClock::empty())),
            delta_store: Arc::new(RwLock::new(HashMap::new())),
        }
    }
}

/// POST /api/v1/sync/delta
/// Receives local state vector clocks, computes state diffs, 
/// applies CRDT merge rules, and returns server delta patches
pub async fn delta_sync(
    State(state): State<SyncState>,
    Json(request): Json<SyncRequest>,
) -> Result<Json<SyncResponse>, ApiError> {
    // Convert client vector clock to our format
    let client_clock = VectorClock {
        counters: request.vector_clock,
    };

    // Get current server vector clock
    let server_clock = {
        let clock = state.server_vector_clock.read().await;
        clock.clone()
    };

    // Compute state diff - find server deltas that client hasn't seen
    let server_deltas = compute_server_deltas(&state, &client_clock).await?;

    // Merge client vector clock with server vector clock
    let merged_clock = server_clock.merge(&client_clock);

    // Apply client deltas to server state
    apply_client_deltas(&state, &request.deltas).await?;

    // Update server vector clock with merged clock
    {
        let mut clock = state.server_vector_clock.write().await;
        *clock = merged_clock.clone();
    }

    // Store client deltas for future syncs
    store_client_deltas(&state, &request.node_id, &request.deltas).await;

    // Convert merged clock back to HashMap for response
    let merged_clock_map = merged_clock.counters;
    let server_clock_map = server_clock.counters;

    Ok(Json(SyncResponse {
        server_vector_clock: server_clock_map,
        server_deltas,
        merged_vector_clock: merged_clock_map,
    }))
}

/// Compute server deltas that the client hasn't seen yet
async fn compute_server_deltas(
    state: &SyncState,
    client_clock: &VectorClock,
) -> Result<Vec<DeltaEntry>, ApiError> {
    let delta_store = state.delta_store.read().await;
    let mut server_deltas = Vec::new();

    for (node_id, deltas) in delta_store.iter() {
        for delta in deltas {
            // Convert delta vector clock to our format
            let delta_clock = VectorClock {
                counters: delta.vector_clock.clone(),
            };

            // Check if this delta happened after the client's clock
            if delta_clock.happened_before(client_clock) || 
               delta_clock.is_concurrent(client_clock) {
                server_deltas.push(delta.clone());
            }
        }
    }

    Ok(server_deltas)
}

/// Apply client deltas to server state
async fn apply_client_deltas(
    state: &SyncState,
    deltas: &[DeltaEntry],
) -> Result<(), ApiError> {
    // In a real implementation, this would:
    // 1. Apply deltas to the appropriate CRDT stores
    // 2. Persist changes to the database
    // 3. Update server state based on the operations

    for delta in deltas {
        match delta.operation {
            DeltaOperation::Add => {
                // Apply add operation to appropriate entity
                apply_add_operation(state, delta).await?;
            }
            DeltaOperation::Remove => {
                // Apply remove operation to appropriate entity
                apply_remove_operation(state, delta).await?;
            }
            DeltaOperation::Update => {
                // Apply update operation to appropriate entity
                apply_update_operation(state, delta).await?;
            }
        }
    }

    Ok(())
}

/// Apply an add operation to the database
async fn apply_add_operation(
    _state: &SyncState,
    delta: &DeltaEntry,
) -> Result<(), ApiError> {
    // This is a placeholder - in a real implementation, you would:
    // 1. Parse the entity type and determine the table
    // 2. Insert or update the record in the database
    // 3. Handle conflicts using CRDT merge rules
    
    tracing::info!(
        "Applying add operation for entity_type={}, entity_id={}",
        delta.entity_type,
        delta.entity_id
    );

    // Example: For bookmark operations
    if delta.entity_type == "bookmark" {
        // Insert bookmark into database
        // sqlx::query!(
        //     "INSERT INTO bookmarks (user_id, event_id, vector_clock) VALUES ($1, $2, $3)",
        //     delta.entity_id,
        //     parse_event_id(&delta.value)?,
        //     serde_json::to_value(&delta.vector_clock)?
        // )
        // .execute(&state.pool)
        // .await?;
    }

    Ok(())
}

/// Apply a remove operation to the database
async fn apply_remove_operation(
    _state: &SyncState,
    delta: &DeltaEntry,
) -> Result<(), ApiError> {
    // This is a placeholder - in a real implementation, you would:
    // 1. Parse the entity type and determine the table
    // 2. Soft delete or mark as removed in the database
    // 3. Handle conflicts using CRDT merge rules
    
    tracing::info!(
        "Applying remove operation for entity_type={}, entity_id={}",
        delta.entity_type,
        delta.entity_id
    );

    Ok(())
}

/// Apply an update operation to the database
async fn apply_update_operation(
    _state: &SyncState,
    delta: &DeltaEntry,
) -> Result<(), ApiError> {
    // This is a placeholder - in a real implementation, you would:
    // 1. Parse the entity type and determine the table
    // 2. Update the record in the database
    // 3. Handle conflicts using CRDT merge rules (last-write-wins)
    
    tracing::info!(
        "Applying update operation for entity_type={}, entity_id={}",
        delta.entity_type,
        delta.entity_id
    );

    Ok(())
}

/// Store client deltas for future syncs
async fn store_client_deltas(
    state: &SyncState,
    node_id: &str,
    deltas: &[DeltaEntry],
) {
    let mut delta_store = state.delta_store.write().await;
    
    let node_deltas = delta_store.entry(node_id.to_string()).or_insert_with(Vec::new);
    
    // Add new deltas, avoiding duplicates
    for delta in deltas {
        if !node_deltas.iter().any(|d| d.id == delta.id) {
            node_deltas.push(delta.clone());
        }
    }

    // Prune old deltas to prevent unbounded growth
    if node_deltas.len() > 1000 {
        node_deltas.sort_by(|a, b| a.timestamp.cmp(&b.timestamp));
        node_deltas.drain(0..node_deltas.len() - 1000);
    }
}

/// GET /api/v1/sync/status
/// Get current sync status for a node
pub async fn sync_status(
    State(state): State<SyncState>,
    axum::extract::Path(node_id): axum::extract::Path<String>,
) -> Result<Json<serde_json::Value>, ApiError> {
    let server_clock = {
        let clock = state.server_vector_clock.read().await;
        clock.clone()
    };

    let delta_store = state.delta_store.read().await;
    let node_deltas = delta_store.get(&node_id).map(|d| d.len()).unwrap_or(0);

    Ok(Json(serde_json::json!({
        "node_id": node_id,
        "server_vector_clock": server_clock.counters,
        "pending_deltas": node_deltas,
    })))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_delta_operation_serialization() {
        let op = DeltaOperation::Add;
        let json = serde_json::to_string(&op).unwrap();
        assert_eq!(json, "\"add\"");
    }

    #[test]
    fn test_delta_entry_serialization() {
        let entry = DeltaEntry {
            id: "test-id".to_string(),
            entity_type: "bookmark".to_string(),
            entity_id: "entity-123".to_string(),
            operation: DeltaOperation::Add,
            value: serde_json::json!({"event_id": "event-456"}),
            vector_clock: HashMap::from([("node1".to_string(), 1)]),
            timestamp: 1234567890,
            synced: false,
        };
        
        let json = serde_json::to_string(&entry).unwrap();
        let parsed: DeltaEntry = serde_json::from_str(&json).unwrap();
        
        assert_eq!(parsed.id, entry.id);
        assert_eq!(parsed.entity_type, entry.entity_type);
    }
}
