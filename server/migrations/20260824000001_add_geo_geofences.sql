-- Migration: add_geo_geofences (Issue: Event Discovery Engine)
--
-- Creates the `geo_geofences` table used by:
--   • POST /api/v1/geo/geofences  (register device/venue pair)
--   • background geofence worker  (poll for proximity alerts)
--
-- Also creates `user_locations` for the server-side worker fallback that
-- checks whether a user's last-known position is inside a fence.
--
-- Event coordinates for radius queries already exist in
-- events.latitude / events.longitude (20260729000001_add_event_coordinates.sql).

-- ---------------------------------------------------------------------------
-- user_locations: most-recent position per wallet (geofence worker fallback)
-- ---------------------------------------------------------------------------
CREATE TABLE IF NOT EXISTS user_locations (
    id              UUID             PRIMARY KEY DEFAULT gen_random_uuid(),
    wallet_address  TEXT             NOT NULL,
    latitude        DOUBLE PRECISION NOT NULL
                        CHECK (latitude  >= -90  AND latitude  <= 90),
    longitude       DOUBLE PRECISION NOT NULL
                        CHECK (longitude >= -180 AND longitude <= 180),
    recorded_at     TIMESTAMPTZ      NOT NULL DEFAULT NOW()
);

-- One row per wallet; upsert keeps the latest position.
CREATE UNIQUE INDEX IF NOT EXISTS idx_user_locations_wallet
    ON user_locations (wallet_address);

-- ---------------------------------------------------------------------------
-- geo_geofences: per-device, per-event proximity registration
-- ---------------------------------------------------------------------------
CREATE TABLE IF NOT EXISTS geo_geofences (
    id              UUID             PRIMARY KEY DEFAULT gen_random_uuid(),
    wallet_address  TEXT             NOT NULL,
    push_token      TEXT             NOT NULL,
    event_id        UUID             NOT NULL REFERENCES events (id) ON DELETE CASCADE,
    venue_lat       DOUBLE PRECISION NOT NULL
                        CHECK (venue_lat  >= -90  AND venue_lat  <= 90),
    venue_lng       DOUBLE PRECISION NOT NULL
                        CHECK (venue_lng >= -180 AND venue_lng <= 180),
    -- Set to true after the first proximity alert is sent; prevents duplicates.
    notified        BOOLEAN          NOT NULL DEFAULT false,
    created_at      TIMESTAMPTZ      NOT NULL DEFAULT NOW(),
    updated_at      TIMESTAMPTZ      NOT NULL DEFAULT NOW()
);

-- Idempotent re-registration: same device + event updates coords.
CREATE UNIQUE INDEX IF NOT EXISTS idx_geo_geofences_token_event
    ON geo_geofences (push_token, event_id);

-- Worker index: fetch un-notified registrations for events starting soon.
CREATE INDEX IF NOT EXISTS idx_geo_geofences_notified_event
    ON geo_geofences (event_id, notified)
    WHERE notified = false;

-- ---------------------------------------------------------------------------
-- Trigger: maintain updated_at on geo_geofences
-- ---------------------------------------------------------------------------
CREATE OR REPLACE FUNCTION set_geo_updated_at()
RETURNS TRIGGER LANGUAGE plpgsql AS $$
BEGIN
    NEW.updated_at = NOW();
    RETURN NEW;
END;
$$;

DO $$
BEGIN
    IF NOT EXISTS (
        SELECT 1 FROM pg_trigger WHERE tgname = 'trg_geo_geofences_updated_at'
    ) THEN
        CREATE TRIGGER trg_geo_geofences_updated_at
            BEFORE UPDATE ON geo_geofences
            FOR EACH ROW EXECUTE FUNCTION set_geo_updated_at();
    END IF;
END
$$;
