/**
 * useBiometricAuth.test.ts — Issue #1179: Offline Ticket Vault
 *
 * Tests for useBiometricAuth hook covering:
 *   - Initial state (checking → idle or unavailable)
 *   - Unavailable state (no hardware, not enrolled)
 *   - Successful authentication
 *   - User cancellation → failed state (not silently unlocked)
 *   - Lockout → failed with distinct message
 *   - Runtime error → error state
 *   - re-prompt after reset()
 */

import { renderHook, act } from '@testing-library/react-native';
import { useBiometricAuth, BiometricState } from '../../hooks/useBiometricAuth';

// ── Mock expo-local-authentication ────────────────────────────────────────────

const mockHasHardware = jest.fn();
const mockIsEnrolled = jest.fn();
const mockSupportedTypes = jest.fn();
const mockAuthenticate = jest.fn();

jest.mock('expo-local-authentication', () => ({
  hasHardwareAsync: () => mockHasHardware(),
  isEnrolledAsync: () => mockIsEnrolled(),
  supportedAuthenticationTypesAsync: () => mockSupportedTypes(),
  authenticateAsync: (opts: unknown) => mockAuthenticate(opts),
  AuthenticationType: { FINGERPRINT: 1, FACIAL_RECOGNITION: 2, IRIS: 3 },
}));

beforeEach(() => {
  jest.clearAllMocks();
  // Default: hardware available, enrolled
  mockHasHardware.mockResolvedValue(true);
  mockIsEnrolled.mockResolvedValue(true);
  mockSupportedTypes.mockResolvedValue([2]); // FACIAL_RECOGNITION
  mockAuthenticate.mockResolvedValue({ success: true });
});

// ── Helper: wait for all async effects to settle ──────────────────────────────

async function flushEffects() {
  await act(async () => {
    await Promise.resolve();
  });
}

// ── Initial state ─────────────────────────────────────────────────────────────

describe('initial state', () => {
  it('starts in checking state', () => {
    const { result } = renderHook(() => useBiometricAuth());
    expect(result.current.state).toBe('checking');
  });

  it('transitions to idle when hardware is available and enrolled', async () => {
    const { result } = renderHook(() => useBiometricAuth());
    await flushEffects();
    expect(result.current.state).toBe('idle');
    expect(result.current.errorMessage).toBeNull();
  });

  it('sets capabilities on load', async () => {
    const { result } = renderHook(() => useBiometricAuth());
    await flushEffects();
    expect(result.current.capabilities).not.toBeNull();
    expect(result.current.capabilities?.hasHardware).toBe(true);
    expect(result.current.capabilities?.isEnrolled).toBe(true);
  });
});

// ── Unavailable states ────────────────────────────────────────────────────────

describe('unavailable: no hardware', () => {
  beforeEach(() => {
    mockHasHardware.mockResolvedValue(false);
    mockIsEnrolled.mockResolvedValue(false);
  });

  it('transitions to unavailable', async () => {
    const { result } = renderHook(() => useBiometricAuth());
    await flushEffects();
    expect(result.current.state).toBe('unavailable');
  });

  it('sets an errorMessage about hardware', async () => {
    const { result } = renderHook(() => useBiometricAuth());
    await flushEffects();
    expect(result.current.errorMessage).toContain('not support');
  });
});

describe('unavailable: not enrolled', () => {
  beforeEach(() => {
    mockHasHardware.mockResolvedValue(true);
    mockIsEnrolled.mockResolvedValue(false);
  });

  it('transitions to unavailable', async () => {
    const { result } = renderHook(() => useBiometricAuth());
    await flushEffects();
    expect(result.current.state).toBe('unavailable');
  });

  it('sets an errorMessage about enrollment', async () => {
    const { result } = renderHook(() => useBiometricAuth());
    await flushEffects();
    expect(result.current.errorMessage).toMatch(/enroll/i);
  });
});

// ── Successful authentication ─────────────────────────────────────────────────

describe('successful authentication', () => {
  it('transitions to success after authenticate()', async () => {
    const { result } = renderHook(() => useBiometricAuth());
    await flushEffects(); // idle

    await act(async () => {
      await result.current.authenticate();
    });

    expect(result.current.state).toBe('success');
    expect(result.current.errorMessage).toBeNull();
  });

  it('does not silently fall through on success — state is explicitly "success"', async () => {
    const { result } = renderHook(() => useBiometricAuth());
    await flushEffects();
    await act(async () => { await result.current.authenticate(); });

    // The gate check in [id].tsx is `if (biometric.state !== 'success')`.
    // This test verifies the state is not some truthy ambiguous value.
    expect(result.current.state).toBe<BiometricState>('success');
  });
});

// ── User cancellation ─────────────────────────────────────────────────────────

describe('user cancellation', () => {
  beforeEach(() => {
    mockAuthenticate.mockResolvedValue({ success: false, error: 'user_cancel' });
  });

  it('transitions to failed (not silently unlocked)', async () => {
    const { result } = renderHook(() => useBiometricAuth());
    await flushEffects();
    await act(async () => { await result.current.authenticate(); });

    expect(result.current.state).toBe('failed');
    // Must NOT be 'success' — never silently fall through
    expect(result.current.state).not.toBe('success');
  });

  it('sets a user-facing error message on cancel', async () => {
    const { result } = renderHook(() => useBiometricAuth());
    await flushEffects();
    await act(async () => { await result.current.authenticate(); });

    expect(result.current.errorMessage).not.toBeNull();
    expect(result.current.errorMessage).toMatch(/cancel/i);
  });
});

// ── Lockout ───────────────────────────────────────────────────────────────────

describe('lockout', () => {
  beforeEach(() => {
    mockAuthenticate.mockResolvedValue({ success: false, error: 'lockout' });
  });

  it('transitions to failed on lockout', async () => {
    const { result } = renderHook(() => useBiometricAuth());
    await flushEffects();
    await act(async () => { await result.current.authenticate(); });
    expect(result.current.state).toBe('failed');
  });

  it('lockout message is distinct from cancel message', async () => {
    const { result } = renderHook(() => useBiometricAuth());
    await flushEffects();
    await act(async () => { await result.current.authenticate(); });
    const lockoutMsg = result.current.errorMessage;

    mockAuthenticate.mockResolvedValue({ success: false, error: 'user_cancel' });
    const { result: result2 } = renderHook(() => useBiometricAuth());
    await flushEffects();
    await act(async () => { await result2.current.authenticate(); });
    const cancelMsg = result2.current.errorMessage;

    expect(lockoutMsg).not.toBe(cancelMsg);
  });
});

// ── Runtime error ─────────────────────────────────────────────────────────────

describe('runtime error', () => {
  it('transitions to error state when authenticateAsync throws', async () => {
    mockAuthenticate.mockRejectedValue(new Error('Biometric hardware fault'));
    const { result } = renderHook(() => useBiometricAuth());
    await flushEffects();
    await act(async () => { await result.current.authenticate(); });
    expect(result.current.state).toBe('error');
    expect(result.current.errorMessage).toContain('fault');
  });

  it('transitions to error state when capability check throws', async () => {
    mockHasHardware.mockRejectedValue(new Error('Hardware query failed'));
    const { result } = renderHook(() => useBiometricAuth());
    await flushEffects();
    expect(result.current.state).toBe('error');
  });
});

// ── reset() ───────────────────────────────────────────────────────────────────

describe('reset', () => {
  it('resets state to idle after failed authentication', async () => {
    mockAuthenticate.mockResolvedValue({ success: false, error: 'user_cancel' });
    const { result } = renderHook(() => useBiometricAuth());
    await flushEffects();
    await act(async () => { await result.current.authenticate(); });
    expect(result.current.state).toBe('failed');

    act(() => { result.current.reset(); });
    expect(result.current.state).toBe('idle');
    expect(result.current.errorMessage).toBeNull();
  });

  it('allows re-authentication after reset', async () => {
    mockAuthenticate
      .mockResolvedValueOnce({ success: false, error: 'user_cancel' })
      .mockResolvedValueOnce({ success: true });

    const { result } = renderHook(() => useBiometricAuth());
    await flushEffects();
    await act(async () => { await result.current.authenticate(); });
    expect(result.current.state).toBe('failed');

    act(() => { result.current.reset(); });
    await act(async () => { await result.current.authenticate(); });
    expect(result.current.state).toBe('success');
  });
});
