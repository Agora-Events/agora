import { useCallback, useEffect, useMemo, useRef, useState } from 'react';
import {
  InventorySocket,
  applyInventoryUpdate,
  type ConnectionStatus,
  type InventoryUpdate,
} from '@/utils/websocket';
import type { TicketTierOption } from '@/types/checkout';

/**
 * Keeps a tier list in sync with live inventory (issue #1010).
 *
 * Opens an {@link InventorySocket} for `eventId` on mount, closes it on
 * unmount, and folds incoming updates into the tier list. Only the `remaining`
 * field is ever rewritten — identity, order, and pricing come from the caller,
 * so a live update re-renders labels without resetting the screen's layout or
 * the user's selection.
 */
export interface UseLiveTicketInventoryOptions {
  eventId: string;
  tiers: TicketTierOption[];
  token?: string;
  url?: string;
  /** Set false to skip connecting entirely (e.g. screen not focused). */
  enabled?: boolean;
}

export interface UseLiveTicketInventoryResult {
  /** `tiers` with any live `remaining` counts applied. */
  tiers: TicketTierOption[];
  status: ConnectionStatus;
  isLive: boolean;
}

export function useLiveTicketInventory({
  eventId,
  tiers,
  token,
  url,
  enabled = true,
}: UseLiveTicketInventoryOptions): UseLiveTicketInventoryResult {
  const [remainingByTier, setRemainingByTier] = useState<Record<string, number>>({});
  const [status, setStatus] = useState<ConnectionStatus>('idle');

  // Held in a ref so the socket effect doesn't re-run whenever the caller
  // passes a new array identity — reconnecting on every render would defeat
  // the point.
  const tiersRef = useRef(tiers);
  tiersRef.current = tiers;

  const handleUpdate = useCallback((update: InventoryUpdate) => {
    setRemainingByTier((previous) => {
      const knownRemaining =
        previous[update.tierId] ??
        tiersRef.current.find((tier) => tier.id === update.tierId)?.remaining;

      const next = applyInventoryUpdate(knownRemaining, update);
      if (next === undefined || next === previous[update.tierId]) return previous;

      return { ...previous, [update.tierId]: next };
    });
  }, []);

  useEffect(() => {
    if (!enabled || !eventId) {
      setStatus('idle');
      return;
    }

    // A different event means the previously accumulated counts no longer apply.
    setRemainingByTier({});

    const socket = new InventorySocket({
      eventId,
      token,
      url,
      onUpdate: handleUpdate,
      onStatusChange: setStatus,
    });
    socket.connect();

    return () => socket.close();
  }, [eventId, token, url, enabled, handleUpdate]);

  const mergedTiers = useMemo(() => {
    if (Object.keys(remainingByTier).length === 0) return tiers;

    return tiers.map((tier) =>
      remainingByTier[tier.id] !== undefined
        ? { ...tier, remaining: remainingByTier[tier.id] }
        : tier,
    );
  }, [tiers, remainingByTier]);

  return {
    tiers: mergedTiers,
    status,
    isLive: status === 'connected',
  };
}
