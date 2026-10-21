import { describe, expect, it } from "vitest";
import type { BBox } from "geojson";
import {
  CLUSTER_MAX_ZOOM,
  CLUSTER_RADIUS,
  buildClusterIndex,
  clusterMarkerSize,
  expansionZoom,
  formatClusterCount,
  getVisibleClusters,
  toGeoJSON,
  type ClusterableEvent,
} from "@/lib/cluster-events";

const WORLD: BBox = [-180, -85, 180, 85];

const event = (id: string, lat: number, lng: number): ClusterableEvent => ({
  id,
  title: `Event ${id}`,
  lat,
  lng,
});

/** Three venues within a few hundred metres, plus one far away. */
const NEARBY = [
  event("a", 51.5074, -0.1278),
  event("b", 51.5079, -0.1281),
  event("c", 51.5081, -0.1275),
];
const FAR = event("d", -33.8688, 151.2093);

describe("toGeoJSON", () => {
  it("converts events to GeoJSON points in lng/lat order", () => {
    const [feature] = toGeoJSON([event("a", 51.5, -0.12)]);
    // GeoJSON is [longitude, latitude] — the reverse of Leaflet's [lat, lng].
    expect(feature.geometry.coordinates).toEqual([-0.12, 51.5]);
    expect(feature.properties.eventId).toBe("a");
    expect(feature.properties.cluster).toBe(false);
  });

  it("drops events whose coordinates cannot be projected", () => {
    const events = [
      event("ok", 10, 10),
      event("nan", Number.NaN, 10),
      event("lat-range", 91, 10),
      event("lng-range", 10, 181),
      { id: "missing", title: "x" } as unknown as ClusterableEvent,
    ];
    // A pin silently rendered in the wrong place is worse than no pin.
    expect(toGeoJSON(events).map((f) => f.properties.eventId)).toEqual(["ok"]);
  });

  it("keeps the extremes of the valid ranges", () => {
    const events = [event("nw", 90, -180), event("se", -90, 180)];
    expect(toGeoJSON(events)).toHaveLength(2);
  });
});

describe("clustering behaviour", () => {
  it("groups nearby events into a single cluster when zoomed out", () => {
    const index = buildClusterIndex(NEARBY);
    const clusters = getVisibleClusters(index, WORLD, 3);

    expect(clusters).toHaveLength(1);
    expect(clusters[0].properties.cluster).toBe(true);
    expect(clusters[0].properties.point_count).toBe(3);
  });

  it("reports a cluster count covering every clustered event", () => {
    const index = buildClusterIndex([...NEARBY, FAR]);
    const clusters = getVisibleClusters(index, WORLD, 3);

    const total = clusters.reduce(
      (sum, feature) => sum + (feature.properties.cluster ? feature.properties.point_count : 1),
      0,
    );
    expect(total).toBe(4);
  });

  it("separates events that are far apart even when zoomed out", () => {
    const index = buildClusterIndex([NEARBY[0], FAR]);
    const clusters = getVisibleClusters(index, WORLD, 3);
    expect(clusters).toHaveLength(2);
    expect(clusters.every((c) => !c.properties.cluster)).toBe(true);
  });

  it("reveals individual markers once sufficiently zoomed in", () => {
    const index = buildClusterIndex(NEARBY);
    const clusters = getVisibleClusters(index, WORLD, 18);

    expect(clusters).toHaveLength(3);
    expect(clusters.every((c) => !c.properties.cluster)).toBe(true);
    expect(clusters.map((c) => c.properties.eventId).sort()).toEqual(["a", "b", "c"]);
  });

  it("cluster counts shrink monotonically as zoom increases", () => {
    const index = buildClusterIndex(NEARBY);
    const countAt = (zoom: number) =>
      getVisibleClusters(index, WORLD, zoom).length;

    // More separate features (or the same) at every higher zoom level.
    for (let zoom = 1; zoom < 18; zoom += 1) {
      expect(countAt(zoom + 1)).toBeGreaterThanOrEqual(countAt(zoom));
    }
  });

  it("rounds a fractional zoom rather than failing mid-pinch", () => {
    const index = buildClusterIndex(NEARBY);
    expect(() => getVisibleClusters(index, WORLD, 4.6)).not.toThrow();
    expect(getVisibleClusters(index, WORLD, 4.6)).toEqual(
      getVisibleClusters(index, WORLD, 5),
    );
  });

  it("only returns features inside the requested viewport", () => {
    const index = buildClusterIndex([NEARBY[0], FAR]);
    const londonOnly: BBox = [-1, 51, 1, 52];
    const visible = getVisibleClusters(index, londonOnly, 10);

    expect(visible).toHaveLength(1);
    expect(visible[0].properties.eventId).toBe("a");
  });

  it("handles an empty event list", () => {
    const index = buildClusterIndex([]);
    expect(getVisibleClusters(index, WORLD, 5)).toEqual([]);
  });

  it("indexes a large dataset without error", () => {
    const many: ClusterableEvent[] = Array.from({ length: 5000 }, (_, i) =>
      event(`e${i}`, 51.5 + (i % 100) * 0.001, -0.12 + Math.floor(i / 100) * 0.001),
    );
    const index = buildClusterIndex(many);

    const clusters = getVisibleClusters(index, WORLD, 3);
    const total = clusters.reduce(
      (sum, f) => sum + (f.properties.cluster ? f.properties.point_count : 1),
      0,
    );
    expect(total).toBe(5000);
  });
});

describe("expansionZoom", () => {
  it("always advances at least one zoom level", () => {
    const index = buildClusterIndex(NEARBY);
    const [cluster] = getVisibleClusters(index, WORLD, 3);
    const clusterId = (cluster.properties as { cluster_id: number }).cluster_id;

    // A click that does not visibly change the map reads as a broken marker.
    expect(expansionZoom(index, clusterId, 3)).toBeGreaterThan(3);
  });

  it("never exceeds the maximum zoom", () => {
    const index = buildClusterIndex(NEARBY);
    const [cluster] = getVisibleClusters(index, WORLD, 3);
    const clusterId = (cluster.properties as { cluster_id: number }).cluster_id;

    expect(expansionZoom(index, clusterId, 17, 18)).toBeLessThanOrEqual(18);
    expect(expansionZoom(index, clusterId, 18, 18)).toBe(18);
  });
});

describe("cluster badge presentation", () => {
  it("grows with the count but stays bounded", () => {
    expect(clusterMarkerSize(2)).toBeGreaterThanOrEqual(34);
    expect(clusterMarkerSize(2)).toBeLessThan(clusterMarkerSize(50));
    expect(clusterMarkerSize(50)).toBeLessThan(clusterMarkerSize(5000));
    expect(clusterMarkerSize(1_000_000)).toBeLessThanOrEqual(64);
  });

  it("formats counts compactly once they reach a thousand", () => {
    expect(formatClusterCount(7)).toBe("7");
    expect(formatClusterCount(999)).toBe("999");
    expect(formatClusterCount(1000)).toBe("1k");
    expect(formatClusterCount(1200)).toBe("1.2k");
    expect(formatClusterCount(15400)).toBe("15k");
  });
});

describe("clustering constants", () => {
  it("stops clustering below Leaflet's maximum tile zoom", () => {
    // Above this zoom every event must be individually addressable.
    expect(CLUSTER_MAX_ZOOM).toBeLessThan(19);
    expect(CLUSTER_RADIUS).toBeGreaterThan(0);
  });
});
