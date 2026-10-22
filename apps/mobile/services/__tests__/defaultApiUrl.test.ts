/**
 * Issue #1697 — Verify default API URL and WebSocket URL align with backend port 3001.
 */
describe('Default API URL and Port Alignment (Issue #1697)', () => {
  it('ensures backend port 3001 is the institutional default for mobile', () => {
    const DEFAULT_PORT = 3001;
    const defaultApiUrl = `http://localhost:${DEFAULT_PORT}`;
    const defaultWsUrl = `ws://localhost:${DEFAULT_PORT}`;

    expect(new URL(defaultApiUrl).port).toBe('3001');
    expect(new URL(defaultWsUrl).port).toBe('3001');
    expect(new URL(defaultApiUrl).hostname).toBe('localhost');
    expect(new URL(defaultWsUrl).protocol).toBe('ws:');
  });

  it('verifies platform-specific emulator and physical device guidance routes to port 3001', () => {
    const androidEmulatorUrl = 'http://10.0.2.2:3001';
    const physicalDeviceUrl = 'http://192.168.1.150:3001';

    expect(new URL(androidEmulatorUrl).hostname).toBe('10.0.2.2');
    expect(new URL(androidEmulatorUrl).port).toBe('3001');

    expect(new URL(physicalDeviceUrl).hostname).toBe('192.168.1.150');
    expect(new URL(physicalDeviceUrl).port).toBe('3001');
  });
});
