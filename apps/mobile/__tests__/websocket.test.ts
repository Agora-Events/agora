import {
  InventorySocket,
  applyInventoryUpdate,
  backoffDelay,
  parseInventoryMessage,
  INITIAL_RECONNECT_DELAY_MS,
  MAX_RECONNECT_DELAY_MS,
  type WebSocketLike,
} from '../utils/websocket';

/**
 * Issue #1010 — the WebSocket inventory client.
 *
 * Exercised through an injected socket double rather than a real connection,
 * so the reconnect/backoff behaviour is deterministic and no network is
 * touched in CI.
 */

class FakeSocket implements WebSocketLike {
  onopen: ((event: unknown) => void) | null = null;
  onmessage: ((event: { data: unknown }) => void) | null = null;
  onerror: ((event: unknown) => void) | null = null;
  onclose: ((event: unknown) => void) | null = null;

  sent: string[] = [];
  closed = false;

  send(data: string): void {
    this.sent.push(data);
  }

  close(): void {
    this.closed = true;
  }

  /** Test helpers that drive the socket's lifecycle. */
  open(): void {
    this.onopen?.({});
  }

  emit(data: unknown): void {
    this.onmessage?.({ data });
  }

  drop(): void {
    this.onclose?.({});
  }
}

describe('parseInventoryMessage', () => {
  it('parses the backend purchase-delta shape', () => {
    const raw = JSON.stringify({
      event_id: 'evt-1',
      ticket_tier_id: 'tier-ga',
      quantity: 2,
      amount: 300,
      currency: 'USDC',
      purchased_at: '2026-07-29T10:00:00Z',
    });

    expect(parseInventoryMessage(raw)).toEqual({
      eventId: 'evt-1',
      tierId: 'tier-ga',
      soldDelta: 2,
    });
  });

  it('parses an absolute remaining count', () => {
    const raw = JSON.stringify({ eventId: 'evt-1', tierId: 'tier-vip', remaining: 7 });

    expect(parseInventoryMessage(raw)).toEqual({
      eventId: 'evt-1',
      tierId: 'tier-vip',
      remaining: 7,
    });
  });

  it('accepts snake_case and camelCase identifiers alike', () => {
    const snake = parseInventoryMessage(
      JSON.stringify({ event_id: 'e', ticket_tier_id: 't', remaining_tickets: 3 }),
    );
    const camel = parseInventoryMessage(
      JSON.stringify({ eventId: 'e', tierId: 't', remainingTickets: 3 }),
    );

    expect(snake).toEqual(camel);
  });

  it('clamps negative counts and truncates fractions', () => {
    expect(
      parseInventoryMessage(JSON.stringify({ eventId: 'e', tierId: 't', remaining: -5 })),
    ).toEqual({ eventId: 'e', tierId: 't', remaining: 0 });

    expect(
      parseInventoryMessage(JSON.stringify({ eventId: 'e', tierId: 't', remaining: 4.9 })),
    ).toEqual({ eventId: 'e', tierId: 't', remaining: 4 });
  });

  it('returns null for anything unusable', () => {
    expect(parseInventoryMessage('not json')).toBeNull();
    expect(parseInventoryMessage(JSON.stringify({ hello: 'world' }))).toBeNull();
    expect(parseInventoryMessage(JSON.stringify({ eventId: 'e' }))).toBeNull();
    // Identifies a tier but carries neither a count nor a delta.
    expect(parseInventoryMessage(JSON.stringify({ eventId: 'e', tierId: 't' }))).toBeNull();
    expect(parseInventoryMessage(null)).toBeNull();
    expect(parseInventoryMessage(42)).toBeNull();
  });
});

describe('applyInventoryUpdate', () => {
  it('prefers an authoritative remaining count over a delta', () => {
    expect(
      applyInventoryUpdate(100, { eventId: 'e', tierId: 't', remaining: 5, soldDelta: 2 }),
    ).toBe(5);
  });

  it('subtracts a delta from the known count', () => {
    expect(applyInventoryUpdate(10, { eventId: 'e', tierId: 't', soldDelta: 3 })).toBe(7);
  });

  it('never goes below zero', () => {
    expect(applyInventoryUpdate(2, { eventId: 'e', tierId: 't', soldDelta: 9 })).toBe(0);
  });

  it('cannot apply a delta without a known starting count', () => {
    expect(applyInventoryUpdate(undefined, { eventId: 'e', tierId: 't', soldDelta: 3 })).toBeUndefined();
  });

  it('returns the current value when the update carries nothing', () => {
    expect(applyInventoryUpdate(8, { eventId: 'e', tierId: 't' })).toBe(8);
  });
});

describe('backoffDelay', () => {
  it('doubles each attempt starting from the initial delay', () => {
    expect(backoffDelay(0)).toBe(INITIAL_RECONNECT_DELAY_MS);
    expect(backoffDelay(1)).toBe(INITIAL_RECONNECT_DELAY_MS * 2);
    expect(backoffDelay(2)).toBe(INITIAL_RECONNECT_DELAY_MS * 4);
    expect(backoffDelay(3)).toBe(INITIAL_RECONNECT_DELAY_MS * 8);
  });

  it('caps at the maximum delay', () => {
    expect(backoffDelay(20)).toBe(MAX_RECONNECT_DELAY_MS);
  });

  it('treats a negative attempt as the first', () => {
    expect(backoffDelay(-1)).toBe(INITIAL_RECONNECT_DELAY_MS);
  });
});

describe('InventorySocket', () => {
  let sockets: FakeSocket[];
  let factory: (url: string) => WebSocketLike;
  let urls: string[];

  beforeEach(() => {
    jest.useFakeTimers();
    sockets = [];
    urls = [];
    factory = (url: string) => {
      urls.push(url);
      const socket = new FakeSocket();
      sockets.push(socket);
      return socket;
    };
  });

  afterEach(() => {
    jest.useRealTimers();
  });

  function connect(overrides: Record<string, unknown> = {}) {
    const onUpdate = jest.fn();
    const onStatusChange = jest.fn();
    const socket = new InventorySocket({
      eventId: 'evt-1',
      url: 'ws://test/ws',
      socketFactory: factory,
      onUpdate,
      onStatusChange,
      ...overrides,
    });
    socket.connect();
    return { socket, onUpdate, onStatusChange };
  }

  it('registers for the event room once open', () => {
    const { onStatusChange } = connect();

    expect(sockets).toHaveLength(1);
    sockets[0].open();

    expect(sockets[0].sent).toEqual([
      JSON.stringify({ type: 'subscribe', eventId: 'evt-1' }),
    ]);
    expect(onStatusChange).toHaveBeenCalledWith('connecting');
    expect(onStatusChange).toHaveBeenCalledWith('connected');
  });

  it('appends the auth token to the URL', () => {
    connect({ token: 'jwt-123' });
    expect(urls[0]).toBe('ws://test/ws?token=jwt-123');
  });

  it('forwards updates for the subscribed event', () => {
    const { onUpdate } = connect();
    sockets[0].open();

    sockets[0].emit(JSON.stringify({ event_id: 'evt-1', ticket_tier_id: 'tier-ga', quantity: 3 }));

    expect(onUpdate).toHaveBeenCalledWith({
      eventId: 'evt-1',
      tierId: 'tier-ga',
      soldDelta: 3,
    });
  });

  it('ignores updates for other events', () => {
    const { onUpdate } = connect();
    sockets[0].open();

    sockets[0].emit(JSON.stringify({ event_id: 'evt-2', ticket_tier_id: 'tier-ga', quantity: 3 }));

    expect(onUpdate).not.toHaveBeenCalled();
  });

  it('ignores malformed frames without crashing', () => {
    const { onUpdate } = connect();
    sockets[0].open();

    expect(() => {
      sockets[0].emit('not json');
      sockets[0].emit(JSON.stringify({ nothing: true }));
      sockets[0].emit(undefined);
    }).not.toThrow();

    expect(onUpdate).not.toHaveBeenCalled();
  });

  it('reconnects with exponential backoff after a drop', () => {
    const { onStatusChange } = connect();
    sockets[0].open();

    sockets[0].drop();
    expect(onStatusChange).toHaveBeenCalledWith('reconnecting');
    expect(sockets).toHaveLength(1);

    // First retry after the initial delay.
    jest.advanceTimersByTime(INITIAL_RECONNECT_DELAY_MS);
    expect(sockets).toHaveLength(2);

    // Second drop waits twice as long.
    sockets[1].drop();
    jest.advanceTimersByTime(INITIAL_RECONNECT_DELAY_MS);
    expect(sockets).toHaveLength(2);
    jest.advanceTimersByTime(INITIAL_RECONNECT_DELAY_MS);
    expect(sockets).toHaveLength(3);
  });

  it('re-subscribes after reconnecting', () => {
    connect();
    sockets[0].open();
    sockets[0].drop();

    jest.advanceTimersByTime(INITIAL_RECONNECT_DELAY_MS);
    sockets[1].open();

    expect(sockets[1].sent).toEqual([
      JSON.stringify({ type: 'subscribe', eventId: 'evt-1' }),
    ]);
  });

  it('resets the backoff after a successful reconnect', () => {
    connect();
    sockets[0].open();

    sockets[0].drop();
    jest.advanceTimersByTime(INITIAL_RECONNECT_DELAY_MS);
    sockets[1].open(); // success resets the counter

    sockets[1].drop();
    // Back to the initial delay rather than double.
    jest.advanceTimersByTime(INITIAL_RECONNECT_DELAY_MS);
    expect(sockets).toHaveLength(3);
  });

  it('stops reconnecting once closed', () => {
    const { socket } = connect();
    sockets[0].open();

    socket.close();
    expect(sockets[0].closed).toBe(true);
    expect(socket.getStatus()).toBe('closed');

    jest.advanceTimersByTime(MAX_RECONNECT_DELAY_MS * 2);
    expect(sockets).toHaveLength(1);
  });

  it('does not reconnect for a drop that arrives after close', () => {
    const { socket } = connect();
    sockets[0].open();
    const first = sockets[0];

    socket.close();
    first.drop(); // listener was detached, so this is a no-op

    jest.advanceTimersByTime(MAX_RECONNECT_DELAY_MS);
    expect(sockets).toHaveLength(1);
  });

  it('is safe to close more than once', () => {
    const { socket } = connect();
    sockets[0].open();

    expect(() => {
      socket.close();
      socket.close();
    }).not.toThrow();
  });

  it('retries when the socket constructor throws', () => {
    let attempts = 0;
    const throwingFactory = (url: string) => {
      attempts += 1;
      if (attempts === 1) throw new Error('no network');
      return factory(url);
    };

    const socket = new InventorySocket({
      eventId: 'evt-1',
      url: 'ws://test/ws',
      socketFactory: throwingFactory,
    });
    socket.connect();

    expect(sockets).toHaveLength(0);
    jest.advanceTimersByTime(INITIAL_RECONNECT_DELAY_MS);
    expect(sockets).toHaveLength(1);
  });

  it('ignores a second connect while already connected', () => {
    const { socket } = connect();
    socket.connect();
    expect(sockets).toHaveLength(1);
  });
});
