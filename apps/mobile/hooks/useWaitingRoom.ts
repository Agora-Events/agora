import { useCallback, useEffect, useMemo, useRef, useState } from 'react';
import {
  fetchPowChallenge,
  joinWaitingRoom,
  openWaitingRoomStream,
  solvePow,
  WaitingRoomError,
} from '@/services/waitingRoom';

/**
 * Drives the virtual waiting room flow (Issue #1187):
 *
 *   fetch challenge → solve PoW → join queue → stream position → admitted
 *
 * The screen only needs `phase`, the position fields for the countdown UI,
 * `grantToken` (triggers the auto-redirect to checkout) and `retry`.
 * All network orchestration lives in `services/waitingRoom.ts`.
 */

export type WaitingRoomPhase =
  | 'idle'
  | 'fetching-challenge'
  | 'solving'
  | 'joining'
  | 'waiting'
  | 'admitted'
  | 'error';

export interface WaitingRoomState {
  phase: WaitingRoomPhase;
  /** 1-based position in line. */
  position: number | null;
  queueSize: number;
  estimatedWaitSeconds: number | null;
  /** Signed checkout access grant — present only when admitted. */
  grantToken: string | null;
  errorMessage: string | null;
}

const initialState: WaitingRoomState = {
  phase: 'idle',
  position: null,
  queueSize: 0,
  estimatedWaitSeconds: null,
  grantToken: null,
  errorMessage: null,
};

export interface UseWaitingRoomResult extends WaitingRoomState {
  retry: () => void;
}

export function useWaitingRoom(eventId: string, clientId: string): UseWaitingRoomResult {
  const [state, setState] = useState<WaitingRoomState>(initialState);
  const [attempt, setAttempt] = useState(0);
  const closeStreamRef = useRef<(() => void) | null>(null);
  // `onClosed` fires immediately after `onAdmitted` (the service aborts and
  // finishes the stream); without this we'd overwrite the admitted state.
  const phaseRef = useRef<WaitingRoomPhase>('idle');

  useEffect(() => {
    if (!eventId || !clientId) {
      setState((s) => ({ ...s, phase: 'idle', errorMessage: null }));
      return;
    }

    let disposed = false;

    const closeStream = () => {
      closeStreamRef.current?.();
      closeStreamRef.current = null;
    };

    // Make sure any stream from a previous attempt is closed before retrying.
    closeStream();

    const set = (patch: Partial<WaitingRoomState>) => {
      if (patch.phase) phaseRef.current = patch.phase;
      if (!disposed) setState((s) => ({ ...s, ...patch }));
    };

    set({ phase: 'fetching-challenge', errorMessage: null, grantToken: null });

    (async () => {
      try {
        const challenge = await fetchPowChallenge(eventId);
        if (disposed) return;

        set({ phase: 'solving' });
        const nonce = solvePow(challenge.challenge, challenge.difficulty);
        if (disposed) return;

        set({ phase: 'joining' });
        const status = await joinWaitingRoom({
          event_id: eventId,
          client_id: clientId,
          challenge: challenge.challenge,
          nonce,
        });
        if (disposed) return;

        if (status.status === 'admitted' && status.grant_token) {
          set({
            phase: 'admitted',
            position: null,
            queueSize: status.queue_size,
            estimatedWaitSeconds: 0,
            grantToken: status.grant_token,
          });
          return;
        }

        set({
          phase: 'waiting',
          position: status.position,
          queueSize: status.queue_size,
          estimatedWaitSeconds: status.estimated_wait_seconds,
        });

        // Stream live position; the server closes with the grant on admission.
        closeStreamRef.current = openWaitingRoomStream(eventId, clientId, {
          onPosition: (position, queueSize, estimatedWaitSeconds) => {
            set({ phase: 'waiting', position, queueSize, estimatedWaitSeconds });
          },
          onAdmitted: (grantToken) => {
            set({
              phase: 'admitted',
              position: null,
              estimatedWaitSeconds: 0,
              grantToken,
            });
          },
          onError: (message) => {
            set({ phase: 'error', errorMessage: message });
          },
          onClosed: () => {
            closeStreamRef.current = null;
            // Only treat an unexpected close while waiting as a failure —
            // after `admitted` or an explicit `error` the stream closing is
            // expected and already handled.
            if (!disposed && phaseRef.current === 'waiting') {
              set({ phase: 'error', errorMessage: 'Lost connection to the queue. Please retry.' });
            }
          },
        });
      } catch (error) {
        if (disposed) return;
        const message =
          error instanceof WaitingRoomError
            ? error.message
            : error instanceof Error
              ? error.message
              : 'Something went wrong joining the queue.';
        set({ phase: 'error', errorMessage: message });
      }
    })();

    return () => {
      disposed = true;
      closeStream();
    };
  }, [eventId, clientId, attempt]);

  const retry = useCallback(() => {
    closeStreamRef.current?.();
    closeStreamRef.current = null;
    setAttempt((n) => n + 1);
  }, []);

  return useMemo(() => ({ ...state, retry }), [state, retry]);
}
