import {
  filterEvents,
  mappableEvents,
  regionForEvents,
  DEFAULT_REGION,
  type DiscoverEvent,
} from "../lib/discoverFilters";

/**
 * Issue #1004 — the filtering and map-framing behaviour behind the Discover
 * screen. Tested here rather than through the screen so no native module
 * (`react-native-maps`) is loaded into the Jest environment.
 */

const events: DiscoverEvent[] = [
  {
    id: "a",
    title: "Afrobeats Live",
    category: "Music",
    startsAt: "2026-08-16T19:30:00.000Z",
    venue: "Eko Hotel",
    latitude: 6.4281,
    longitude: 3.4306,
  },
  {
    id: "b",
    title: "Lagos Tech Summit",
    category: "Tech",
    startsAt: "2026-08-14T09:00:00.000Z",
    venue: "Landmark Centre",
    latitude: 6.4281,
    longitude: 3.4219,
  },
  {
    id: "c",
    title: "Remote Webinar",
    category: "Tech",
    startsAt: "2026-09-01T09:00:00.000Z",
    venue: null,
    latitude: null,
    longitude: null,
  },
];

describe("filterEvents", () => {
  it("returns everything with no filters", () => {
    expect(filterEvents(events)).toHaveLength(3);
  });

  it("filters by category", () => {
    expect(filterEvents(events, { category: "Tech" }).map((e) => e.id)).toEqual(["b", "c"]);
  });

  it("treats a null category as no filter", () => {
    expect(filterEvents(events, { category: null })).toHaveLength(3);
  });

  it("matches the query against title, venue and category", () => {
    expect(filterEvents(events, { query: "afrobeats" }).map((e) => e.id)).toEqual(["a"]);
    expect(filterEvents(events, { query: "landmark" }).map((e) => e.id)).toEqual(["b"]);
    expect(filterEvents(events, { query: "music" }).map((e) => e.id)).toEqual(["a"]);
  });

  it("ignores query casing and surrounding whitespace", () => {
    expect(filterEvents(events, { query: "  LAGOS  " }).map((e) => e.id)).toEqual(["b"]);
  });

  it("combines category and query", () => {
    expect(filterEvents(events, { category: "Tech", query: "summit" }).map((e) => e.id)).toEqual([
      "b",
    ]);
  });

  it("filters by date range, inclusive of both bounds", () => {
    const out = filterEvents(events, {
      dateFrom: "2026-08-14T09:00:00.000Z",
      dateTo: "2026-08-16T19:30:00.000Z",
    });
    expect(out.map((e) => e.id).sort()).toEqual(["a", "b"]);
  });

  it("drops events with an unparseable date when a date filter is active", () => {
    const broken: DiscoverEvent[] = [
      { id: "x", title: "Broken", category: "Music", startsAt: "not-a-date" },
    ];
    // Showing an event with an unknown date under a date filter is worse than
    // omitting it.
    expect(filterEvents(broken, { dateFrom: "2026-01-01T00:00:00.000Z" })).toHaveLength(0);
    // With no date filter it still shows.
    expect(filterEvents(broken)).toHaveLength(1);
  });
});

describe("mappableEvents", () => {
  it("excludes events without usable coordinates", () => {
    // A null coordinate would otherwise render at (0, 0) — in the Atlantic —
    // which reads as a real pin in the wrong place.
    expect(mappableEvents(events).map((e) => e.id)).toEqual(["a", "b"]);
  });

  it("excludes non-finite coordinates", () => {
    const bad: DiscoverEvent[] = [
      { id: "n", title: "NaN", category: "Music", startsAt: "2026-08-01T00:00:00.000Z", latitude: NaN, longitude: 3 },
    ];
    expect(mappableEvents(bad)).toHaveLength(0);
  });
});

describe("regionForEvents", () => {
  it("falls back to the default region when nothing is mappable", () => {
    expect(regionForEvents([events[2]])).toEqual(DEFAULT_REGION);
    expect(regionForEvents([])).toEqual(DEFAULT_REGION);
  });

  it("centres on the mappable events", () => {
    const region = regionForEvents(events);
    expect(region.latitude).toBeCloseTo(6.4281, 4);
    expect(region.longitude).toBeCloseTo((3.4306 + 3.4219) / 2, 4);
  });

  it("never zooms below the minimum span for a single pin", () => {
    // Without a floor, one event zooms to street level with no context.
    const region = regionForEvents([events[0]]);
    expect(region.latitudeDelta).toBeGreaterThanOrEqual(0.02);
    expect(region.longitudeDelta).toBeGreaterThanOrEqual(0.02);
  });

  it("reframes to the filtered subset", () => {
    // The map must follow the active filter, not the full dataset.
    const techOnly = filterEvents(events, { category: "Tech" });
    expect(regionForEvents(techOnly).longitude).toBeCloseTo(3.4219, 4);
  });
});
