import { act, renderHook, waitFor } from '@testing-library/react-native';
import { useWaitingRoom } from '../useWaitingRoom';
import {
  fetchPowChallenge,
  joinWaitingRoom,
  openWaitingRoomStream,
  solvePow,
  WaitingRoomError,
  WaitingRoomStreamHandlers,
} from '@/services/waitingRoom';

/**
 * Issue #1465 — useWaitingRoom.
 *
 * The network layer in `services/waitingRoom.ts` is mocked (challenge fetch,
 * PoW solve, join, SSE stream). `WaitingRoomError` is kept real so the hook's
 * error-message mapping is exercised. The stream mock captures the handlers
 * the hook registers so tests can push position / admitted / error / closed
 * events at will.
 */

jest.mock('@/services/waitingRoom', () => {
  const actual = jest.requireActual('@/services/waitingRoom');
  return {
    ...actual,
    fetchPowChallenge: jest.fn(),
    solvePow: jest.fn(),
    joinWaitingRoom: jest.fn(),
    openWaitingRoomStream: jest.fn(),
  };
});

const mockFetchChallenge = fetchPowChallenge as jest.MockedFunction<typeof fetchPowChallenge>;
const mockSolvePow = solvePow as jest.MockedFunction<typeof solvePow>;
const mockJoin = joinWaitingRoom as jest.MockedFunction<typeof joinWaitingRoom>;
const mockOpenStream = openWaitingRoomStream as jest.MockedFunction<typeof openWaitingRoomStream>;

const EVENT_ID = 'evt-1';
const CLIENT_ID = 'client-abc';

let streamHandlers: WaitingRoomStreamHandlers | undefined;
const mockCloseStream = jest.fn();

beforeEach(() => {
  jest.clearAllMocks();
  streamHandlers = undefined;

  mockFetchChallenge.mockResolvedValue({ challenge: 'chal-1', difficulty: 4, expires_in: 60 });
  mockSolvePow.mockReturnValue('nonce-1');
  mockJoin.mockResolvedValue({
    status: 'waiting',
    position: 42,
    queue_size: 100,
    estimated_wait_seconds: 120,
    grant_token: null,
  });
  mockOpenStream.mockImplementation((_eventId, _clientId, handlers) => {
    streamHandlers = handlers;
    return mockCloseStream;
  });
});

async function renderWaiting() {
  const hook = renderHook(() => useWaitingRoom(EVENT_ID, CLIENT_ID));
  await waitFor(() => expect(hook.result.current.phase).toBe('waiting'));
  return hook;
}

describe('useWaitingRoom', () => {
  it('stays idle and makes no requests without an event or client id', () => {
    const { result } = renderHook(() => useWaitingRoom('', CLIENT_ID));

    expect(result.current.phase).toBe('idle');
    expect(result.current.position).toBeNull();
    expect(mockFetchChallenge).not.toHaveBeenCalled();
  });

  it('joins the queue with the solved PoW and exposes the initial position', async () => {
    const { result } = await renderWaiting();

    expect(mockFetchChallenge).toHaveBeenCalledWith(EVENT_ID);
    expect(mockSolvePow).toHaveBeenCalledWith('chal-1', 4);
    expect(mockJoin).toHaveBeenCalledWith({
      event_id: EVENT_ID,
      client_id: CLIENT_ID,
      challenge: 'chal-1',
      nonce: 'nonce-1',
    });
    expect(result.current.position).toBe(42);
    expect(result.current.queueSize).toBe(100);
    expect(result.current.estimatedWaitSeconds).toBe(120);
    expect(result.current.grantToken).toBeNull();
    expect(mockOpenStream).toHaveBeenCalledWith(EVENT_ID, CLIENT_ID, expect.any(Object));
  });

  it('updates the position as stream events arrive', async () => {
    const { result } = await renderWaiting();

    act(() => {
      streamHandlers!.onPosition(30, 95, 90);
    });
    expect(result.current).toMatchObject({
      phase: 'waiting',
      position: 30,
      queueSize: 95,
      estimatedWaitSeconds: 90,
    });

    act(() => {
      streamHandlers!.onPosition(1, 60, 5);
    });
    expect(result.current.position).toBe(1);
    expect(result.current.estimatedWaitSeconds).toBe(5);
  });

  it('transitions to admitted with the grant token, and the following close is not an error', async () => {
    const { result } = await renderWaiting();

    act(() => {
      streamHandlers!.onAdmitted('grant-xyz');
      // The service always fires onClosed right after onAdmitted.
      streamHandlers!.onClosed();
    });

    expect(result.current).toMatchObject({
      phase: 'admitted',
      position: null,
      estimatedWaitSeconds: 0,
      grantToken: 'grant-xyz',
      errorMessage: null,
    });
  });

  it('goes straight to admitted without opening a stream when join admits immediately', async () => {
    mockJoin.mockResolvedValue({
      status: 'admitted',
      position: null,
      queue_size: 3,
      estimated_wait_seconds: 0,
      grant_token: 'grant-now',
    });

    const { result } = renderHook(() => useWaitingRoom(EVENT_ID, CLIENT_ID));

    await waitFor(() => expect(result.current.phase).toBe('admitted'));
    expect(result.current.grantToken).toBe('grant-now');
    expect(result.current.queueSize).toBe(3);
    expect(mockOpenStream).not.toHaveBeenCalled();
  });

  it('surfaces a service error in errorMessage', async () => {
    mockFetchChallenge.mockRejectedValue(new WaitingRoomError('Waiting room is closed', 410));

    const { result } = renderHook(() => useWaitingRoom(EVENT_ID, CLIENT_ID));

    await waitFor(() => expect(result.current.phase).toBe('error'));
    expect(result.current.errorMessage).toBe('Waiting room is closed');
    expect(mockJoin).not.toHaveBeenCalled();
  });

  it('surfaces errors pushed on the position stream', async () => {
    const { result } = await renderWaiting();

    act(() => {
      streamHandlers!.onError('Queue service unavailable');
    });
    expect(result.current.phase).toBe('error');
    expect(result.current.errorMessage).toBe('Queue service unavailable');
  });

  it('treats a stream close while still waiting as a lost connection', async () => {
    const { result } = await renderWaiting();

    act(() => {
      streamHandlers!.onClosed();
    });

    expect(result.current.phase).toBe('error');
    expect(result.current.errorMessage).toMatch(/lost connection/i);
  });

  it('retry() closes the current stream and rejoins the queue', async () => {
    const { result } = await renderWaiting();
    act(() => {
      streamHandlers!.onError('boom');
    });
    expect(result.current.phase).toBe('error');

    act(() => {
      result.current.retry();
    });

    expect(mockCloseStream).toHaveBeenCalled();
    await waitFor(() => expect(result.current.phase).toBe('waiting'));
    expect(mockFetchChallenge).toHaveBeenCalledTimes(2);
    expect(mockJoin).toHaveBeenCalledTimes(2);
    expect(result.current.errorMessage).toBeNull();
  });

  it('closes the position stream on unmount and ignores late events', async () => {
    const { result, unmount } = await renderWaiting();
    const handlers = streamHandlers!;

    unmount();

    expect(mockCloseStream).toHaveBeenCalledTimes(1);
    // Late events after unmount must not throw or update state.
    expect(() => handlers.onPosition(5, 10, 10)).not.toThrow();
    expect(result.current.position).toBe(42);
  });

  it('does not open a stream if unmounted before the join resolves', async () => {
    let resolveJoin: (value: Awaited<ReturnType<typeof joinWaitingRoom>>) => void = () => {};
    mockJoin.mockImplementation(
      () =>
        new Promise((resolve) => {
          resolveJoin = resolve;
        })
    );

    const { result, unmount } = renderHook(() => useWaitingRoom(EVENT_ID, CLIENT_ID));
    await waitFor(() => expect(result.current.phase).toBe('joining'));

    unmount();
    await act(async () => {
      resolveJoin({
        status: 'waiting',
        position: 1,
        queue_size: 1,
        estimated_wait_seconds: 1,
        grant_token: null,
      });
    });

    expect(mockOpenStream).not.toHaveBeenCalled();
  });
});
