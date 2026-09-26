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

use crate::utils::error::AppError;

/// Maximum accepted search radius — 500 km (Issue #1259). Values above this are
/// rejected at the edge with a `400` rather than forwarded to the database.
pub const MAX_RADIUS_M: f64 = 500_000.0;
/// Minimum accepted search radius (metres). Non-positive values are rejected.
pub const MIN_RADIUS_M: f64 = 1.0;

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

impl NearbyQuery {
    /// Validate every coordinate and radius supplied on this query, returning a
    /// `400`-class [`AppError`] that names the offending field. Surfacing bad
    /// input here (rather than in a downstream database constraint) keeps the
    /// endpoint from ever producing a `500` (Issue #1259).
    pub fn validate(&self) -> Result<(), AppError> {
        validate_latitude(self.lat)?;
        validate_longitude(self.lng)?;
        if let Some(radius) = self.radius_m {
            validate_radius(radius)?;
        }
        // Bounding-box corners, when supplied, must also be valid coordinates.
        if let Some(v) = self.bbox_sw_lat {
            validate_latitude(v)?;
        }
        if let Some(v) = self.bbox_ne_lat {
            validate_latitude(v)?;
        }
        if let Some(v) = self.bbox_sw_lng {
            validate_longitude(v)?;
        }
        if let Some(v) = self.bbox_ne_lng {
            validate_longitude(v)?;
        }
        Ok(())
    }
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

impl GeofenceRegistration {
    /// Validate the venue coordinates, returning a `400`-class [`AppError`] that
    /// names the offending field (Issue #1259).
    pub fn validate(&self) -> Result<(), AppError> {
        validate_latitude(self.venue_lat)?;
        validate_longitude(self.venue_lng)?;
        Ok(())
    }
}

// ---------------------------------------------------------------------------
// Field-level validators
// ---------------------------------------------------------------------------

/// Reject `NaN`/`inf` and out-of-range latitudes with a field-named error.
pub fn validate_latitude(lat: f64) -> Result<(), AppError> {
    if !lat.is_finite() {
        return Err(AppError::ValidationError(format!(
            "lat must be a finite number, got {lat}"
        )));
    }
    if !(-90.0..=90.0).contains(&lat) {
        return Err(AppError::ValidationError(format!(
            "lat must be between -90 and 90, got {lat}"
        )));
    }
    Ok(())
}

/// Reject `NaN`/`inf` and out-of-range longitudes with a field-named error.
pub fn validate_longitude(lng: f64) -> Result<(), AppError> {
    if !lng.is_finite() {
        return Err(AppError::ValidationError(format!(
            "lng must be a finite number, got {lng}"
        )));
    }
    if !(-180.0..=180.0).contains(&lng) {
        return Err(AppError::ValidationError(format!(
            "lng must be between -180 and 180, got {lng}"
        )));
    }
    Ok(())
}

/// Reject non-positive, non-finite, or overly large radii with a field-named
/// error. The ceiling is [`MAX_RADIUS_M`] (500 km).
pub fn validate_radius(radius_m: f64) -> Result<(), AppError> {
    if !radius_m.is_finite() {
        return Err(AppError::ValidationError(format!(
            "radius_m must be a finite number, got {radius_m}"
        )));
    }
    if radius_m < MIN_RADIUS_M {
        return Err(AppError::ValidationError(format!(
            "radius_m must be greater than 0, got {radius_m}"
        )));
    }
    if radius_m > MAX_RADIUS_M {
        return Err(AppError::ValidationError(format!(
            "radius_m must be at most {MAX_RADIUS_M} metres (500 km), got {radius_m}"
        )));
    }
    Ok(())
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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn latitude_boundaries_are_valid() {
        assert!(validate_latitude(-90.0).is_ok());
        assert!(validate_latitude(90.0).is_ok());
        assert!(validate_latitude(0.0).is_ok());
    }

    #[test]
    fn latitude_out_of_range_is_rejected() {
        let err = validate_latitude(-90.0001).unwrap_err();
        assert!(err.public_message().contains("lat"));
        let err = validate_latitude(90.0001).unwrap_err();
        assert!(err.public_message().contains("lat"));
    }

    #[test]
    fn latitude_nan_and_infinity_are_rejected() {
        let err = validate_latitude(f64::NAN).unwrap_err();
        assert!(err.public_message().contains("lat"));
        assert!(err.public_message().contains("finite"));
        let err = validate_latitude(f64::INFINITY).unwrap_err();
        assert!(err.public_message().contains("lat"));
        let err = validate_latitude(f64::NEG_INFINITY).unwrap_err();
        assert!(err.public_message().contains("lat"));
    }

    #[test]
    fn longitude_boundaries_are_valid() {
        assert!(validate_longitude(-180.0).is_ok());
        assert!(validate_longitude(180.0).is_ok());
    }

    #[test]
    fn longitude_out_of_range_is_rejected() {
        assert!(validate_longitude(-180.0001).is_err());
        assert!(validate_longitude(180.0001).is_err());
    }

    #[test]
    fn radius_boundaries_and_violations() {
        assert!(validate_radius(0.0).is_err());
        assert!(validate_radius(-5.0).is_err());
        assert!(validate_radius(f64::NAN).is_err());
        assert!(validate_radius(MAX_RADIUS_M).is_ok());
        assert!(validate_radius(MAX_RADIUS_M + 1.0).is_err());
        // Field name surfaced in the message.
        assert!(validate_radius(1_000_000.0)
            .unwrap_err()
            .public_message()
            .contains("radius_m"));
    }

    #[test]
    fn nearby_query_validate_accepts_valid_input() {
        let q = NearbyQuery {
            lat: 51.5074,
            lng: -0.1278,
            radius_m: Some(1000.0),
            bbox_sw_lat: None,
            bbox_sw_lng: None,
            bbox_ne_lat: None,
            bbox_ne_lng: None,
            limit: Some(10),
        };
        assert!(q.validate().is_ok());
    }

    #[test]
    fn nearby_query_validate_rejects_bad_latitude() {
        let q = NearbyQuery {
            lat: 99.0,
            lng: 0.0,
            radius_m: None,
            bbox_sw_lat: None,
            bbox_sw_lng: None,
            bbox_ne_lat: None,
            bbox_ne_lng: None,
            limit: None,
        };
        assert!(q.validate().is_err());
    }

    #[test]
    fn geofence_registration_validates_venue_coords() {
        let reg = GeofenceRegistration {
            event_id: uuid::Uuid::nil(),
            push_token: "tok".into(),
            venue_lat: -90.0,
            venue_lng: 180.0,
        };
        assert!(reg.validate().is_ok());

        let reg = GeofenceRegistration {
            event_id: uuid::Uuid::nil(),
            push_token: "tok".into(),
            venue_lat: f64::NAN,
            venue_lng: 0.0,
        };
        assert!(reg.validate().is_err());
    }
}
