import { useCallback, useEffect, useRef, useState } from "react";
import mockEvents from "../constants/mockEvents.json";
import {
  filterEvents,
  type DiscoverEvent,
  type DiscoverFilters,
} from "../lib/discoverFilters";

/**
 * Fetches the discover feed with caching and an offline fallback (issue #1004).
 *
 * Deliberately not built on `@tanstack/react-query`. The issue allows "react-query
 * or similar caching solutions", and react-query is not currently a dependency
 * of this app — adding it for one screen would pull a provider requirement into
 * the root layout and a peer dependency into every future test that renders a
 * component using it. What this screen actually needs is one cached resource
 * with a stale window and a fallback, which is small enough to own outright.
 *
 * Swap it for react-query if the app adopts it more widely; the hook's surface
 * is deliberately the same shape (`data`, `isLoading`, `error`, `refetch`).
 */

const CACHE_TTL_MS = 5 * 60 * 1000;
const REQUEST_TIMEOUT_MS = 8_000;

/** Module-scoped so the cache survives navigating away and back. */
let cache: { fetchedAt: number; events: DiscoverEvent[] } | null = null;

export interface UseDiscoverEventsResult {
  events: DiscoverEvent[];
  isLoading: boolean;
  isRefreshing: boolean;
  /** True when the list came from cache or bundled mock data. */
  isOffline: boolean;
  error: string | null;
  refetch: () => Promise<void>;
}

const API_URL =
  process.env.EXPO_PUBLIC_API_URL ?? "http://localhost:8080";

async function fetchDiscoverEvents(signal: AbortSignal): Promise<DiscoverEvent[]> {
  const res = await fetch(`${API_URL}/api/events/discover`, { signal });
  if (!res.ok) throw new Error(`Discover request failed (${res.status})`);
  const body = await res.json();
  // Accept either a bare array or an envelope, so the hook does not break the
  // screen if the backend adds pagination metadata later.
  return Array.isArray(body) ? body : (body?.data ?? []);
}

export function useDiscoverEvents(filters: DiscoverFilters = {}) {
  const [events, setEvents] = useState<DiscoverEvent[]>(() => cache?.events ?? []);
  const [isLoading, setLoading] = useState(!cache);
  const [isRefreshing, setRefreshing] = useState(false);
  const [isOffline, setOffline] = useState(false);
  const [error, setError] = useState<string | null>(null);

  // Guards against setting state after unmount, which React warns about and
  // which happens routinely here since the screen is a tab.
  const mounted = useRef(true);
  useEffect(() => {
    mounted.current = true;
    return () => {
      mounted.current = false;
    };
  }, []);

  const load = useCallback(async (isManualRefresh: boolean) => {
    if (isManualRefresh) setRefreshing(true);

    const controller = new AbortController();
    const timeout = setTimeout(() => controller.abort(), REQUEST_TIMEOUT_MS);

    try {
      const fresh = await fetchDiscoverEvents(controller.signal);
      cache = { fetchedAt: Date.now(), events: fresh };
      if (!mounted.current) return;
      setEvents(fresh);
      setOffline(false);
      setError(null);
    } catch (err) {
      if (!mounted.current) return;

      // Degrade rather than fail: a cached list is more useful than an error
      // screen, and the bundled mock data keeps the screen functional on a
      // first run with no backend — which the issue asks for explicitly.
      if (cache) {
        setEvents(cache.events);
      } else {
        setEvents(mockEvents as DiscoverEvent[]);
      }
      setOffline(true);
      setError(err instanceof Error ? err.message : "Could not reach the server");
    } finally {
      clearTimeout(timeout);
      if (mounted.current) {
        setLoading(false);
        setRefreshing(false);
      }
    }
  }, []);

  useEffect(() => {
    const isFresh = cache && Date.now() - cache.fetchedAt < CACHE_TTL_MS;
    if (isFresh) {
      setEvents(cache!.events);
      setLoading(false);
      return;
    }
    void load(false);
  }, [load]);

  // Filtering is applied here rather than in the query so a category tap is
  // instant — it never waits on the network, which is the acceptance criterion.
  const filtered = filterEvents(events, filters);

  return {
    events: filtered,
    isLoading,
    isRefreshing,
    isOffline,
    error,
    refetch: () => load(true),
  } satisfies UseDiscoverEventsResult;
}

export default useDiscoverEvents;
