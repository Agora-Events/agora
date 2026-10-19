import React from 'react';
import { render, fireEvent } from '@testing-library/react-native';
import WaitingRoomScreen from '../waiting-room';
import { useWaitingRoom } from '@/hooks/useWaitingRoom';
import { useAuth } from '@/hooks/useAuth';

/**
 * Issue #1187 acceptance criteria covered here:
 *   - A dedicated countdown / waiting room screen exists.
 *   - It shows the live queue position and estimated wait.
 *   - On receiving the signed access grant it auto-redirects to checkout.
 *
 * Network orchestration (PoW + SSE) is mocked via `useWaitingRoom`; the hook
 * and service layers have their own coverage.
 */

let mockParams: Record<string, string> = {
  eventId: '550e8400-e29b-41d4-a716-446655440000',
  eventTitle: 'Stellar Meridian 2026',
};
const mockReplace = jest.fn();
const mockRetry = jest.fn();

jest.mock('expo-router', () => ({
  useLocalSearchParams: () => mockParams,
  useRouter: () => ({ replace: mockReplace, push: jest.fn() }),
}));

jest.mock('@/hooks/useAuth');
jest.mock('@/hooks/useWaitingRoom');

const mockUseAuth = useAuth as jest.MockedFunction<typeof useAuth>;
const mockUseWaitingRoom = useWaitingRoom as jest.MockedFunction<typeof useWaitingRoom>;

function waitingState(overrides: Partial<ReturnType<typeof useWaitingRoom>> = {}) {
  return {
    phase: 'waiting' as const,
    position: 142,
    queueSize: 1000,
    estimatedWaitSeconds: 45,
    grantToken: null,
    errorMessage: null,
    retry: mockRetry,
    ...overrides,
  };
}

beforeEach(() => {
  jest.clearAllMocks();
  mockParams = {
    eventId: '550e8400-e29b-41d4-a716-446655440000',
    eventTitle: 'Stellar Meridian 2026',
  };
  mockUseAuth.mockReturnValue({
    token: 'mock-jwt-token-agora',
    user: { name: 'Agora User', email: 'user@example.com', walletAddress: 'GCLIENTWALLET' },
    isAuthenticated: true,
    login: jest.fn(),
    logout: jest.fn(),
    updateWalletAddress: jest.fn(),
  } as any);
  mockUseWaitingRoom.mockReturnValue(waitingState());
});

describe('WaitingRoomScreen', () => {
  it('shows the live queue position and estimated wait while waiting', () => {
    const { getByTestId, getByText } = render(<WaitingRoomScreen />);

    // JSX `#${position}` renders as a children array in React Native.
    expect(getByTestId('queue-position').props.children).toEqual(['#', 142]);
    expect(getByText('Estimated wait: 45s')).toBeTruthy();
    expect(getByText('1,000 people ahead of you')).toBeTruthy();
  });

  it('shows the human-verification state while solving the PoW challenge', () => {
    mockUseWaitingRoom.mockReturnValue(waitingState({ phase: 'solving', position: null }));

    const { getByText } = render(<WaitingRoomScreen />);
    expect(getByText('Verifying you are human...')).toBeTruthy();
  });

  it('shows an error with a retry button when joining fails', () => {
    mockUseWaitingRoom.mockReturnValue(
      waitingState({ phase: 'error', errorMessage: 'Proof-of-work solution is incorrect' })
    );

    const { getByText, getByTestId } = render(<WaitingRoomScreen />);
    expect(getByText('Could not join the queue')).toBeTruthy();
    expect(getByText('Proof-of-work solution is incorrect')).toBeTruthy();

    fireEvent.press(getByTestId('waiting-room-retry'));
    expect(mockRetry).toHaveBeenCalled();
  });

  it('auto-redirects to checkout when the signed grant token arrives', () => {
    mockUseWaitingRoom.mockReturnValue(
      waitingState({
        phase: 'admitted',
        position: null,
        estimatedWaitSeconds: 0,
        grantToken: 'eyJhbGciOiJIUzI1NiJ9.signed-grant',
      })
    );

    render(<WaitingRoomScreen />);

    expect(mockReplace).toHaveBeenCalledWith({
      pathname: '/checkout',
      params: {
        eventId: '550e8400-e29b-41d4-a716-446655440000',
        eventTitle: 'Stellar Meridian 2026',
        grantToken: 'eyJhbGciOiJIUzI1NiJ9.signed-grant',
      },
    });
  });
});
