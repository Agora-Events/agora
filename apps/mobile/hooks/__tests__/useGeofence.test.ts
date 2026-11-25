import { act, renderHook, waitFor } from '@testing-library/react-native';
import * as Location from 'expo-location';
import * as Notifications from 'expo-notifications';
import { useGeofence, TicketWithVenue } from '../useGeofence';
import { GEOFENCE_TASK_NAME, locationService } from '@/services/locationService';
import { apiUrl } from '@/constants/api';

/**
 * Issue #1463 — useGeofence.
 *
 * `expo-location` is mocked so no native permission prompts or OS geofence
 * regions are touched. The real `locationService` singleton runs on top of
 * that mock, so these tests cover the hook → service → `expo-location` path
 * end to end: permission handling, `startGeofencingAsync` with the ticket
 * venues, server registration, notification dispatch, and cleanup on unmount.
 *
 * The suite also guards the bugs this coverage exposed: `permission-denied`
 * used to be unreachable because the hook matched a human-readable message,
 * `geofencedEventIds` claimed success when background permission was denied,
 * and an unmount during initialisation could leave registered regions behind.
 */

jest.mock(
  'expo-location',
  () => ({
    requestForegroundPermissionsAsync: jest.fn(),
    requestBackgroundPermissionsAsync: jest.fn(),
    hasStartedLocationUpdatesAsync: jest.fn(),
    startLocationUpdatesAsync: jest.fn(),
    stopLocationUpdatesAsync: jest.fn(),
    hasStartedGeofencingAsync: jest.fn(),
    startGeofencingAsync: jest.fn(),
    stopGeofencingAsync: jest.fn(),
    getLastKnownPositionAsync: jest.fn(),
    getCurrentPositionAsync: jest.fn(),
    Accuracy: { Balanced: 3 },
    GeofencingEventType: { Enter: 1, Exit: 2 },
  }),
  { virtual: true }
);

jest.mock('expo-task-manager', () => ({ defineTask: jest.fn() }), { virtual: true });

jest.mock('expo-notifications', () => ({
  setNotificationChannelAsync: jest.fn().mockResolvedValue(undefined),
  setNotificationCategoryAsync: jest.fn().mockResolvedValue(undefined),
  scheduleNotificationAsync: jest.fn().mockResolvedValue('notif-id'),
  addNotificationResponseReceivedListener: jest.fn(),
  AndroidImportance: { HIGH: 4 },
}));

const mockLocation = Location as jest.Mocked<typeof Location>;
const mockNotifications = Notifications as jest.Mocked<typeof Notifications>;
const mockRemoveResponseListener = jest.fn();

const tickets: TicketWithVenue[] = [
  { eventId: 'evt-1', eventTitle: 'Stellar Meridian 2026', venueLat: 6.5244, venueLng: 3.3792 },
  { eventId: 'evt-2', eventTitle: 'Soroban Summit', venueLat: 51.5072, venueLng: -0.1276 },
];

const extraTicket: TicketWithVenue = {
  eventId: 'evt-3',
  eventTitle: 'Builder Meetup',
  venueLat: 48.8566,
  venueLng: 2.3522,
};

const baseOptions = { tickets, pushToken: null, authToken: null };

function grantPermissions({ background = true } = {}) {
  mockLocation.requestForegroundPermissionsAsync.mockResolvedValue({ status: 'granted' } as any);
  mockLocation.requestBackgroundPermissionsAsync.mockResolvedValue({
    status: background ? 'granted' : 'denied',
  } as any);
}

function responseFor(actionIdentifier: string, eventId: string) {
  return {
    actionIdentifier,
    notification: { request: { content: { data: { eventId } } } },
  } as any;
}

beforeEach(() => {
  jest.clearAllMocks();
  jest.spyOn(console, 'info').mockImplementation(() => {});
  jest.spyOn(console, 'warn').mockImplementation(() => {});
  jest.spyOn(console, 'error').mockImplementation(() => {});

  // `locationService` is a module singleton that caches its permission result;
  // reset it so each test goes through the permission flow again.
  (locationService as any).initialised = false;
  (locationService as any).backgroundGranted = false;

  (global as any).fetch = jest.fn().mockResolvedValue({ ok: true, status: 200 });

  mockLocation.hasStartedLocationUpdatesAsync.mockResolvedValue(false);
  mockLocation.startLocationUpdatesAsync.mockResolvedValue(undefined);
  mockLocation.hasStartedGeofencingAsync.mockResolvedValue(true);
  mockLocation.startGeofencingAsync.mockResolvedValue(undefined);
  mockLocation.stopGeofencingAsync.mockResolvedValue(undefined);
  mockNotifications.addNotificationResponseReceivedListener.mockReturnValue({
    remove: mockRemoveResponseListener,
  } as any);
});

afterEach(() => {
  jest.restoreAllMocks();
});

describe('useGeofence', () => {
  it('starts geofencing for every ticket venue once permissions are granted', async () => {
    grantPermissions();

    const { result } = renderHook(() => useGeofence(baseOptions));

    await waitFor(() => expect(result.current.status).toBe('active'));

    expect(mockLocation.requestForegroundPermissionsAsync).toHaveBeenCalledTimes(1);
    expect(mockLocation.requestBackgroundPermissionsAsync).toHaveBeenCalledTimes(1);
    expect(mockLocation.startGeofencingAsync).toHaveBeenCalledWith(GEOFENCE_TASK_NAME, [
      expect.objectContaining({
        identifier: 'evt-1',
        latitude: 6.5244,
        longitude: 3.3792,
        radius: 150,
        notifyOnEnter: true,
      }),
      expect.objectContaining({
        identifier: 'evt-2',
        latitude: 51.5072,
        longitude: -0.1276,
        radius: 150,
        notifyOnEnter: true,
      }),
    ]);
    expect(result.current.geofencedEventIds).toEqual(new Set(['evt-1', 'evt-2']));
    expect(result.current.errorMessage).toBeNull();
    expect(mockNotifications.setNotificationChannelAsync).toHaveBeenCalledTimes(1);
  });

  it('uses a custom radius when one is provided', async () => {
    grantPermissions();

    const { result } = renderHook(() => useGeofence({ ...baseOptions, radiusMeters: 300 }));

    await waitFor(() => expect(result.current.status).toBe('active'));

    const [, regions] = mockLocation.startGeofencingAsync.mock.calls[0];
    expect(regions.every((r: Location.LocationRegion) => r.radius === 300)).toBe(true);
  });

  it('reports the dedicated permission-denied state and never starts geofencing', async () => {
    mockLocation.requestForegroundPermissionsAsync.mockResolvedValue({ status: 'denied' } as any);

    const { result } = renderHook(() => useGeofence(baseOptions));

    await waitFor(() => expect(result.current.status).toBe('permission-denied'));

    expect(result.current.errorMessage).toMatch(/permission is required/i);
    expect(mockLocation.requestBackgroundPermissionsAsync).not.toHaveBeenCalled();
    expect(mockLocation.startGeofencingAsync).not.toHaveBeenCalled();
    expect(result.current.geofencedEventIds.size).toBe(0);
  });

  it('surfaces a generic error when initialisation fails for a non-permission reason', async () => {
    mockLocation.requestForegroundPermissionsAsync.mockRejectedValue(
      new Error('location services off')
    );

    const { result } = renderHook(() => useGeofence(baseOptions));

    await waitFor(() => expect(result.current.status).toBe('error'));
    expect(result.current.errorMessage).toBe('location services off');
    expect(mockLocation.startGeofencingAsync).not.toHaveBeenCalled();
  });

  it('stays active with no geofences reported when only background permission is denied', async () => {
    grantPermissions({ background: false });

    const { result } = renderHook(() =>
      useGeofence({ ...baseOptions, pushToken: 'ExponentPushToken[abc]', authToken: 'jwt-123' })
    );

    await waitFor(() => expect(result.current.status).toBe('active'));

    expect(mockLocation.startLocationUpdatesAsync).not.toHaveBeenCalled();
    expect(mockLocation.startGeofencingAsync).not.toHaveBeenCalled();
    // Reporting these ids would make the UI promise alerts the OS cannot fire.
    expect(result.current.geofencedEventIds.size).toBe(0);
    // No point telling the backend to geofence venues the device is not
    // monitoring either.
    expect(global.fetch).not.toHaveBeenCalled();
  });

  it('registers every venue with the backend when a push and auth token are available', async () => {
    grantPermissions();

    const { result } = renderHook(() =>
      useGeofence({
        ...baseOptions,
        pushToken: 'ExponentPushToken[abc]',
        authToken: 'jwt-123',
      })
    );

    await waitFor(() => expect(result.current.status).toBe('active'));
    await waitFor(() => expect(global.fetch).toHaveBeenCalledTimes(2));

    const [url, init] = (global.fetch as jest.Mock).mock.calls[0];
    expect(url).toBe(apiUrl('/geo/geofences'));
    expect(init.method).toBe('POST');
    expect(init.headers).toEqual({
      'Content-Type': 'application/json',
      Authorization: 'Bearer jwt-123',
    });
    expect(JSON.parse(init.body)).toEqual({
      event_id: 'evt-1',
      push_token: 'ExponentPushToken[abc]',
      venue_lat: 6.5244,
      venue_lng: 3.3792,
    });
    expect(JSON.parse((global.fetch as jest.Mock).mock.calls[1][1].body)).toMatchObject({
      event_id: 'evt-2',
      venue_lat: 51.5072,
      venue_lng: -0.1276,
    });
  });

  it('does not call the backend when no push or auth token is available', async () => {
    grantPermissions();

    const { result } = renderHook(() => useGeofence(baseOptions));
    await waitFor(() => expect(result.current.status).toBe('active'));

    expect(global.fetch).not.toHaveBeenCalled();
  });

  it('sets nearbyEventId and schedules a notification when a geofence is entered', async () => {
    grantPermissions();
    let enterHandler: ((eventId: string) => void) | undefined;
    const realOnGeofenceEnter = locationService.onGeofenceEnter.bind(locationService);
    jest.spyOn(locationService, 'onGeofenceEnter').mockImplementation((handler) => {
      enterHandler = handler;
      return realOnGeofenceEnter(handler);
    });

    const { result } = renderHook(() => useGeofence(baseOptions));
    await waitFor(() => expect(result.current.status).toBe('active'));

    await act(async () => {
      await (enterHandler as any)?.('evt-2');
    });

    expect(result.current.nearbyEventId).toBe('evt-2');
    expect(mockNotifications.scheduleNotificationAsync).toHaveBeenCalledWith(
      expect.objectContaining({
        content: expect.objectContaining({
          data: { eventId: 'evt-2', action: 'proximity-alert' },
        }),
      })
    );

    act(() => {
      result.current.dismissNearbyPrompt();
    });
    expect(result.current.nearbyEventId).toBeNull();
  });

  it('ignores geofence entries for events the user has no ticket for', async () => {
    grantPermissions();
    let enterHandler: ((eventId: string) => void) | undefined;
    const realOnGeofenceEnter = locationService.onGeofenceEnter.bind(locationService);
    jest.spyOn(locationService, 'onGeofenceEnter').mockImplementation((handler) => {
      enterHandler = handler;
      return realOnGeofenceEnter(handler);
    });

    const { result } = renderHook(() => useGeofence(baseOptions));
    await waitFor(() => expect(result.current.status).toBe('active'));

    await act(async () => {
      await (enterHandler as any)?.('evt-unknown');
    });

    expect(result.current.nearbyEventId).toBeNull();
    expect(mockNotifications.scheduleNotificationAsync).not.toHaveBeenCalled();
  });

  it('maps notification quick actions to ticket and gate-map intents', async () => {
    grantPermissions();
    let responseHandler: ((response: any) => void) | undefined;
    mockNotifications.addNotificationResponseReceivedListener.mockImplementation((handler: any) => {
      responseHandler = handler;
      return { remove: mockRemoveResponseListener } as any;
    });

    const { result } = renderHook(() => useGeofence(baseOptions));
    await waitFor(() => expect(result.current.status).toBe('active'));

    act(() => {
      responseHandler?.(responseFor('SHOW_TICKET', 'evt-2'));
    });
    expect(result.current.nearbyEventId).toBe('evt-2');

    act(() => {
      responseHandler?.(responseFor('GATE_MAP', 'evt-1'));
    });
    expect(result.current.nearbyEventId).toBe('gate-map::evt-1');
  });

  it('stops geofencing and removes listeners on unmount', async () => {
    grantPermissions();

    const { result, unmount } = renderHook(() => useGeofence(baseOptions));
    await waitFor(() => expect(result.current.status).toBe('active'));
    expect(mockLocation.stopGeofencingAsync).not.toHaveBeenCalled();

    unmount();

    await waitFor(() =>
      expect(mockLocation.stopGeofencingAsync).toHaveBeenCalledWith(GEOFENCE_TASK_NAME)
    );
    expect(mockRemoveResponseListener).toHaveBeenCalledTimes(1);
  });

  it('does not register or subscribe when unmounted while permissions are being requested', async () => {
    let resolveForeground: ((value: any) => void) | undefined;
    mockLocation.requestForegroundPermissionsAsync.mockImplementation(
      () =>
        new Promise((resolve) => {
          resolveForeground = resolve;
        }) as any
    );
    mockLocation.requestBackgroundPermissionsAsync.mockResolvedValue({ status: 'granted' } as any);
    const onEnterSpy = jest.spyOn(locationService, 'onGeofenceEnter');

    const { unmount } = renderHook(() => useGeofence(baseOptions));
    await waitFor(() => expect(mockLocation.requestForegroundPermissionsAsync).toHaveBeenCalled());

    unmount();

    await act(async () => {
      resolveForeground?.({ status: 'granted' });
    });

    // Cleanup ran before init() finished; init() then clears again so it cannot
    // leave the regions it just registered behind.
    await waitFor(() => expect(mockLocation.stopGeofencingAsync).toHaveBeenCalledTimes(2));
    expect(mockLocation.startGeofencingAsync).not.toHaveBeenCalled();
    expect(onEnterSpy).not.toHaveBeenCalled();
  });

  it('re-registers when the ticket list changes and when refresh() is called', async () => {
    grantPermissions();

    const { result, rerender } = renderHook(
      (props: { options: typeof baseOptions }) => useGeofence(props.options),
      { initialProps: { options: baseOptions } }
    );
    await waitFor(() => expect(result.current.status).toBe('active'));
    expect(mockLocation.startGeofencingAsync).toHaveBeenCalledTimes(1);

    rerender({ options: { ...baseOptions, tickets: [...tickets, extraTicket] } });

    await waitFor(() => expect(mockLocation.startGeofencingAsync).toHaveBeenCalledTimes(2));
    const [, regions] = mockLocation.startGeofencingAsync.mock.calls[1];
    expect(regions.map((r: Location.LocationRegion) => r.identifier)).toEqual([
      'evt-1',
      'evt-2',
      'evt-3',
    ]);
    expect(result.current.geofencedEventIds).toEqual(new Set(['evt-1', 'evt-2', 'evt-3']));

    await act(async () => {
      await result.current.refresh();
    });
    expect(mockLocation.startGeofencingAsync).toHaveBeenCalledTimes(3);
  });

  it('clears geofences instead of registering when the user has no tickets', async () => {
    grantPermissions();

    const { result } = renderHook(() => useGeofence({ ...baseOptions, tickets: [] }));

    await waitFor(() => expect(result.current.status).toBe('active'));

    expect(mockLocation.startGeofencingAsync).not.toHaveBeenCalled();
    expect(mockLocation.stopGeofencingAsync).toHaveBeenCalledWith(GEOFENCE_TASK_NAME);
    expect(result.current.geofencedEventIds.size).toBe(0);
  });
});
