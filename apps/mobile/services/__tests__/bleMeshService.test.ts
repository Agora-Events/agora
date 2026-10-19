import { BleMeshService } from '../bleMeshService';
import NetInfo from '@react-native-community/netinfo';

/**
 * Covers the gossip merge + gateway-flush logic in isolation. The BLE
 * central (`react-native-ble-plx`) is mocked entirely so these tests run
 * without any native module or real network access.
 */

jest.mock('react-native-ble-plx', () => ({
  BleManager: jest.fn().mockImplementation(() => ({
    startDeviceScan: jest.fn(),
  })),
}));

jest.mock('@react-native-community/netinfo', () => ({
  __esModule: true,
  default: { fetch: jest.fn() },
}));


beforeEach(() => {
  jest.restoreAllMocks();
  global.fetch = jest.fn();
});

describe('BleMeshService.recordCheckIn', () => {
  it('adds a pending record that increments pendingCount', () => {
    const mesh = new BleMeshService('scanner-1');
    expect(mesh.pendingCount).toBe(0);

    mesh.recordCheckIn('ticket-1', 'gate-1', 'approved');

    expect(mesh.pendingCount).toBe(1);
  });
});

describe('BleMeshService gateway flush (via gossip tick)', () => {
  it('does not attempt a flush when offline', async () => {
    (NetInfo.fetch as jest.Mock).mockResolvedValue({ isConnected: false, isInternetReachable: false });

    const mesh = new BleMeshService('scanner-1');
    mesh.recordCheckIn('ticket-1', 'gate-1', 'approved');
    await (mesh as any).tryFlushToGateway('gate-1');

    expect(global.fetch).not.toHaveBeenCalled();
    expect(mesh.pendingCount).toBe(1);
  });

  it('flushes and clears accepted records when online', async () => {
    (NetInfo.fetch as jest.Mock).mockResolvedValue({ isConnected: true, isInternetReachable: true });

    const mesh = new BleMeshService('scanner-1');
    mesh.recordCheckIn('ticket-1', 'gate-1', 'approved');
    const recordId = Array.from((mesh as any).log.keys())[0];

    (global.fetch as jest.Mock).mockResolvedValueOnce({
      ok: true,
      json: async () => ({ acceptedRecordIds: [recordId] }),
    });

    await (mesh as any).tryFlushToGateway('gate-1');

    expect(global.fetch).toHaveBeenCalledWith(
      expect.stringContaining('/api/gate/checkins/batch'),
      expect.objectContaining({ method: 'POST' }),
    );
    expect(mesh.pendingCount).toBe(0);
  });
});
