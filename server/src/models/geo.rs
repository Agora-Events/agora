//! # Geo / Spatial models (Issue: Event Discovery Engine)
//!
//! Structs used by [`crate::handlers::geo`] for the `/api/v1/events/nearby`
//! endpoint and the background geofence worker.
//!
//! The database uses plain `latitude` / `longitude` DOUBLE PRECISION columns
//! (added in migration `20260729000001_add_event_coordinates.sql`) with
//! Haversine distance computed in SQL — portable with no PostGIS requirement.
//! A drop-in upgrade to `ST_DWithin` / `ST_Contains` is straightforward.

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use sqlx::FromRow;
use uuid::Uuid;

// ---------------------------------------------------------------------------
// Query parameters
// ---------------------------------------------------------------------------

/// Query-string parameters accepted by `GET /api/v1/events/nearby`.
///
/// One of `radius_m` **or** all four `bbox_*` params must be present.
/// If both are absent the handler falls back to 5 000 m.
#[derive(Debug, Deserialize)]
pub struct NearbyQuery {
    /// Observer latitude (WGS-84, decimal degrees).
    pub lat: f64,
    /// Observer longitude (WGS-84, decimal degrees).
    pub lng: f64,
    /// Search radius in metres (circle query; mutually exclusive with bbox).
    pub radius_m: Option<f64>,
    /// Bounding-box south-west latitude.
    pub bbox_sw_lat: Option<f64>,
    /// Bounding-box south-west longitude.
    pub bbox_sw_lng: Option<f64>,
    /// Bounding-box north-east latitude.
    pub bbox_ne_lat: Option<f64>,
    /// Bounding-box north-east longitude.
    pub bbox_ne_lng: Option<f64>,
    /// Maximum results (default 50, max 200).
    pub limit: Option<i64>,
}

// ---------------------------------------------------------------------------
// Result row
// ---------------------------------------------------------------------------

/// One event returned by the nearby query, including Haversine distance.
#[derive(Debug, Serialize, FromRow)]
pub struct NearbyEvent {
    pub id: Uuid,
    pub title: String,
    pub start_date: DateTime<Utc>,
    pub end_date: Option<DateTime<Utc>>,
    pub latitude: f64,
    pub longitude: f64,
    pub venue: Option<String>,
    pub image_url: Option<String>,
    /// Great-circle distance from the query point to the venue, in metres.
    pub distance_m: f64,
}

// ---------------------------------------------------------------------------
// Geofence registration (client → server)
// ---------------------------------------------------------------------------

/// Request body for `POST /api/v1/geo/geofences`.
#[derive(Debug, Deserialize)]
pub struct GeofenceRegistration {
    pub event_id: Uuid,
    /// Expo push token (`ExponentPushToken[…]`).
    pub push_token: String,
    pub venue_lat: f64,
    pub venue_lng: f64,
}

// ---------------------------------------------------------------------------
// Database row for the geofence worker
// ---------------------------------------------------------------------------

/// Row loaded by the background worker to check proximity.
#[derive(Debug, FromRow)]
pub struct GeofenceRow {
    pub id: Uuid,
    pub wallet_address: String,
    pub push_token: String,
    pub event_id: Uuid,
    pub venue_lat: f64,
    pub venue_lng: f64,
    pub notified: bool,
}
