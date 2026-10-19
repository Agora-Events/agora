/**
 * Issue #1467 — unit tests for `locationService`. `expo-location` and
 * `expo-task-manager` are fully mocked so no native device APIs or network
 * requests are touched.
 */

jest.mock('expo-location', () => ({
  requestForegroundPermissionsAsync: jest.fn(),
  requestBackgroundPermissionsAsync: jest.fn(),
  getLastKnownPositionAsync: jest.fn(),
  getCurrentPositionAsync: jest.fn(),
  hasStartedLocationUpdatesAsync: jest.fn(),
  startLocationUpdatesAsync: jest.fn(),
  stopLocationUpdatesAsync: jest.fn(),
  hasStartedGeofencingAsync: jest.fn(),
  startGeofencingAsync: jest.fn(),
  stopGeofencingAsync: jest.fn(),
  Accuracy: { Balanced: 3 },
  GeofencingEventType: { Enter: 1, Exit: 2 },
}));

jest.mock('expo-task-manager', () => ({
  defineTask: jest.fn(),
}));

import * as Location from 'expo-location';
import * as TaskManager from 'expo-task-manager';

type LocationServiceModule = typeof import('../locationService');

const mockLocation = Location as jest.Mocked<typeof Location>;
const mockTaskManager = TaskManager as jest.Mocked<typeof TaskManager>;

const MOCK_COORDS = { latitude: 6.5244, longitude: 3.3792, accuracy: 12 };

/** Load a fresh copy of the module so the singleton starts uninitialised. */
function loadModule(): LocationServiceModule {
  let mod!: LocationServiceModule;
  jest.isolateModules(() => {
    mod = require('../locationService');
  });
  return mod;
}

function mockPermissions(foreground: boolean, background: boolean) {
  mockLocation.requestForegroundPermissionsAsync.mockResolvedValue({
    status: foreground ? 'granted' : 'denied',
    granted: foreground,
  } as any);
  mockLocation.requestBackgroundPermissionsAsync.mockResolvedValue({
    status: background ? 'granted' : 'denied',
    granted: background,
  } as any);
}

beforeEach(() => {
  jest.clearAllMocks();
  jest.spyOn(console, 'warn').mockImplementation(() => {});
  jest.spyOn(console, 'info').mockImplementation(() => {});
  global.fetch = jest.fn();

  mockLocation.getLastKnownPositionAsync.mockResolvedValue(null);
  mockLocation.getCurrentPositionAsync.mockResolvedValue({
    coords: MOCK_COORDS,
    timestamp: Date.now(),
  } as any);
  mockLocation.hasStartedLocationUpdatesAsync.mockResolvedValue(false);
  mockLocation.hasStartedGeofencingAsync.mockResolvedValue(false);
});

afterEach(() => {
  jest.restoreAllMocks();
});

describe('locationService — permission granted', () => {
  it('initialises and returns coordinates from getCurrentPositionAsync', async () => {
    mockPermissions(true, true);
    const { locationService } = loadModule();

    await expect(locationService.init()).resolves.toBeUndefined();
    const position = await locationService.getCurrentPosition();

    expect(mockLocation.requestForegroundPermissionsAsync).toHaveBeenCalledTimes(1);
    expect(mockLocation.getCurrentPositionAsync).toHaveBeenCalledWith({
      accuracy: Location.Accuracy.Balanced,
    });
    expect(position).toEqual(MOCK_COORDS);
  });

  it('prefers a cached last-known position when one is available', async () => {
    mockPermissions(true, true);
    mockLocation.getLastKnownPositionAsync.mockResolvedValueOnce({
      coords: { latitude: 1, longitude: 2, accuracy: 50 },
      timestamp: Date.now(),
    } as any);
    const { locationService } = loadModule();

    const position = await locationService.getCurrentPosition();

    expect(position).toEqual({ latitude: 1, longitude: 2, accuracy: 50 });
    expect(mockLocation.getCurrentPositionAsync).not.toHaveBeenCalled();
  });
});

describe('locationService — permission denied', () => {
  it('throws a LocationError when foreground permission is denied', async () => {
    mockPermissions(false, false);
    const { locationService, LocationError } = loadModule();

    const error = await locationService.init().catch((e) => e);

    expect(error).toBeInstanceOf(LocationError);
    expect(error.code).toBe('PERMISSION_DENIED_FOREGROUND');
    expect(mockLocation.requestBackgroundPermissionsAsync).not.toHaveBeenCalled();
    expect(mockLocation.startLocationUpdatesAsync).not.toHaveBeenCalled();
  });

  it('skips background tracking when only background permission is denied', async () => {
    mockPermissions(true, false);
    const { locationService } = loadModule();

    await locationService.init();
    await locationService.registerGeofences([
      { eventId: 'evt-1', latitude: 1, longitude: 2, radiusMeters: 150 },
    ]);

    expect(mockLocation.startLocationUpdatesAsync).not.toHaveBeenCalled();
    expect(mockLocation.startGeofencingAsync).not.toHaveBeenCalled();
  });
});

describe('locationService — background task registration', () => {
  it('defines the location and geofence tasks with TaskManager on import', () => {
    const { LOCATION_TASK_NAME, GEOFENCE_TASK_NAME } = loadModule();

    const taskNames = mockTaskManager.defineTask.mock.calls.map(([name]) => name);
    expect(taskNames).toEqual(expect.arrayContaining([LOCATION_TASK_NAME, GEOFENCE_TASK_NAME]));
    expect(LOCATION_TASK_NAME).toBe('AGORA_BACKGROUND_LOCATION');
    expect(GEOFENCE_TASK_NAME).toBe('AGORA_GEOFENCE_MONITOR');
  });

  it('starts background location updates under LOCATION_TASK_NAME', async () => {
    mockPermissions(true, true);
    const { locationService, LOCATION_TASK_NAME } = loadModule();

    await locationService.init();

    expect(mockLocation.startLocationUpdatesAsync).toHaveBeenCalledWith(
      LOCATION_TASK_NAME,
      expect.objectContaining({ distanceInterval: 300 }),
    );
  });

  it('does not restart location updates if the task is already running', async () => {
    mockPermissions(true, true);
    mockLocation.hasStartedLocationUpdatesAsync.mockResolvedValue(true);
    const { locationService } = loadModule();

    await locationService.init();

    expect(mockLocation.startLocationUpdatesAsync).not.toHaveBeenCalled();
  });

  it('registers geofence regions under GEOFENCE_TASK_NAME', async () => {
    mockPermissions(true, true);
    const { locationService, GEOFENCE_TASK_NAME } = loadModule();

    await locationService.init();
    await locationService.registerGeofences([
      { eventId: 'evt-1', latitude: 6.5, longitude: 3.3, radiusMeters: 150 },
    ]);

    expect(mockLocation.startGeofencingAsync).toHaveBeenCalledWith(GEOFENCE_TASK_NAME, [
      {
        identifier: 'evt-1',
        latitude: 6.5,
        longitude: 3.3,
        radius: 150,
        notifyOnEnter: true,
        notifyOnExit: false,
      },
    ]);
  });

  it('notifies onGeofenceEnter subscribers when the geofence task fires an Enter event', async () => {
    const { locationService, GEOFENCE_TASK_NAME } = loadModule();
    const geofenceCall = mockTaskManager.defineTask.mock.calls.find(
      ([name]) => name === GEOFENCE_TASK_NAME,
    );
    const geofenceTask = geofenceCall![1] as (body: any) => Promise<void>;
    const handler = jest.fn();
    const unsubscribe = locationService.onGeofenceEnter(handler);

    await geofenceTask({
      data: { eventType: Location.GeofencingEventType.Enter, region: { identifier: 'evt-9' } },
      error: null,
    });
    unsubscribe();
    await geofenceTask({
      data: { eventType: Location.GeofencingEventType.Enter, region: { identifier: 'evt-10' } },
      error: null,
    });

    expect(handler).toHaveBeenCalledTimes(1);
    expect(handler).toHaveBeenCalledWith('evt-9');
  });

  it('stops location updates and geofencing on stop()', async () => {
    mockPermissions(true, true);
    const { locationService, LOCATION_TASK_NAME, GEOFENCE_TASK_NAME } = loadModule();
    await locationService.init();
    mockLocation.hasStartedLocationUpdatesAsync.mockResolvedValue(true);
    mockLocation.hasStartedGeofencingAsync.mockResolvedValue(true);

    await locationService.stop();

    expect(mockLocation.stopGeofencingAsync).toHaveBeenCalledWith(GEOFENCE_TASK_NAME);
    expect(mockLocation.stopLocationUpdatesAsync).toHaveBeenCalledWith(LOCATION_TASK_NAME);
  });
});
