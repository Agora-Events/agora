/**
 * useGeofence.ts  (Issue: Event Discovery Engine)
 *
 * React hook that wires together:
 *   1. `locationService` — background location + OS geofence regions.
 *   2. Server geofence registration (`POST /api/v1/geo/geofences`).
 *   3. Local Expo notifications with "Show Ticket" / "Gate Map" quick actions.
 *
 * ## Usage
 * ```tsx
 * const { status, nearbyEventId, dismissNearbyPrompt } = useGeofence({
 *   tickets,      // TicketWithVenue[]
 *   pushToken,    // Expo push token string | null
 *   authToken,    // JWT | null
 * });
 *
 * // When nearbyEventId is set, surface the check-in bottom sheet.
 * ```
 *
 * ## Flow
 * Mount → init locationService → register OS geofences → post to server →
 * geofence enter → schedule local notification (+ set nearbyEventId) →
 * user taps "Show Ticket" or "Gate Map" → navigate.
 */

import { useCallback, useEffect, useRef, useState } from 'react';
import * as Notifications from 'expo-notifications';
import { locationService, VenueGeofence } from '@/services/locationService';

const API_BASE_URL = process.env.EXPO_PUBLIC_API_URL ?? 'http://localhost:8080';

// ---------------------------------------------------------------------------
// Types
// ---------------------------------------------------------------------------

export interface TicketWithVenue {
  eventId: string;
  eventTitle: string;
  venueLat: number;
  venueLng: number;
}

export type GeofenceStatus =
  | 'idle'
  | 'initialising'
  | 'active'
  | 'permission-denied'
  | 'error';

export interface UseGeofenceOptions {
  tickets: TicketWithVenue[];
  pushToken: string | null;
  authToken: string | null;
  /** Geofence perimeter in metres (default: 150). */
  radiusMeters?: number;
}

export interface UseGeofenceResult {
  status: GeofenceStatus;
  geofencedEventIds: Set<string>;
  /** Set when the user enters a geofence; null otherwise. */
  nearbyEventId: string | null;
  errorMessage: string | null;
  /** Dismiss the nearby prompt (e.g., after navigating to ticket). */
  dismissNearbyPrompt: () => void;
  /** Re-register geofences (call after purchasing a new ticket). */
  refresh: () => Promise<void>;
}

// ---------------------------------------------------------------------------
// Notification setup
// ---------------------------------------------------------------------------

const PROXIMITY_CHANNEL_ID = 'agora-venue-proximity';
const PROXIMITY_CATEGORY = 'VENUE_PROXIMITY';

async function ensureNotificationSetup(): Promise<void> {
  await Notifications.setNotificationChannelAsync(PROXIMITY_CHANNEL_ID, {
    name: 'Venue proximity alerts',
    importance: Notifications.AndroidImportance.HIGH,
    vibrationPattern: [0, 250, 250, 250],
    lightColor: '#7C3AED',
  });

  await Notifications.setNotificationCategoryAsync(PROXIMITY_CATEGORY, [
    {
      identifier: 'SHOW_TICKET',
      buttonTitle: 'Show Ticket',
      options: { opensAppToForeground: true },
    },
    {
      identifier: 'GATE_MAP',
      buttonTitle: 'Gate Map',
      options: { opensAppToForeground: true },
    },
  ]);
}

// ---------------------------------------------------------------------------
// Server sync helper
// ---------------------------------------------------------------------------

async function registerGeofenceOnServer(
  eventId: string,
  pushToken: string,
  venueLat: number,
  venueLng: number,
  authToken: string,
): Promise<void> {
  const controller = new AbortController();
  const tid = setTimeout(() => controller.abort(), 15_000);
  try {
    const res = await fetch(`${API_BASE_URL}/api/v1/geo/geofences`, {
      method: 'POST',
      headers: {
        'Content-Type': 'application/json',
        Authorization: `Bearer ${authToken}`,
      },
      body: JSON.stringify({
        event_id: eventId,
        push_token: pushToken,
        venue_lat: venueLat,
        venue_lng: venueLng,
      }),
      signal: controller.signal,
    });
    if (!res.ok) {
      console.warn(`[useGeofence] Server sync returned ${res.status}`);
    }
  } finally {
    clearTimeout(tid);
  }
}

// ---------------------------------------------------------------------------
// Local notification helper
// ---------------------------------------------------------------------------

async function scheduleProximityNotification(
  eventId: string,
  eventTitle: string,
): Promise<void> {
  await Notifications.scheduleNotificationAsync({
    content: {
      title: "You're near the venue! 🎟️",
      body: `${eventTitle} is starting soon. Tap to check in.`,
      categoryIdentifier: PROXIMITY_CATEGORY,
      data: { eventId, action: 'proximity-alert' },
      sound: true,
    },
    trigger: null, // fire immediately
  });
}

// ---------------------------------------------------------------------------
// Hook
// ---------------------------------------------------------------------------

const DEFAULT_RADIUS_M = 150;

export function useGeofence({
  tickets,
  pushToken,
  authToken,
  radiusMeters = DEFAULT_RADIUS_M,
}: UseGeofenceOptions): UseGeofenceResult {
  const [status, setStatus] = useState<GeofenceStatus>('idle');
  const [geofencedEventIds, setGeofencedEventIds] = useState<Set<string>>(
    new Set(),
  );
  const [nearbyEventId, setNearbyEventId] = useState<string | null>(null);
  const [errorMessage, setErrorMessage] = useState<string | null>(null);

  // Stable ref so handlers always close over latest tickets without re-register.
  const ticketsRef = useRef(tickets);
  ticketsRef.current = tickets;

  const unsubscribeRef = useRef<(() => void) | null>(null);

  // ------------------------------------------------------------------
  // Register geofences (OS + server)
  // ------------------------------------------------------------------

  const registerGeofences = useCallback(async () => {
    const current = ticketsRef.current;

    if (current.length === 0) {
      await locationService.clearGeofences();
      setGeofencedEventIds(new Set());
      return;
    }

    const venues: VenueGeofence[] = current.map((t) => ({
      eventId: t.eventId,
      latitude: t.venueLat,
      longitude: t.venueLng,
      radiusMeters,
      eventTitle: t.eventTitle,
    }));

    await locationService.registerGeofences(venues);
    setGeofencedEventIds(new Set(current.map((t) => t.eventId)));

    // Server sync — fire-and-forget; best effort.
    if (pushToken && authToken) {
      for (const t of current) {
        registerGeofenceOnServer(
          t.eventId,
          pushToken,
          t.venueLat,
          t.venueLng,
          authToken,
        ).catch((e) =>
          console.warn(`[useGeofence] Server sync failed (${t.eventId}):`, e),
        );
      }
    }
  }, [radiusMeters, pushToken, authToken]);

  // ------------------------------------------------------------------
  // Initialise
  // ------------------------------------------------------------------

  const initialise = useCallback(async () => {
    setStatus('initialising');
    setErrorMessage(null);

    try {
      await ensureNotificationSetup();
      await locationService.init();
      await registerGeofences();

      // Subscribe to OS geofence-enter events.
      unsubscribeRef.current?.();
      unsubscribeRef.current = locationService.onGeofenceEnter(
        async (eventId) => {
          const ticket = ticketsRef.current.find((t) => t.eventId === eventId);
          if (!ticket) return;
          setNearbyEventId(eventId);
          await scheduleProximityNotification(
            eventId,
            ticket.eventTitle,
          ).catch((e) =>
            console.warn('[useGeofence] Notification failed:', e),
          );
        },
      );

      setStatus('active');
    } catch (err: unknown) {
      const msg = err instanceof Error ? err.message : String(err);
      setStatus(msg.includes('PERMISSION_DENIED') ? 'permission-denied' : 'error');
      setErrorMessage(msg);
      console.error('[useGeofence] Init failed:', err);
    }
  }, [registerGeofences]);

  // ------------------------------------------------------------------
  // Notification response listener (quick-action buttons)
  // ------------------------------------------------------------------

  useEffect(() => {
    const sub = Notifications.addNotificationResponseReceivedListener(
      (response) => {
        const { actionIdentifier, notification } = response;
        const data = notification.request.content.data as Record<string, unknown>;
        const eventId = data?.eventId as string | undefined;
        if (!eventId) return;

        if (actionIdentifier === 'SHOW_TICKET') {
          setNearbyEventId(eventId);
        } else if (actionIdentifier === 'GATE_MAP') {
          // Prefix signals gate-map intent to the navigation layer.
          setNearbyEventId(`gate-map::${eventId}`);
        }
      },
    );
    return () => sub.remove();
  }, []);

  // ------------------------------------------------------------------
  // Lifecycle
  // ------------------------------------------------------------------

  useEffect(() => {
    initialise();
    return () => {
      unsubscribeRef.current?.();
      locationService.clearGeofences().catch(() => {});
    };
    // eslint-disable-next-line react-hooks/exhaustive-deps
  }, []);

  // Re-register when ticket list changes after first init.
  useEffect(() => {
    if (status === 'active') {
      registerGeofences().catch((e) =>
        console.warn('[useGeofence] Re-registration failed:', e),
      );
    }
    // eslint-disable-next-line react-hooks/exhaustive-deps
  }, [tickets]);

  // ------------------------------------------------------------------
  // Public API
  // ------------------------------------------------------------------

  const dismissNearbyPrompt = useCallback(() => setNearbyEventId(null), []);
  const refresh = useCallback(async () => registerGeofences(), [registerGeofences]);

  return {
    status,
    geofencedEventIds,
    nearbyEventId,
    errorMessage,
    dismissNearbyPrompt,
    refresh,
  };
}
