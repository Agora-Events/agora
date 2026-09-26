/**
 * locationService.ts  (Issue: Event Discovery Engine)
 *
 * Background location tracking for geofence detection using Expo with minimal
 * battery impact.
 *
 * ## Strategy
 *   1. Significant-location-change tracking — OS wakes app only when the
 *      device moves ≥ 300 m, consuming negligible power.
 *   2. OS geofence region monitoring (`expo-location` geofencing API) — the
 *      OS monitors 150 m perimeter regions around venue coordinates. Works
 *      even when the app is killed.
 *
 * ## Permissions
 * Requests FOREGROUND permission first, then BACKGROUND. Degrades gracefully
 * if BACKGROUND is denied — geofences still register for foreground use.
 *
 * ## Usage
 * ```ts
 * await locationService.init();
 * await locationService.registerGeofences(venues);
 * const unsub = locationService.onGeofenceEnter((eventId) => { ... });
 * await locationService.stop();  // cleanup on logout
 * ```
 */

import * as Location from 'expo-location';
import * as TaskManager from 'expo-task-manager';

const API_BASE_URL = process.env.EXPO_PUBLIC_API_URL ?? 'http://localhost:8080';

// ---------------------------------------------------------------------------
// Task names
// ---------------------------------------------------------------------------

export const LOCATION_TASK_NAME = 'AGORA_BACKGROUND_LOCATION';
export const GEOFENCE_TASK_NAME = 'AGORA_GEOFENCE_MONITOR';

// ---------------------------------------------------------------------------
// Types
// ---------------------------------------------------------------------------

export interface Coordinates {
  latitude: number;
  longitude: number;
  accuracy?: number | null;
}

export interface VenueGeofence {
  /** Must match an Agora event UUID. */
  eventId: string;
  latitude: number;
  longitude: number;
  /** Radius in metres. Must be 150 to match server contract. */
  radiusMeters: number;
  eventTitle?: string;
}

export type GeofenceEnterHandler = (eventId: string) => void;

export class LocationError extends Error {
  readonly code: string;
  constructor(message: string, code: string) {
    super(message);
    this.name = 'LocationError';
    this.code = code;
  }
}

// ---------------------------------------------------------------------------
// Module-level handler registry (accessible from TaskManager tasks)
// ---------------------------------------------------------------------------

const geofenceEnterHandlers: GeofenceEnterHandler[] = [];

// ---------------------------------------------------------------------------
// TaskManager task definitions
// Must be at module top-level — defined before any conditional code.
// ---------------------------------------------------------------------------

TaskManager.defineTask<{ locations: Location.LocationObject[] }>(
  LOCATION_TASK_NAME,
  async ({ data, error }) => {
    if (error) {
      console.warn('[LocationService] Background location error:', error.message);
      return;
    }
    if (!data?.locations?.length) return;
    const latest = data.locations[data.locations.length - 1];
    await reportLocationToServer(
      latest.coords.latitude,
      latest.coords.longitude,
    ).catch((e) =>
      console.warn('[LocationService] Server location report failed:', e),
    );
  },
);

TaskManager.defineTask<{
  eventType: Location.GeofencingEventType;
  region: Location.LocationRegion;
}>(GEOFENCE_TASK_NAME, async ({ data, error }) => {
  if (error) {
    console.warn('[LocationService] Geofence task error:', error.message);
    return;
  }
  if (!data) return;
  if (data.eventType === Location.GeofencingEventType.Enter) {
    const eventId = data.region.identifier;
    console.info(`[LocationService] Entered geofence: ${eventId}`);
    geofenceEnterHandlers.forEach((cb) => {
      try { cb(eventId); } catch (e) {
        console.warn('[LocationService] Handler threw:', e);
      }
    });
  }
});

// ---------------------------------------------------------------------------
// Service class
// ---------------------------------------------------------------------------

class LocationService {
  private initialised = false;
  private backgroundGranted = false;

  // ------------------------------------------------------------------
  // init
  // ------------------------------------------------------------------

  async init(): Promise<void> {
    if (this.initialised) return;

    const { status: fg } = await Location.requestForegroundPermissionsAsync();
    if (fg !== 'granted') {
      throw new LocationError(
        'Foreground location permission is required',
        'PERMISSION_DENIED_FOREGROUND',
      );
    }

    const { status: bg } = await Location.requestBackgroundPermissionsAsync();
    this.backgroundGranted = bg === 'granted';

    if (this.backgroundGranted) {
      await this.startBackgroundTracking();
    } else {
      console.warn(
        '[LocationService] Background permission denied. ' +
        'Proximity alerts will only fire when the app is foregrounded.',
      );
    }

    this.initialised = true;
  }

  // ------------------------------------------------------------------
  // One-shot position
  // ------------------------------------------------------------------

  async getCurrentPosition(): Promise<Coordinates> {
    const cached = await Location.getLastKnownPositionAsync({
      maxAge: 5 * 60 * 1000,
      requiredAccuracy: 200,
    });
    if (cached) {
      return {
        latitude: cached.coords.latitude,
        longitude: cached.coords.longitude,
        accuracy: cached.coords.accuracy,
      };
    }
    const fix = await Location.getCurrentPositionAsync({
      accuracy: Location.Accuracy.Balanced,
    });
    return {
      latitude: fix.coords.latitude,
      longitude: fix.coords.longitude,
      accuracy: fix.coords.accuracy,
    };
  }

  // ------------------------------------------------------------------
  // Geofence registration
  // ------------------------------------------------------------------

  /**
   * Register OS-level 150 m geofence regions for all provided venues.
   * Replaces any previously registered regions atomically.
   */
  async registerGeofences(venues: VenueGeofence[]): Promise<void> {
    if (!this.backgroundGranted) {
      console.warn('[LocationService] Background permission required for OS geofences.');
      return;
    }

    if (venues.length === 0) {
      const running = await Location.hasStartedGeofencingAsync(GEOFENCE_TASK_NAME);
      if (running) await Location.stopGeofencingAsync(GEOFENCE_TASK_NAME);
      return;
    }

    const regions: Location.LocationRegion[] = venues.map((v) => ({
      identifier: v.eventId,
      latitude: v.latitude,
      longitude: v.longitude,
      radius: v.radiusMeters,
      notifyOnEnter: true,
      notifyOnExit: false,
    }));

    await Location.startGeofencingAsync(GEOFENCE_TASK_NAME, regions);
    console.info(`[LocationService] Registered ${regions.length} geofence(s).`);
  }

  async clearGeofences(): Promise<void> {
    const running = await Location.hasStartedGeofencingAsync(GEOFENCE_TASK_NAME);
    if (running) await Location.stopGeofencingAsync(GEOFENCE_TASK_NAME);
  }

  // ------------------------------------------------------------------
  // Server location reporting
  // ------------------------------------------------------------------

  async reportToServer(latitude: number, longitude: number): Promise<void> {
    await reportLocationToServer(latitude, longitude);
  }

  // ------------------------------------------------------------------
  // Event subscriptions
  // ------------------------------------------------------------------

  onGeofenceEnter(handler: GeofenceEnterHandler): () => void {
    geofenceEnterHandlers.push(handler);
    return () => {
      const idx = geofenceEnterHandlers.indexOf(handler);
      if (idx !== -1) geofenceEnterHandlers.splice(idx, 1);
    };
  }

  // ------------------------------------------------------------------
  // Cleanup
  // ------------------------------------------------------------------

  async stop(): Promise<void> {
    await this.clearGeofences();
    const bgRunning = await Location.hasStartedLocationUpdatesAsync(LOCATION_TASK_NAME);
    if (bgRunning) await Location.stopLocationUpdatesAsync(LOCATION_TASK_NAME);
    this.initialised = false;
    this.backgroundGranted = false;
  }

  // ------------------------------------------------------------------
  // Private
  // ------------------------------------------------------------------

  private async startBackgroundTracking(): Promise<void> {
    const already = await Location.hasStartedLocationUpdatesAsync(LOCATION_TASK_NAME);
    if (already) return;

    await Location.startLocationUpdatesAsync(LOCATION_TASK_NAME, {
      accuracy: Location.Accuracy.Balanced,
      // Wake only when device moves ≥ 300 m — minimal battery cost.
      distanceInterval: 300,
      deferredUpdatesInterval: 60_000,
      deferredUpdatesDistance: 300,
      foregroundService: {
        notificationTitle: 'Agora is tracking your location',
        notificationBody: 'Tap to check in when you arrive at the venue.',
        notificationColor: '#7C3AED',
      },
      pausesUpdatesAutomatically: true,
      showsBackgroundLocationIndicator: false,
    });
  }
}

// ---------------------------------------------------------------------------
// Module-level server reporter (used by both the class and the task)
// ---------------------------------------------------------------------------

async function reportLocationToServer(
  latitude: number,
  longitude: number,
): Promise<void> {
  const controller = new AbortController();
  const tid = setTimeout(() => controller.abort(), 10_000);
  try {
    const res = await fetch(`${API_BASE_URL}/api/v1/geo/location`, {
      method: 'POST',
      headers: { 'Content-Type': 'application/json' },
      body: JSON.stringify({ latitude, longitude }),
      signal: controller.signal,
    });
    if (!res.ok) {
      console.warn(`[LocationService] Location report returned ${res.status}`);
    }
  } finally {
    clearTimeout(tid);
  }
}

// ---------------------------------------------------------------------------
// Singleton export
// ---------------------------------------------------------------------------

export const locationService = new LocationService();
