/**
 * Real-time inventory WebSocket client (issue #1010).
 *
 * Wraps the platform `WebSocket` with the things every screen needs and none
 * of them should reimplement: room registration, exponential-backoff
 * reconnection, and parsing of inventory messages into a single normalised
 * shape.
 *
 * ## Message formats
 *
 * The Rust backend (`server/src/handlers/ws.rs`) broadcasts *purchase deltas*:
 *
 * ```json
 * { "event_id": "...", "ticket_tier_id": "...", "quantity": 2,
 *   "amount": 300.0, "currency": "USDC", "purchased_at": "..." }
 * ```
 *
 * The issue additionally specifies payloads carrying *updated remaining
 * counts*. Both are accepted: a delta decrements the local count, an absolute
 * count replaces it. That keeps this client working against the backend as it
 * exists today without needing a change when authoritative counts are added.
 */

/** A normalised inventory update, whichever wire format produced it. */
export interface InventoryUpdate {
  eventId: string;
  tierId: string;
  /** Authoritative remaining count, when the server sent one. */
  remaining?: number;
  /** Number of tickets just sold, when the server sent a delta. */
  soldDelta?: number;
}

export type ConnectionStatus =
  | 'idle'
  | 'connecting'
  | 'connected'
  | 'reconnecting'
  | 'closed';

export interface InventorySocketOptions {
  /** Event room to register for. Updates for other events are ignored. */
  eventId: string;
  /** Overrides the default endpoint; useful in tests and for staging. */
  url?: string;
  /** Bearer token — the backend rejects the upgrade without one. */
  token?: string;
  onUpdate?: (update: InventoryUpdate) => void;
  onStatusChange?: (status: ConnectionStatus) => void;
  /** Injectable for tests; defaults to the global `WebSocket`. */
  socketFactory?: (url: string) => WebSocketLike;
}

/**
 * The slice of the `WebSocket` API this client uses. Declared structurally so
 * tests can supply a double without a DOM lib dependency.
 */
export interface WebSocketLike {
  send(data: string): void;
  close(code?: number, reason?: string): void;
  onopen: ((event: unknown) => void) | null;
  onmessage: ((event: { data: unknown }) => void) | null;
  onerror: ((event: unknown) => void) | null;
  onclose: ((event: unknown) => void) | null;
}

const DEFAULT_WS_URL =
  process.env.EXPO_PUBLIC_WS_URL ?? 'ws://localhost:3001/api/v1/ws/purchases';

/** Backoff schedule: 1s, 2s, 4s, 8s, 16s, then hold at 30s. */
export const INITIAL_RECONNECT_DELAY_MS = 1000;
export const MAX_RECONNECT_DELAY_MS = 30000;

/**
 * Delay before reconnect attempt `attempt` (0-based), doubling each time and
 * capped so a long outage doesn't push the retry interval to hours.
 */
export function backoffDelay(attempt: number): number {
  const exponential = INITIAL_RECONNECT_DELAY_MS * 2 ** Math.max(0, attempt);
  return Math.min(exponential, MAX_RECONNECT_DELAY_MS);
}

function asFiniteNumber(value: unknown): number | undefined {
  return typeof value === 'number' && Number.isFinite(value) ? value : undefined;
}

function asNonEmptyString(value: unknown): string | undefined {
  return typeof value === 'string' && value.length > 0 ? value : undefined;
}

/**
 * Parses a raw frame into an `InventoryUpdate`.
 *
 * Returns null for anything unrecognised — malformed JSON, heartbeats, or
 * messages for other channels — so callers can ignore them without
 * distinguishing "broken" from "not for me". Exported for direct testing.
 */
export function parseInventoryMessage(raw: unknown): InventoryUpdate | null {
  if (typeof raw !== 'string') return null;

  let payload: unknown;
  try {
    payload = JSON.parse(raw);
  } catch {
    return null;
  }

  if (typeof payload !== 'object' || payload === null) return null;
  const data = payload as Record<string, unknown>;

  // Accept snake_case (Rust backend) and camelCase (JS producers) alike.
  const eventId = asNonEmptyString(data.event_id) ?? asNonEmptyString(data.eventId);
  const tierId =
    asNonEmptyString(data.ticket_tier_id) ??
    asNonEmptyString(data.tierId) ??
    asNonEmptyString(data.tier_id);

  if (!eventId || !tierId) return null;

  const remaining =
    asFiniteNumber(data.remaining) ??
    asFiniteNumber(data.remaining_tickets) ??
    asFiniteNumber(data.remainingTickets);

  const soldDelta = asFiniteNumber(data.quantity) ?? asFiniteNumber(data.sold);

  // A message that identifies a tier but carries neither a count nor a delta
  // tells us nothing actionable.
  if (remaining === undefined && soldDelta === undefined) return null;

  return {
    eventId,
    tierId,
    ...(remaining !== undefined ? { remaining: Math.max(0, Math.trunc(remaining)) } : {}),
    ...(soldDelta !== undefined ? { soldDelta: Math.max(0, Math.trunc(soldDelta)) } : {}),
  };
}

/**
 * Applies an update to a known remaining count.
 *
 * An absolute `remaining` wins over a delta, since it is authoritative. The
 * result never goes below zero. Returns `current` unchanged when the update
 * carries nothing usable.
 */
export function applyInventoryUpdate(
  current: number | undefined,
  update: InventoryUpdate,
): number | undefined {
  if (update.remaining !== undefined) return update.remaining;
  if (update.soldDelta !== undefined && current !== undefined) {
    return Math.max(0, current - update.soldDelta);
  }
  return current;
}

function buildUrl(baseUrl: string, token?: string): string {
  if (!token) return baseUrl;
  const separator = baseUrl.includes('?') ? '&' : '?';
  return `${baseUrl}${separator}token=${encodeURIComponent(token)}`;
}

/**
 * A reconnecting inventory socket scoped to one event.
 *
 * Lifecycle is explicit — `connect()` / `close()` — so a screen can tie it to
 * mount/unmount without the class guessing. After `close()` the instance stays
 * closed; reconnection only happens for drops it did not initiate.
 */
export class InventorySocket {
  private socket: WebSocketLike | null = null;
  private status: ConnectionStatus = 'idle';
  private reconnectAttempt = 0;
  private reconnectTimer: ReturnType<typeof setTimeout> | null = null;
  private manuallyClosed = false;

  constructor(private readonly options: InventorySocketOptions) {}

  getStatus(): ConnectionStatus {
    return this.status;
  }

  private setStatus(next: ConnectionStatus): void {
    if (this.status === next) return;
    this.status = next;
    this.options.onStatusChange?.(next);
  }

  connect(): void {
    if (this.socket || this.manuallyClosed) return;

    this.setStatus(this.reconnectAttempt === 0 ? 'connecting' : 'reconnecting');

    const url = buildUrl(this.options.url ?? DEFAULT_WS_URL, this.options.token);
    const factory =
      this.options.socketFactory ??
      ((target: string) => new WebSocket(target) as unknown as WebSocketLike);

    let socket: WebSocketLike;
    try {
      socket = factory(url);
    } catch {
      // A synchronous constructor throw (bad URL, no network stack) is treated
      // like any other drop so the backoff still applies.
      this.scheduleReconnect();
      return;
    }

    this.socket = socket;

    socket.onopen = () => {
      this.reconnectAttempt = 0;
      this.setStatus('connected');
      this.subscribe();
    };

    socket.onmessage = (event: { data: unknown }) => {
      const update = parseInventoryMessage(event?.data);
      // The backend broadcasts globally, so filter to this screen's event.
      if (!update || update.eventId !== this.options.eventId) return;
      this.options.onUpdate?.(update);
    };

    socket.onerror = () => {
      // `onclose` always follows; reconnection is handled there so a single
      // failure cannot schedule two attempts.
    };

    socket.onclose = () => {
      this.socket = null;
      if (this.manuallyClosed) {
        this.setStatus('closed');
        return;
      }
      this.scheduleReconnect();
    };
  }

  /**
   * Registers for this event's room. The current backend broadcasts to every
   * client and ignores unknown frames, so this is forward-compatible rather
   * than required today — client-side filtering in `onmessage` is what
   * actually scopes the stream.
   */
  private subscribe(): void {
    try {
      this.socket?.send(
        JSON.stringify({ type: 'subscribe', eventId: this.options.eventId }),
      );
    } catch {
      // A send failure means the socket is already gone; `onclose` will fire.
    }
  }

  private scheduleReconnect(): void {
    if (this.manuallyClosed || this.reconnectTimer) return;

    const delay = backoffDelay(this.reconnectAttempt);
    this.reconnectAttempt += 1;
    this.setStatus('reconnecting');

    this.reconnectTimer = setTimeout(() => {
      this.reconnectTimer = null;
      this.connect();
    }, delay);
  }

  /** Closes the socket and stops reconnecting. Safe to call more than once. */
  close(): void {
    this.manuallyClosed = true;

    if (this.reconnectTimer) {
      clearTimeout(this.reconnectTimer);
      this.reconnectTimer = null;
    }

    const socket = this.socket;
    this.socket = null;

    if (socket) {
      socket.onopen = null;
      socket.onmessage = null;
      socket.onerror = null;
      socket.onclose = null;
      try {
        socket.close();
      } catch {
        // Already closed — nothing to do.
      }
    }

    this.setStatus('closed');
  }
}

/** Convenience factory: constructs and immediately connects. */
export function createInventorySocket(
  options: InventorySocketOptions,
): InventorySocket {
  const socket = new InventorySocket(options);
  socket.connect();
  return socket;
}
