import Supercluster from "supercluster";
import type { BBox } from "geojson";

/**
 * Clustering for event map pins (Issue #1140).
 *
 * Kept separate from the map component: the component needs a browser and a
 * Leaflet instance, whereas the grouping and expansion rules are pure and can
 * be tested directly.
 */

export interface ClusterableEvent {
  id: string;
  title: string;
  /** Latitude in degrees, -90..90. */
  lat: number;
  /** Longitude in degrees, -180..180. */
  lng: number;
}

/** Properties carried on a single-event point. */
export interface EventPointProperties {
  cluster: false;
  eventId: string;
  title: string;
}

export type EventFeature = GeoJSON.Feature<GeoJSON.Point, EventPointProperties>;

/**
 * Radius in pixels within which points are merged, and the zoom past which
 * clustering stops and individual pins are shown.
 *
 * 60px is roughly two pin-widths, so pins stop overlapping without collapsing
 * genuinely distinct venues in the same neighbourhood.
 */
export const CLUSTER_RADIUS = 60;
export const CLUSTER_MAX_ZOOM = 16;

/** Coordinates outside these ranges cannot be projected and are dropped. */
const isRenderableCoordinate = (lat: number, lng: number) =>
  Number.isFinite(lat) &&
  Number.isFinite(lng) &&
  lat >= -90 &&
  lat <= 90 &&
  lng >= -180 &&
  lng <= 180;

/**
 * Convert events to GeoJSON, dropping any that cannot be placed.
 *
 * An event with a missing or out-of-range coordinate is skipped rather than
 * clamped: a pin silently rendered at the wrong place is worse than no pin.
 */
export function toGeoJSON(events: ClusterableEvent[]): EventFeature[] {
  return events
    .filter((event) => isRenderableCoordinate(event.lat, event.lng))
    .map((event) => ({
      type: "Feature" as const,
      properties: { cluster: false as const, eventId: event.id, title: event.title },
      geometry: { type: "Point" as const, coordinates: [event.lng, event.lat] },
    }));
}

/**
 * Build the spatial index once for a set of events.
 *
 * Indexing is the expensive step, so callers should memoise on the event list
 * and re-query per viewport change rather than rebuilding as the user pans.
 */
export function buildClusterIndex(
  events: ClusterableEvent[],
): Supercluster<EventPointProperties> {
  const index = new Supercluster<EventPointProperties>({
    radius: CLUSTER_RADIUS,
    maxZoom: CLUSTER_MAX_ZOOM,
  });
  index.load(toGeoJSON(events));
  return index;
}

/** Clusters and single points visible in the given viewport. */
export function getVisibleClusters(
  index: Supercluster<EventPointProperties>,
  bounds: BBox,
  zoom: number,
) {
  // Supercluster expects an integer zoom; Leaflet reports fractional zoom
  // while a pinch is in progress.
  return index.getClusters(bounds, Math.round(zoom));
}

/**
 * Zoom at which a cluster breaks apart, capped so a click always produces a
 * visible change even for points that only separate past the max zoom.
 */
export function expansionZoom(
  index: Supercluster<EventPointProperties>,
  clusterId: number,
  currentZoom: number,
  maxZoom = 18,
): number {
  const target = index.getClusterExpansionZoom(clusterId);
  return Math.min(Math.max(target, currentZoom + 1), maxZoom);
}

/**
 * Marker diameter in pixels for a cluster of `count` events.
 *
 * Scales with the logarithm of the count so that a cluster of 1,000 is
 * visibly larger than one of 10 without being fifty times the size.
 */
export function clusterMarkerSize(count: number): number {
  const MIN = 34;
  const MAX = 64;
  if (count <= 1) return MIN;
  const scaled = MIN + Math.log10(count) * 14;
  return Math.round(Math.min(scaled, MAX));
}

/** Compact label for a cluster badge: 1200 renders as "1.2k". */
export function formatClusterCount(count: number): string {
  if (count < 1000) return String(count);
  const thousands = count / 1000;
  return `${thousands >= 10 ? Math.round(thousands) : thousands.toFixed(1).replace(/\.0$/, "")}k`;
}
