import { act, renderHook, waitFor } from '@testing-library/react-native';
import * as Location from 'expo-location';
import * as Notifications from 'expo-notifications';
import { useGeofence, TicketWithVenue } from '../useGeofence';
import { GEOFENCE_TASK_NAME, locationService } from '@/services/locationService';

/**
 * Issue #1463 — useGeofence.
 *
 * `expo-location` is mocked so no native permission prompts or OS geofence
 * regions are touched. The real `locationService` singleton runs on top of
 * that mock, so these tests cover the hook → service → `expo-location` path
 * end to end: permission handling, `startGeofencingAsync` with the ticket
 * venues, and `stopGeofencingAsync` on unmount.
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

const baseOptions = { tickets, pushToken: null, authToken: null };

function grantPermissions({ background = true } = {}) {
  mockLocation.requestForegroundPermissionsAsync.mockResolvedValue({ status: 'granted' } as any);
  mockLocation.requestBackgroundPermissionsAsync.mockResolvedValue({
    status: background ? 'granted' : 'denied',
  } as any);
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
  });

  it('uses a custom radius when one is provided', async () => {
    grantPermissions();

    const { result } = renderHook(() => useGeofence({ ...baseOptions, radiusMeters: 300 }));

    await waitFor(() => expect(result.current.status).toBe('active'));

    const [, regions] = mockLocation.startGeofencingAsync.mock.calls[0];
    expect(regions.every((r: Location.LocationRegion) => r.radius === 300)).toBe(true);
  });

  it('surfaces an error and never starts geofencing when foreground permission is denied', async () => {
    mockLocation.requestForegroundPermissionsAsync.mockResolvedValue({ status: 'denied' } as any);

    const { result } = renderHook(() => useGeofence(baseOptions));

    // NOTE: `locationService` throws a `LocationError` whose *code* is
    // `PERMISSION_DENIED_FOREGROUND`, but the hook only inspects the message
    // text for "PERMISSION_DENIED", so a denial currently lands in `error`
    // rather than `permission-denied`.
    await waitFor(() => expect(result.current.status).toBe('error'));

    expect(result.current.errorMessage).toMatch(/permission is required/i);
    expect(mockLocation.requestBackgroundPermissionsAsync).not.toHaveBeenCalled();
    expect(mockLocation.startGeofencingAsync).not.toHaveBeenCalled();
    expect(result.current.geofencedEventIds.size).toBe(0);
  });

  it('stays active without OS geofences when only background permission is denied', async () => {
    grantPermissions({ background: false });

    const { result } = renderHook(() => useGeofence(baseOptions));

    await waitFor(() => expect(result.current.status).toBe('active'));

    expect(mockLocation.startLocationUpdatesAsync).not.toHaveBeenCalled();
    expect(mockLocation.startGeofencingAsync).not.toHaveBeenCalled();
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
      await enterHandler?.('evt-2');
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
});
