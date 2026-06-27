"use client";

import { useEffect, useState, useRef } from "react";
import useSWR from "swr";

/**
 * Ticket availability data returned from API
 */
export interface TicketAvailabilityData {
  totalTickets: number;
  mintedTickets: number;
  availableTickets: number;
  isSoldOut: boolean;
  percentageSold: number;
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
  const response = await fetch(url, {
    credentials: "include",
  });

  if (!response.ok) {
    const error = new Error("Failed to fetch ticket availability");
    throw error;
  }

  return response.json();
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

  // Regular SWR polling approach
  const url = `/api/events/${eventId}/availability`;
  const { data, error, isLoading, mutate } = useSWR<TicketAvailabilityData>(
    !useSSE ? url : null,
    fetcher,
    {
      // Only revalidate on focus if pollOnBlur is false
      revalidateOnFocus: !pollOnBlur,
      revalidateOnReconnect: true,
      // Poll at specified interval (0 disables polling)
      refreshInterval: pollInterval > 0 ? pollInterval : 0,
    },
  );

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
