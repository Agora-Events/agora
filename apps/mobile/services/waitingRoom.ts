/**
 * waitingRoom.ts
 *
 * Client for the virtual waiting room API (Issue #1187):
 *
 *   1. `fetchPowChallenge`     – POST /api/v1/waiting-room/challenge
 *   2. `solvePow`              – brute-force a SHA-256 nonce (Hashcash-style)
 *   3. `joinWaitingRoom`       – POST /api/v1/waiting-room/join
 *   4. `getWaitingRoomStatus`  – GET  /api/v1/waiting-room/status
 *   5. `openWaitingRoomStream` – GET  /api/v1/waiting-room/stream (SSE)
 *
 * React Native's `fetch` cannot stream response bodies, so the SSE stream is
 * consumed through `XMLHttpRequest` `onprogress` (which does deliver partial
 * response text) and parsed incrementally.
 */

import { sha256Hex } from '@/utils/sha256';

const API_BASE_URL = process.env.EXPO_PUBLIC_API_URL ?? 'http://localhost:8080';

const REQUEST_TIMEOUT_MS = 15_000;

export interface PowChallenge {
  challenge: string;
  difficulty: number;
  expires_in: number;
}

export interface JoinWaitingRoomRequest {
  event_id: string;
  client_id: string;
  challenge: string;
  nonce: string;
}

export interface QueueStatusPayload {
  status: 'waiting' | 'admitted';
  /** 1-based position in line; null once admitted. */
  position: number | null;
  queue_size: number;
  estimated_wait_seconds: number | null;
  /** Cryptographically signed checkout access grant; present when admitted. */
  grant_token: string | null;
}

export class WaitingRoomError extends Error {
  readonly status: number | null;
  constructor(message: string, status: number | null = null) {
    super(message);
    this.name = 'WaitingRoomError';
    this.status = status;
  }
}

function describeError(error: unknown): string {
  if (error instanceof WaitingRoomError) return error.message;
  if ((error as any)?.name === 'AbortError') return 'Request timed out. Please try again.';
  return error instanceof Error ? error.message : 'Something went wrong. Please try again.';
}

async function parseJson(response: Response): Promise<any> {
  try {
    return await response.json();
  } catch {
    return null;
  }
}

function extractMessage(body: any, status: number, fallback: string): string {
  return body?.message || body?.error?.message || fallback;
}

async function postJson<T>(path: string, payload: unknown): Promise<T> {
  const controller = new AbortController();
  const timeout = setTimeout(() => controller.abort(), REQUEST_TIMEOUT_MS);
  try {
    const response = await fetch(`${API_BASE_URL}${path}`, {
      method: 'POST',
      headers: { 'Content-Type': 'application/json' },
      body: JSON.stringify(payload),
      signal: controller.signal,
    });
    const body = await parseJson(response);
    if (!response.ok) {
      throw new WaitingRoomError(extractMessage(body, response.status, `Request failed (${response.status}).`), response.status);
    }
    // Our server wraps successful payloads in `{ success, data, message }`.
    return (body?.data ?? body) as T;
  } catch (error) {
    if (error instanceof WaitingRoomError) throw error;
    throw new WaitingRoomError(describeError(error));
  } finally {
    clearTimeout(timeout);
  }
}

async function getJson<T>(path: string): Promise<T> {
  const controller = new AbortController();
  const timeout = setTimeout(() => controller.abort(), REQUEST_TIMEOUT_MS);
  try {
    const response = await fetch(`${API_BASE_URL}${path}`, {
      method: 'GET',
      headers: { Accept: 'application/json' },
      signal: controller.signal,
    });
    const body = await parseJson(response);
    if (!response.ok) {
      throw new WaitingRoomError(extractMessage(body, response.status, `Request failed (${response.status}).`), response.status);
    }
    return (body?.data ?? body) as T;
  } catch (error) {
    if (error instanceof WaitingRoomError) throw error;
    throw new WaitingRoomError(describeError(error));
  } finally {
    clearTimeout(timeout);
  }
}

/** Issue a fresh proof-of-work challenge for an event. */
export async function fetchPowChallenge(eventId: string): Promise<PowChallenge> {
  return postJson<PowChallenge>('/api/v1/waiting-room/challenge', { event_id: eventId });
}

/**
 * Find a nonce where SHA-256(challenge || nonce) starts with `difficulty`
 * hex zeros. Expected attempts: 2^(4*difficulty).
 */
export function solvePow(challenge: string, difficulty: number, maxAttempts = 2_000_000): string {
  const prefix = '0'.repeat(difficulty);
  for (let nonce = 0; nonce < maxAttempts; nonce++) {
    if (sha256Hex(`${challenge}${nonce}`).startsWith(prefix)) {
      return String(nonce);
    }
  }
  throw new WaitingRoomError(
    `Could not solve the proof-of-work challenge at difficulty ${difficulty}. Please try again.`
  );
}

/** Verify the client is in the queue / holds a grant. */
export async function joinWaitingRoom(payload: JoinWaitingRoomRequest): Promise<QueueStatusPayload> {
  return postJson<QueueStatusPayload>('/api/v1/waiting-room/join', payload);
}

/** Current queue status for a client. Throws 404 when not in the queue. */
export async function getWaitingRoomStatus(eventId: string, clientId: string): Promise<QueueStatusPayload> {
  const query = `event_id=${encodeURIComponent(eventId)}&client_id=${encodeURIComponent(clientId)}`;
  return getJson<QueueStatusPayload>(`/api/v1/waiting-room/status?${query}`);
}

export interface WaitingRoomStreamHandlers {
  onPosition: (position: number, queueSize: number, estimatedWaitSeconds: number) => void;
  onAdmitted: (grantToken: string) => void;
  onError: (message: string) => void;
  /** Fired when the stream ends (admitted, server error, or abort). */
  onClosed: () => void;
}

/**
 * Open the SSE position stream for a client. Returns a `close()` function.
 *
 * The server pushes a `position` event every ~2s and finishes with an
 * `admitted` event carrying the signed checkout access grant token.
 */
export function openWaitingRoomStream(
  eventId: string,
  clientId: string,
  handlers: WaitingRoomStreamHandlers
): () => void {
  const url = `${API_BASE_URL}/api/v1/waiting-room/stream?event_id=${encodeURIComponent(eventId)}&client_id=${encodeURIComponent(clientId)}`;

  const xhr = new XMLHttpRequest();
  let cursor = 0;
  let buffer = '';
  let finished = false;

  const finish = () => {
    if (finished) return;
    finished = true;
    handlers.onClosed();
  };

  const handleMessage = (message: any) => {
    switch (message?.type) {
      case 'position':
        handlers.onPosition(
          message.position ?? 0,
          message.queue_size ?? 0,
          message.estimated_wait_seconds ?? 0
        );
        break;
      case 'admitted':
        if (message.grant_token) {
          handlers.onAdmitted(message.grant_token);
        }
        xhr.abort();
        finish();
        break;
      case 'not-in-queue':
        handlers.onError('You are not in the queue for this event.');
        xhr.abort();
        finish();
        break;
      case 'error':
        handlers.onError(message.message ?? 'The queue stream ended with an error.');
        xhr.abort();
        finish();
        break;
      default:
        break;
    }
  };

  const consumeChunk = () => {
    const text = xhr.responseText ?? '';
    if (text.length > cursor) {
      buffer += text.slice(cursor);
      cursor = text.length;
    }
    // SSE events are separated by blank lines.
    let separator = buffer.indexOf('\n\n');
    while (separator !== -1) {
      const rawEvent = buffer.slice(0, separator);
      buffer = buffer.slice(separator + 2);
      const dataLines = rawEvent
        .split('\n')
        .filter((line) => line.startsWith('data:'))
        .map((line) => line.slice(5).trim());
      if (dataLines.length > 0) {
        try {
          handleMessage(JSON.parse(dataLines.join('\n')));
        } catch {
          // Ignore malformed frames; keepalive comments carry no `data:`.
        }
      }
      separator = buffer.indexOf('\n\n');
    }
  };

  xhr.open('GET', url, true);
  xhr.setRequestHeader('Accept', 'text/event-stream');
  xhr.onprogress = consumeChunk;
  xhr.onload = () => {
    consumeChunk();
    finish();
  };
  xhr.onerror = () => {
    handlers.onError('Lost connection to the queue stream. Please retry.');
    finish();
  };
  xhr.send();

  // Cleanup: aborting is silent (no `onerror` fires for `abort()`), so no
  // handler callbacks run on unmount.
  return () => {
    try {
      xhr.abort();
    } catch {
      // noop
    }
  };
}
