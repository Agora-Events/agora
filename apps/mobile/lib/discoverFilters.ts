/**
 * Discover feed filtering and map helpers (issue #1004).
 *
 * Kept as a pure module, free of React and of `react-native-maps`, so the
 * behaviour the acceptance criteria describe — category refines both list and
 * pins, offline falls back to cached data — is testable without rendering a
 * map. That matters here: the map is a native module, and a test that imports
 * it needs a native runtime the CI job does not have.
 */

export interface DiscoverEvent {
  id: string;
  title: string;
  category: string;
  startsAt: string;
  imageUrl?: string | null;
  latitude?: number | null;
  longitude?: number | null;
  venue?: string | null;
}

export interface DiscoverFilters {
  query?: string;
  category?: string | null;
  /** ISO date — events starting on or after this instant. */
  dateFrom?: string;
  /** ISO date — events starting on or before this instant. */
  dateTo?: string;
}

/** Coordinates used when location permission is denied or unavailable. */
export const DEFAULT_REGION = {
  // New York City, as the issue specifies.
  latitude: 40.7128,
  longitude: -74.006,
  latitudeDelta: 0.15,
  longitudeDelta: 0.15,
};

/**
 * Apply the search header's filters.
 *
 * One function drives both the list and the map pins. Filtering them
 * separately is how the two drift apart — the acceptance criterion is that a
 * category tap refines both, and sharing the implementation is what guarantees
 * it rather than relying on two call sites staying in step.
 */
export function filterEvents(
  events: DiscoverEvent[],
  filters: DiscoverFilters = {},
): DiscoverEvent[] {
  const query = filters.query?.trim().toLowerCase() ?? "";
  const from = filters.dateFrom ? Date.parse(filters.dateFrom) : null;
  const to = filters.dateTo ? Date.parse(filters.dateTo) : null;

  return events.filter((event) => {
    if (filters.category && event.category !== filters.category) return false;

    if (query) {
      const haystack = `${event.title} ${event.venue ?? ""} ${event.category}`.toLowerCase();
      if (!haystack.includes(query)) return false;
    }

    if (from !== null || to !== null) {
      const startsAt = Date.parse(event.startsAt);
      // An unparseable date is dropped rather than treated as matching —
      // showing an event with an unknown date under a date filter is worse
      // than omitting it.
      if (Number.isNaN(startsAt)) return false;
      if (from !== null && startsAt < from) return false;
      if (to !== null && startsAt > to) return false;
    }

    return true;
  });
}

/**
 * Events that can actually be drawn as pins.
 *
 * An event with a null or non-finite coordinate would render at (0, 0) — in
 * the Atlantic — which reads as a real pin in the wrong place rather than as
 * missing data.
 */
export function mappableEvents(events: DiscoverEvent[]): DiscoverEvent[] {
  return events.filter(
    (e) =>
      typeof e.latitude === "number" &&
      typeof e.longitude === "number" &&
      Number.isFinite(e.latitude) &&
      Number.isFinite(e.longitude),
  );
}

/**
 * Region enclosing the given events, or `DEFAULT_REGION` when none are
 * mappable.
 *
 * Padding the span by 40% keeps pins off the screen edge, and the minimum
 * delta stops a single event zooming to street level where there is no context
 * around it.
 */
export function regionForEvents(events: DiscoverEvent[]) {
  const pins = mappableEvents(events);
  if (pins.length === 0) return DEFAULT_REGION;

  const lats = pins.map((p) => p.latitude as number);
  const lngs = pins.map((p) => p.longitude as number);

  const minLat = Math.min(...lats);
  const maxLat = Math.max(...lats);
  const minLng = Math.min(...lngs);
  const maxLng = Math.max(...lngs);

  const MIN_DELTA = 0.02;
  return {
    latitude: (minLat + maxLat) / 2,
    longitude: (minLng + maxLng) / 2,
    latitudeDelta: Math.max((maxLat - minLat) * 1.4, MIN_DELTA),
    longitudeDelta: Math.max((maxLng - minLng) * 1.4, MIN_DELTA),
  };
}
