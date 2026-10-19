"use client";

import { useEffect, useState, useRef, useCallback } from "react";
import useSWR from "swr";
import {
  tierMetadataCache,
  eventTiersCacheKey,
  invalidateEventCache,
} from "@/lib/stellar/contract-cache";

/**
 * Ticket availability data returned from API
 */
export interface TicketAvailabilityData {
  totalTickets: number;
  mintedTickets: number;
  availableTickets: number;
  isSoldOut: boolean;
  percentageSold: number;
  isUsingSSE?: boolean;
}

/**
 * Configuration for polling intervals and behavior
 */
interface UseTicketAvailabilityOptions {
  /**
   * Poll interval in milliseconds (default: 5000ms = 5 seconds)
   * Set to 0 to disable polling and only fetch on mount
   */
  pollInterval?: number;
  /**
   * Whether to continue polling when page loses focus (default: true)
   */
  pollOnBlur?: boolean;
  /**
   * Optional WebSocket URL for real-time updates via server-sent events
   * If provided, will use SSE instead of polling for more efficient updates
   */
  sseUrl?: string;
}

const fetcher = async (url: string): Promise<TicketAvailabilityData> => {
  // ── Cache layer ────────────────────────────────────────────────────────────
  // Extract the eventId from the URL pattern `/api/events/{eventId}/availability`
  // and check the in-memory cache before hitting the network.
  const eventIdMatch = url.match(/\/api\/events\/([^/]+)\/availability/);
  const eventId = eventIdMatch?.[1];
  const cacheKey = eventId ? eventTiersCacheKey(eventId) : null;

  if (cacheKey) {
    const cached = tierMetadataCache.get(cacheKey) as TicketAvailabilityData | undefined;
    if (cached) {
      return cached;
    }
  }

  const response = await fetch(url, {
    credentials: "include",
  });

  if (!response.ok) {
    const error = new Error("Failed to fetch ticket availability");
    throw error;
  }

  const data: TicketAvailabilityData = await response.json();

  // Populate cache so repeated modal opens within the TTL window skip the RPC call.
  if (cacheKey) {
    tierMetadataCache.set(cacheKey, data);
  }

  return data;
};

/**
 * Hook to monitor real-time ticket availability for an event
 *
 * @param eventId - The ID of the event to monitor
 * @param options - Configuration options for polling behavior
 * @returns Ticket availability data, loading state, and error status
 *
 * @example
 * ```typescript
 * const { data, isLoading, error, refresh } = useTicketAvailability('evt_123', {
 *   pollInterval: 3000,  // Check every 3 seconds
 *   pollOnBlur: true     // Continue checking even if tab is not focused
 * });
 *
 * if (isLoading) return <LoadingSpinner />;
 * if (error) return <p>Failed to load ticket info</p>;
 *
 * return (
 *   <div>
 *     <p>Available: {data.availableTickets}</p>
 *     <div className="w-full bg-gray-200">
 *       <div
 *         className="bg-green-500 h-2"
 *         style={{ width: `${100 - data.percentageSold}%` }}
 *       />
 *     </div>
 *   </div>
 * );
 * ```
 */
export function useTicketAvailability(
  eventId: string,
  options: UseTicketAvailabilityOptions = {},
) {
  const { pollInterval = 5000, pollOnBlur = true, sseUrl } = options;

  const [useSSE, setUseSSE] = useState(!!sseUrl);
  const [sseData, setSSEData] = useState<TicketAvailabilityData | null>(null);
  const eventSourceRef = useRef<EventSource | null>(null);

  // Track whether the browser tab is currently visible
  const [isTabVisible, setIsTabVisible] = useState(
    typeof document !== "undefined"
      ? document.visibilityState === "visible"
      : true,
  );

  // Page Visibility API: pause polling when hidden, resume + re-fetch on focus
  useEffect(() => {
    const handleVisibilityChange = () => {
      const visible = document.visibilityState === "visible";
      setIsTabVisible(visible);
    };

    document.addEventListener("visibilitychange", handleVisibilityChange);
    return () => {
      document.removeEventListener("visibilitychange", handleVisibilityChange);
    };
  }, []);

  // Regular SWR polling approach
  const url = `/api/events/${eventId}/availability`;
  const { data, error, isLoading, mutate } = useSWR<TicketAvailabilityData>(
    !useSSE ? url : null,
    fetcher,
    {
      // Only revalidate on focus if pollOnBlur is false
      revalidateOnFocus: !pollOnBlur,
      revalidateOnReconnect: true,
      // Pause polling while the tab is hidden; resume at the configured interval when visible
      refreshInterval: pollInterval > 0 && isTabVisible ? pollInterval : 0,
    },
  );

  // Re-fetch immediately when the tab becomes visible again to catch missed updates
  const prevIsTabVisible = useRef(isTabVisible);
  useEffect(() => {
    if (!useSSE && isTabVisible && !prevIsTabVisible.current) {
      mutate();
    }
    prevIsTabVisible.current = isTabVisible;
  }, [isTabVisible, useSSE, mutate]);

  // Server-Sent Events (SSE) approach for real-time updates
  useEffect(() => {
    if (!useSSE || !sseUrl || !eventId) return;

    const eventSource = new EventSource(`${sseUrl}?eventId=${eventId}`);

    eventSource.addEventListener("availability", (event) => {
      try {
        const parsed = JSON.parse(event.data);
        setSSEData(parsed);
      } catch (err) {
        console.error("Failed to parse SSE data:", err);
      }
    });

    eventSource.addEventListener("error", (event) => {
      console.error("SSE connection error:", event);
      // Fallback to polling if SSE fails
      setUseSSE(false);
    });

    eventSourceRef.current = eventSource;

    return () => {
      eventSource.close();
      eventSourceRef.current = null;
    };
  }, [useSSE, sseUrl, eventId]);

  // Use SSE data if available, otherwise use SWR data
  const availabilityData = useSSE ? sseData : data;

  /**
   * Invalidates the in-memory contract cache for this event and triggers a
   * fresh SWR re-fetch. Call this immediately after a successful ticket
   * purchase to ensure inventory reflects the latest on-chain state.
   */
  const invalidateCache = useCallback(() => {
    invalidateEventCache(eventId);
    if (!useSSE) {
      mutate(undefined, { revalidate: true });
    }
  }, [eventId, useSSE, mutate]);

  return {
    /**
     * Current ticket availability data
     */
    data: availabilityData ?? undefined,
    /**
     * Whether data is currently being fetched
     */
    isLoading: useSSE ? false : isLoading,
    /**
     * Error object if fetch failed
     */
    error: useSSE ? null : error,
    /**
     * Manual refresh function to force a data fetch
     */
    refresh: () => {
      if (useSSE) {
        // Can't manually refresh SSE, but it updates automatically
        return;
      }
      mutate();
    },
    /**
     * Invalidates the local contract metadata cache for this event and
     * forces a fresh fetch. Use after a completed purchase transaction.
     */
    invalidateCache,
    /**
     * Whether using Server-Sent Events for real-time updates
     */
    isUsingSSE: useSSE,
  };
}

/**
 * Utility function to calculate ticket availability percentage
 */
export function calculateAvailabilityPercentage(
  available: number,
  total: number,
): number {
  if (total === 0) return 0;
  return (available / total) * 100;
}

/**
 * Utility function to get availability status message
 */
export function getAvailabilityStatus(data: TicketAvailabilityData): string {
  if (data.isSoldOut) {
    return "Sold Out";
  }

  if (data.availableTickets <= 5) {
    return `Only ${data.availableTickets} left!`;
  }

  if (data.percentageSold > 75) {
    return "Almost Sold Out";
  }

  return `${data.availableTickets} tickets available`;
}
