import { renderHook, act } from '@testing-library/react-native';
import { useLiveTicketInventory } from '../hooks/useLiveTicketInventory';
import { InventorySocket } from '../utils/websocket';
import type { TicketTierOption } from '../types/checkout';

/**
 * Issue #1010 — the hook that folds live updates into a tier list.
 *
 * `InventorySocket` is mocked so the test drives `onUpdate` directly; the
 * socket's own reconnect behaviour is covered in `websocket.test.ts`.
 */

jest.mock('../utils/websocket', () => {
  const actual = jest.requireActual('../utils/websocket');
  return {
    ...actual,
    InventorySocket: jest.fn(),
  };
});

const MockedInventorySocket = InventorySocket as unknown as jest.Mock;

const TIERS: TicketTierOption[] = [
  { id: 'tier-ga', name: 'General Admission', priceUsdc: 150, remaining: 340 },
  { id: 'tier-vip', name: 'VIP', priceUsdc: 450, remaining: 12 },
];

interface Handles {
  onUpdate: (update: { eventId: string; tierId: string; remaining?: number; soldDelta?: number }) => void;
  onStatusChange: (status: string) => void;
  close: jest.Mock;
  connect: jest.Mock;
}

let handles: Handles;

beforeEach(() => {
  MockedInventorySocket.mockReset();
  MockedInventorySocket.mockImplementation((options: Record<string, never>) => {
    const opts = options as unknown as {
      onUpdate: Handles['onUpdate'];
      onStatusChange: Handles['onStatusChange'];
    };
    handles = {
      onUpdate: opts.onUpdate,
      onStatusChange: opts.onStatusChange,
      connect: jest.fn(),
      close: jest.fn(),
    };
    return { connect: handles.connect, close: handles.close };
  });
});

describe('useLiveTicketInventory', () => {
  it('returns the supplied tiers untouched before any update', () => {
    const { result } = renderHook(() =>
      useLiveTicketInventory({ eventId: 'evt-1', tiers: TIERS }),
    );

    expect(result.current.tiers).toEqual(TIERS);
    expect(result.current.isLive).toBe(false);
  });

  it('opens a socket on mount and closes it on unmount', () => {
    const { unmount } = renderHook(() =>
      useLiveTicketInventory({ eventId: 'evt-1', tiers: TIERS }),
    );

    expect(handles.connect).toHaveBeenCalledTimes(1);

    unmount();
    expect(handles.close).toHaveBeenCalledTimes(1);
  });

  it('does not connect when disabled', () => {
    renderHook(() => useLiveTicketInventory({ eventId: 'evt-1', tiers: TIERS, enabled: false }));

    expect(MockedInventorySocket).not.toHaveBeenCalled();
  });

  it('applies an absolute remaining count to the matching tier only', () => {
    const { result } = renderHook(() =>
      useLiveTicketInventory({ eventId: 'evt-1', tiers: TIERS }),
    );

    act(() => {
      handles.onUpdate({ eventId: 'evt-1', tierId: 'tier-vip', remaining: 3 });
    });

    expect(result.current.tiers[0]).toEqual(TIERS[0]);
    expect(result.current.tiers[1].remaining).toBe(3);
  });

  it('subtracts a purchase delta from the seeded count', () => {
    const { result } = renderHook(() =>
      useLiveTicketInventory({ eventId: 'evt-1', tiers: TIERS }),
    );

    act(() => {
      handles.onUpdate({ eventId: 'evt-1', tierId: 'tier-ga', soldDelta: 40 });
    });

    expect(result.current.tiers[0].remaining).toBe(300);
  });

  it('accumulates successive deltas', () => {
    const { result } = renderHook(() =>
      useLiveTicketInventory({ eventId: 'evt-1', tiers: TIERS }),
    );

    act(() => {
      handles.onUpdate({ eventId: 'evt-1', tierId: 'tier-vip', soldDelta: 5 });
    });
    act(() => {
      handles.onUpdate({ eventId: 'evt-1', tierId: 'tier-vip', soldDelta: 4 });
    });

    expect(result.current.tiers[1].remaining).toBe(3);
  });

  it('drives a tier to sold out and never below zero', () => {
    const { result } = renderHook(() =>
      useLiveTicketInventory({ eventId: 'evt-1', tiers: TIERS }),
    );

    act(() => {
      handles.onUpdate({ eventId: 'evt-1', tierId: 'tier-vip', soldDelta: 999 });
    });

    expect(result.current.tiers[1].remaining).toBe(0);
  });

  it('preserves tier identity and order so the list does not reset', () => {
    const { result } = renderHook(() =>
      useLiveTicketInventory({ eventId: 'evt-1', tiers: TIERS }),
    );

    act(() => {
      handles.onUpdate({ eventId: 'evt-1', tierId: 'tier-ga', remaining: 100 });
    });

    expect(result.current.tiers.map((t) => t.id)).toEqual(['tier-ga', 'tier-vip']);
    expect(result.current.tiers[0].name).toBe('General Admission');
    expect(result.current.tiers[0].priceUsdc).toBe(150);
  });

  it('reports live status from the socket', () => {
    const { result } = renderHook(() =>
      useLiveTicketInventory({ eventId: 'evt-1', tiers: TIERS }),
    );

    act(() => {
      handles.onStatusChange('connected');
    });
    expect(result.current.isLive).toBe(true);

    act(() => {
      handles.onStatusChange('reconnecting');
    });
    expect(result.current.isLive).toBe(false);
  });

  it('ignores updates for a tier it does not know', () => {
    const { result } = renderHook(() =>
      useLiveTicketInventory({ eventId: 'evt-1', tiers: TIERS }),
    );

    act(() => {
      handles.onUpdate({ eventId: 'evt-1', tierId: 'tier-unknown', remaining: 1 });
    });

    expect(result.current.tiers).toEqual(TIERS);
  });
});
