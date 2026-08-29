//! # Geo / Spatial Handlers (Issue: Event Discovery Engine)
//!
//! ## Endpoints
//! - `GET  /api/v1/events/nearby`  — radius or bounding-box query
//! - `POST /api/v1/geo/geofences`  — register a device geofence
//! - `run_geofence_worker`         — background task: dispatches proximity push notifications
//!
//! ### Spatial Strategy
//! Uses plain `latitude`/`longitude` DOUBLE PRECISION columns (already present
//! in the schema from migration `20260729000001_add_event_coordinates.sql`).
//! Nearby queries run the Haversine formula in SQL, pre-filtered by a
//! bounding-box that the partial index `idx_events_coordinates` can accelerate.
//! No PostGIS extension required; upgrade path is a one-line `WHERE` swap.
//!
//! ### Push Notification Gateway
//! The background worker polls `geo_geofences` every 30 s. For each
//! un-notified registration whose event starts within 24 h it checks
//! `user_locations` (the device's last-reported coordinates) and — when the
//! user is within 150 m — dispatches an Expo push notification via
//! [`crate::notifications::NotificationService`], then marks `notified = true`.

use axum::{
    extract::{Query, State},
    http::HeaderMap,
    response::{IntoResponse, Response},
    Json,
};
use chrono::Utc;
use serde::Serialize;
use sqlx::PgPool;
use std::sync::Arc;
use tokio::time::{interval, Duration};
use tokio_util::sync::CancellationToken;
use uuid::Uuid;

use crate::handlers::auth::extract_auth;
use crate::models::geo::{GeofenceRegistration, GeofenceRow, NearbyEvent, NearbyQuery};
use crate::notifications::{Notification, NotificationService};
use crate::utils::error::AppError;
use crate::utils::response::success;

// ---------------------------------------------------------------------------
// Shared state
// ---------------------------------------------------------------------------

#[derive(Clone)]
pub struct GeoState {
    pub pool: PgPool,
    pub notifications: Arc<NotificationService>,
}

// ---------------------------------------------------------------------------
// Constants
// ---------------------------------------------------------------------------

const DEFAULT_RADIUS_M: f64 = 5_000.0;
const MAX_RADIUS_M: f64 = 500_000.0; // 500 km — matches models::geo::MAX_RADIUS_M (Issue #1259)
const DEFAULT_LIMIT: i64 = 50;
const MAX_LIMIT: i64 = 200;
/// Geofence alert perimeter — mirrors the mobile 150 m perimeter.
const GEOFENCE_RADIUS_M: f64 = 150.0;
const WORKER_INTERVAL_SECS: u64 = 30;

// ---------------------------------------------------------------------------
// Response DTOs
// ---------------------------------------------------------------------------

#[derive(Serialize)]
struct NearbyResponse {
    events: Vec<NearbyEvent>,
    count: usize,
}

#[derive(Serialize)]
struct GeofenceResponse {
    geofence_id: Uuid,
    message: &'static str,
}

// ---------------------------------------------------------------------------
// GET /api/v1/events/nearby
// ---------------------------------------------------------------------------

/// Returns upcoming events within a radius or bounding box of the caller's
/// position, ordered by ascending distance.
pub async fn get_nearby_events(
    State(state): State<GeoState>,
    Query(q): Query<NearbyQuery>,
) -> Result<Response, AppError> {
    // Reject malformed / out-of-range coordinates and radii at the edge (Issue #1259)
    // so we never surface a database constraint violation as a 500.
    q.validate()?;

    let limit = q.limit.unwrap_or(DEFAULT_LIMIT).clamp(1, MAX_LIMIT);

    let has_bbox = [&q.bbox_sw_lat, &q.bbox_sw_lng, &q.bbox_ne_lat, &q.bbox_ne_lng]
        .iter()
        .any(|v| v.is_some());
    let has_radius = q.radius_m.is_some();

    if has_bbox && has_radius {
        return Err(AppError::ValidationError(
            "Supply either radius_m or bbox_* parameters, not both".into(),
        ));
    }

    let events: Vec<NearbyEvent> = if has_bbox {
        // ---- Bounding-box query -------------------------------------------
        let sw_lat = q
            .bbox_sw_lat
            .ok_or_else(|| AppError::ValidationError("bbox_sw_lat required".into()))?;
        let sw_lng = q
            .bbox_sw_lng
            .ok_or_else(|| AppError::ValidationError("bbox_sw_lng required".into()))?;
        let ne_lat = q
            .bbox_ne_lat
            .ok_or_else(|| AppError::ValidationError("bbox_ne_lat required".into()))?;
        let ne_lng = q
            .bbox_ne_lng
            .ok_or_else(|| AppError::ValidationError("bbox_ne_lng required".into()))?;

        sqlx::query_as::<_, NearbyEvent>(
            r#"
            SELECT
                e.id,
                e.title,
                e.start_date,
                e.end_date,
                e.latitude,
                e.longitude,
                e.venue,
                e.image_url,
                6371000.0 * 2.0 * ASIN(SQRT(
                    POWER(SIN(RADIANS(e.latitude  - $1) / 2.0), 2)
                  + COS(RADIANS($1)) * COS(RADIANS(e.latitude))
                    * POWER(SIN(RADIANS(e.longitude - $2) / 2.0), 2)
                )) AS distance_m
            FROM events e
            WHERE
                e.latitude  IS NOT NULL
                AND e.longitude IS NOT NULL
                AND e.latitude  BETWEEN $3 AND $5
                AND e.longitude BETWEEN $4 AND $6
                AND e.start_date >= NOW()
            ORDER BY distance_m ASC
            LIMIT $7
            "#,
        )
        .bind(q.lat)
        .bind(q.lng)
        .bind(sw_lat)
        .bind(sw_lng)
        .bind(ne_lat)
        .bind(ne_lng)
        .bind(limit)
        .fetch_all(&state.pool)
        .await
        .map_err(AppError::from)?
    } else {
        // ---- Radius (circle) query ----------------------------------------
        let radius_m = q
            .radius_m
            .unwrap_or(DEFAULT_RADIUS_M)
            .clamp(1.0, MAX_RADIUS_M);

        // Bounding-box pre-filter to exploit `idx_events_coordinates`.
        // 1 degree latitude ≈ 111 319 m.
        let lat_delta = radius_m / 111_319.0;
        let lng_delta =
            radius_m / (111_319.0 * q.lat.to_radians().cos().abs().max(1e-4));

        sqlx::query_as::<_, NearbyEvent>(
            r#"
            SELECT
                e.id,
                e.title,
                e.start_date,
                e.end_date,
                e.latitude,
                e.longitude,
                e.venue,
                e.image_url,
                6371000.0 * 2.0 * ASIN(SQRT(
                    POWER(SIN(RADIANS(e.latitude  - $1) / 2.0), 2)
                  + COS(RADIANS($1)) * COS(RADIANS(e.latitude))
                    * POWER(SIN(RADIANS(e.longitude - $2) / 2.0), 2)
                )) AS distance_m
            FROM events e
            WHERE
                e.latitude  IS NOT NULL
                AND e.longitude IS NOT NULL
                AND e.latitude  BETWEEN $3 AND $4
                AND e.longitude BETWEEN $5 AND $6
                AND 6371000.0 * 2.0 * ASIN(SQRT(
                        POWER(SIN(RADIANS(e.latitude  - $1) / 2.0), 2)
                      + COS(RADIANS($1)) * COS(RADIANS(e.latitude))
                        * POWER(SIN(RADIANS(e.longitude - $2) / 2.0), 2)
                    )) <= $7
                AND e.start_date >= NOW()
            ORDER BY distance_m ASC
            LIMIT $8
            "#,
        )
        .bind(q.lat)
        .bind(q.lng)
        .bind(q.lat - lat_delta)
        .bind(q.lat + lat_delta)
        .bind(q.lng - lng_delta)
        .bind(q.lng + lng_delta)
        .bind(radius_m)
        .bind(limit)
        .fetch_all(&state.pool)
        .await
        .map_err(AppError::from)?
    };

    let count = events.len();
    Ok(success(NearbyResponse { events, count }, "Nearby events retrieved").into_response())
}

// ---------------------------------------------------------------------------
// POST /api/v1/geo/geofences
// ---------------------------------------------------------------------------

/// Register a device geofence for a purchased ticket's venue.
///
/// Idempotent — re-registering the same `(push_token, event_id)` pair updates
/// venue coordinates and resets `notified` to false.
pub async fn register_geofence(
    State(state): State<GeoState>,
    headers: HeaderMap,
    Json(payload): Json<GeofenceRegistration>,
) -> Result<Response, AppError> {
    let wallet = extract_auth(&headers)?;

    // Reject malformed / out-of-range venue coordinates at the edge (Issue #1259).
    payload.validate()?;

    if payload.push_token.trim().is_empty() {
        return Err(AppError::ValidationError("push_token cannot be empty".into()));
    }

    let geofence_id = sqlx::query_scalar::<_, Uuid>(
        r#"
        INSERT INTO geo_geofences
            (wallet_address, push_token, event_id, venue_lat, venue_lng, notified)
        VALUES ($1, $2, $3, $4, $5, false)
        ON CONFLICT (push_token, event_id) DO UPDATE
            SET venue_lat  = EXCLUDED.venue_lat,
                venue_lng  = EXCLUDED.venue_lng,
                notified   = false,
                updated_at = NOW()
        RETURNING id
        "#,
    )
    .bind(&wallet)
    .bind(payload.push_token.trim())
    .bind(payload.event_id)
    .bind(payload.venue_lat)
    .bind(payload.venue_lng)
    .fetch_one(&state.pool)
    .await
    .map_err(AppError::from)?;

    Ok(success(
        GeofenceResponse {
            geofence_id,
            message: "Geofence registered",
        },
        "Geofence registered successfully",
    )
    .into_response())
}

// ---------------------------------------------------------------------------
// Background geofence worker
// ---------------------------------------------------------------------------

/// Long-running task: polls `geo_geofences` every 30 s and dispatches Expo
/// push notifications when a user is within 150 m of their ticket's venue.
///
/// Spawn once at server startup:
/// ```rust
/// tokio::spawn(run_geofence_worker(pool.clone(), notifications.clone(), shutdown.clone()));
/// ```
///
/// `shutdown` is a [`tokio_util::sync::CancellationToken`] used for graceful
/// shutdown (Issue #1261); the worker exits cleanly when it is cancelled.
pub async fn run_geofence_worker(
    pool: PgPool,
    notifications: Arc<NotificationService>,
    shutdown: CancellationToken,
) {
    let mut ticker = interval(Duration::from_secs(WORKER_INTERVAL_SECS));
    loop {
        tokio::select! {
            _ = shutdown.cancelled() => {
                tracing::info!("Geofence worker stopping");
                break;
            }
            _ = ticker.tick() => {
                if let Err(e) = process_geofence_batch(&pool, &notifications).await {
                    tracing::error!(error = %e, "Geofence worker batch failed");
                }
            }
        }
    }
}

async fn process_geofence_batch(
    pool: &PgPool,
    notifications: &NotificationService,
) -> Result<(), AppError> {
    // Load un-notified registrations for events starting within 24 h.
    let registrations: Vec<GeofenceRow> = sqlx::query_as::<_, GeofenceRow>(
        r#"
        SELECT
            g.id,
            g.wallet_address,
            g.push_token,
            g.event_id,
            g.venue_lat,
            g.venue_lng,
            g.notified
        FROM geo_geofences g
        JOIN events e ON e.id = g.event_id
        WHERE
            g.notified = false
            AND e.start_date BETWEEN NOW() AND NOW() + INTERVAL '24 hours'
        "#,
    )
    .fetch_all(pool)
    .await
    .map_err(AppError::from)?;

    for reg in &registrations {
        // Fetch the most-recently reported device location.
        let location: Option<(f64, f64)> = sqlx::query_as(
            r#"
            SELECT latitude, longitude
            FROM user_locations
            WHERE wallet_address = $1
            ORDER BY recorded_at DESC
            LIMIT 1
            "#,
        )
        .bind(&reg.wallet_address)
        .fetch_optional(pool)
        .await
        .map_err(AppError::from)?;

        let (user_lat, user_lng) = match location {
            Some(pair) => pair,
            None => continue,
        };

        if haversine_m(user_lat, user_lng, reg.venue_lat, reg.venue_lng) > GEOFENCE_RADIUS_M {
            continue;
        }

        let event_title: Option<String> =
            sqlx::query_scalar("SELECT title FROM events WHERE id = $1")
                .bind(reg.event_id)
                .fetch_optional(pool)
                .await
                .map_err(AppError::from)?;

        let title = event_title.unwrap_or_else(|| "your event".into());

        let notification = Notification {
            recipient: reg.push_token.clone(),
            subject: "You're near the venue! 🎟️".into(),
            body: format!(
                "{title} is starting soon and you're close by. Show your ticket at the gate."
            ),
        };

        match notifications.send(&notification).await {
            Ok(_) => {
                tracing::info!(
                    geofence_id = %reg.id,
                    wallet      = %reg.wallet_address,
                    event_id    = %reg.event_id,
                    "Geofence proximity alert dispatched"
                );
                let _ = sqlx::query(
                    "UPDATE geo_geofences SET notified = true, updated_at = NOW() WHERE id = $1",
                )
                .bind(reg.id)
                .execute(pool)
                .await;
            }
            Err(e) => {
                tracing::warn!(geofence_id = %reg.id, error = %e, "Failed to dispatch geofence alert");
            }
        }
    }

    tracing::debug!(
        checked = registrations.len(),
        ts = %Utc::now(),
        "Geofence worker iteration complete"
    );
    Ok(())
}

// ---------------------------------------------------------------------------
// Helpers
// ---------------------------------------------------------------------------

/// Great-circle distance between two WGS-84 points in metres (Haversine).
fn haversine_m(lat1: f64, lng1: f64, lat2: f64, lng2: f64) -> f64 {
    const R: f64 = 6_371_000.0;
    let dlat = (lat2 - lat1).to_radians();
    let dlng = (lng2 - lng1).to_radians();
    let a = (dlat / 2.0).sin().powi(2)
        + lat1.to_radians().cos() * lat2.to_radians().cos() * (dlng / 2.0).sin().powi(2);
    R * 2.0 * a.sqrt().asin()
}
