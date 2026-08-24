"use client";

import React, { useEffect, useState, useCallback, useMemo, useRef } from "react";
import {
  MapContainer,
  TileLayer,
  Marker,
  Popup,
  useMap,
  useMapEvents,
} from "react-leaflet";
import L from "leaflet";
import "leaflet/dist/leaflet.css";
import Supercluster from "supercluster";

// Fix default icon path issues with webpack/Next.js
delete (L.Icon.Default.prototype as unknown as Record<string, unknown>)
  ._getIconUrl;
L.Icon.Default.mergeOptions({
  iconRetinaUrl:
    "https://unpkg.com/leaflet@1.9.4/dist/images/marker-icon-2x.png",
  iconUrl: "https://unpkg.com/leaflet@1.9.4/dist/images/marker-icon.png",
  shadowUrl: "https://unpkg.com/leaflet@1.9.4/dist/images/marker-shadow.png",
});

const customIcon = new L.Icon({
  iconUrl: "/icons/map-pin.svg",
  iconSize: [40, 40],
  iconAnchor: [20, 40],
  popupAnchor: [0, -40],
});

/**
 * Cluster icon factory — renders a circle with the count.
 * Colour and size scale with the number of events in the cluster.
 */
function createClusterIcon(count: number): L.DivIcon {
  const size = count < 10 ? 44 : count < 50 ? 54 : 64;
  const hue = count < 10 ? 200 : count < 50 ? 30 : 0; // blue → orange → red
  return L.divIcon({
    html: `<div style="
      width:${size}px;height:${size}px;
      border-radius:50%;
      background:hsl(${hue},70%,55%);
      color:#fff;
      display:flex;
      align-items:center;
      justify-content:center;
      font-size:${size < 54 ? 14 : 16}px;
      font-weight:700;
      box-shadow:0 2px 6px rgba(0,0,0,.3);
      border:3px solid rgba(255,255,255,.8);
      cursor:pointer;
    ">${count}</div>`,
    className: "",
    iconSize: [size, size],
    iconAnchor: [size / 2, size / 2],
  });
}

/**
 * Default centre: Lagos, Nigeria (West Africa region).
 * Falls back to this when no events with coordinates are found.
 */
const DEFAULT_CENTER: [number, number] = [8.5, 4.5];
const DEFAULT_ZOOM = 5;

// ─── Types ───────────────────────────────────────────────────────────────────

interface MapEvent {
  id: string;
  title: string;
  date: string;
  location: string;
  latitude: number | null;
  longitude: number | null;
  price: string;
  imageUrl: string;
  category: string;
}

interface MapViewState {
  center: [number, number];
  zoom: number;
}

// ─── Supercluster setup ──────────────────────────────────────────────────────

/** Convert a MapEvent array into GeoJSON FeatureCollection for supercluster. */
function eventsToGeoJSON(events: MapEvent[]): Supercluster.PointFeature<MapEvent>[] {
  return events
    .filter((e) => e.latitude != null && e.longitude != null)
    .map((e) => ({
      type: "Feature" as const,
      geometry: {
        type: "Point" as const,
        coordinates: [e.longitude!, e.latitude!],
      },
      properties: { ...e },
    }));
}

// ─── Sub-components ──────────────────────────────────────────────────────────

/** Updates the map viewport when the centre/zoom changes. */
function ChangeView({ center, zoom }: MapViewState) {
  const map = useMap();
  useEffect(() => {
    map.setView(center, zoom);
  }, [center, zoom, map]);
  return null;
}

/** Fits the map bounds to all event markers, or falls back to default. */
function FitBounds({ events }: { events: MapEvent[] }) {
  const map = useMap();
  useEffect(() => {
    const coords = events
      .filter((e) => e.latitude != null && e.longitude != null)
      .map((e) => [e.latitude!, e.longitude!] as [number, number]);

    if (coords.length > 0) {
      const bounds = L.latLngBounds(coords);
      map.fitBounds(bounds, { padding: [50, 50], maxZoom: 14 });
    }
  }, [events, map]);
  return null;
}

/**
 * Renders clustered markers using supercluster.
 * Listens to viewport changes and re-computes clusters on every move/zoom.
 */
function ClusterLayer({
  events,
  supercluster,
}: {
  events: MapEvent[];
  supercluster: Supercluster<MapEvent, MapEvent>;
}) {
  const map = useMap();
  const [zoom, setZoom] = useState(map.getZoom());
  const [bounds, setBounds] = useState(map.getBounds());

  // Recalculate clusters when the viewport changes
  const onViewChange = useCallback(() => {
    setZoom(map.getZoom());
    setBounds(map.getBounds());
  }, [map]);

  useMapEvents({
    moveend: onViewChange,
    zoomend: onViewChange,
  });

  // Get clusters for the current viewport
  const clusters = useMemo(() => {
    const b = bounds;
    const bbox: [number, number, number, number] = [
      b.getWest(),
      b.getSouth(),
      b.getEast(),
      b.getNorth(),
    ];
    return supercluster.getClusters(bbox, Math.round(zoom));
  }, [supercluster, bounds, zoom]);

  return (
    <>
      {(clusters ?? []).map((cluster) => {
        const [lng, lat] = cluster.geometry.coordinates;
        const props = cluster.properties;

        if (cluster.properties.cluster) {
          // Cluster marker
          const count = cluster.properties.point_count;
          const clusterId = cluster.properties.cluster_id;
          return (
            <Marker
              key={`cluster-${clusterId}`}
              position={[lat, lng]}
              icon={createClusterIcon(count)}
              eventHandlers={{
                click: () => {
                  // Zoom in so the cluster expands
                  const expansionZoom =
                    supercluster.getClusterExpansionZoom(clusterId);
                  map.setView([lat, lng], expansionZoom, { animate: true });
                },
              }}
            />
          );
        }

        // Individual event marker
        return (
          <Marker key={props.id} position={[lat, lng]} icon={customIcon}>
            <Popup>
              <div className="min-w-[180px]">
                {props.imageUrl && (
                  <img
                    src={props.imageUrl}
                    alt={props.title}
                    className="mb-2 h-24 w-full rounded-lg object-cover"
                  />
                )}
                <h3 className="text-sm font-semibold text-gray-900">
                  {props.title}
                </h3>
                <p className="mt-1 text-xs text-gray-600">{props.date}</p>
                <p className="text-xs text-gray-500">{props.location}</p>
                <p className="mt-1 text-xs font-medium text-gray-800">
                  {props.price === "Free" ? "Free" : `${props.price}`}
                </p>
              </div>
            </Popup>
          </Marker>
        );
      })}
    </>
  );
}

// ─── Main Component ──────────────────────────────────────────────────────────

interface EventMapProps {
  /** Optional: pre-fetched events to display. When omitted, the component
   *  fetches events from the discover API. */
  initialEvents?: MapEvent[];
}

export default function EventMap({ initialEvents }: EventMapProps) {
  const [events, setEvents] = useState<MapEvent[]>(initialEvents ?? []);
  const [isLoading, setIsLoading] = useState(!initialEvents);
  const [error, setError] = useState<string | null>(null);
  const [selectedEvent, setSelectedEvent] = useState<MapEvent | null>(null);
  const clusterRef = useRef<Supercluster<MapEvent, MapEvent> | null>(null);

  const fetchEvents = useCallback(async () => {
    if (initialEvents) return;
    setIsLoading(true);
    setError(null);

    try {
      const response = await fetch("/api/events/discover");
      if (!response.ok) {
        throw new Error("Failed to fetch event data");
      }
      const data = await response.json();
      // The discover API returns popularEvents with latitude/longitude
      const mapEvents: MapEvent[] = (data.popularEvents ?? []) as MapEvent[];
      setEvents(mapEvents);
    } catch (err) {
      setError(
        err instanceof Error ? err.message : "An unexpected error occurred"
      );
    } finally {
      setIsLoading(false);
    }
  }, [initialEvents]);

  useEffect(() => {
    fetchEvents();
  }, [fetchEvents]);

  // Build supercluster index from events
  const supercluster = useMemo(() => {
    const features = eventsToGeoJSON(events);
    if (features.length === 0) return null;

    const index = new Supercluster<MapEvent, MapEvent>({
      radius: 60, // cluster radius in pixels
      maxZoom: 16, // max zoom level to cluster
      minZoom: 2,  // min zoom level to start clustering
    });
    index.load(features);
    return index;
  }, [events]);

  // Filter events that have coordinates
  const eventsWithCoords = events.filter(
    (e) => e.latitude != null && e.longitude != null
  );

  // ── Loading state ──────────────────────────────────────────────────────
  if (isLoading) {
    return (
      <div
        className="flex h-full w-full items-center justify-center"
        role="status"
        aria-label="Loading map"
      >
        <div className="flex flex-col items-center gap-3">
          <div className="h-8 w-8 animate-spin rounded-full border-4 border-accent border-t-transparent" />
          <span className="text-sm font-medium text-sand">
            Loading events map...
          </span>
        </div>
      </div>
    );
  }

  // ── Error state ────────────────────────────────────────────────────────
  if (error) {
    return (
      <div
        className="flex h-full w-full items-center justify-center"
        role="alert"
      >
        <div className="flex flex-col items-center gap-2 text-center">
          <svg
            className="h-10 w-10 text-red-400"
            fill="none"
            viewBox="0 0 24 24"
            stroke="currentColor"
            strokeWidth={1.5}
            aria-hidden="true"
          >
            <path
              strokeLinecap="round"
              strokeLinejoin="round"
              d="M12 9v3.75m-9.303 3.376c-.866 1.5.217 3.374 1.948 3.374h14.71c1.73 0 2.813-1.874 1.948-3.374L13.949 3.378c-.866-1.5-3.032-1.5-3.898 0L2.697 16.126ZM12 15.75h.007v.008H12v-.008Z"
            />
          </svg>
          <p className="text-sm font-medium text-red-500">
            Failed to load map data
          </p>
          <p className="text-xs text-sand">{error}</p>
          <button
            onClick={fetchEvents}
            className="mt-2 rounded-lg bg-accent px-4 py-1.5 text-sm font-medium text-white transition-colors hover:bg-accent-dark"
          >
            Retry
          </button>
        </div>
      </div>
    );
  }

  // ── Empty state ────────────────────────────────────────────────────────
  if (events.length === 0) {
    return (
      <div
        className="flex h-full w-full items-center justify-center"
        data-testid="map-empty"
      >
        <div className="flex flex-col items-center gap-2 text-center">
          <svg
            className="h-12 w-12 text-sand"
            fill="none"
            viewBox="0 0 24 24"
            stroke="currentColor"
            strokeWidth={1.5}
            aria-hidden="true"
          >
            <path
              strokeLinecap="round"
              strokeLinejoin="round"
              d="M9 6.75V15m6-6v8.25m.503 3.498 4.875-2.437c.381-.19.622-.58.622-1.006V4.82c0-.836-.88-1.38-1.628-1.006l-3.869 1.934c-.317.159-.69.159-1.006 0L9.503 3.252a1.125 1.125 0 0 0-1.006 0L3.622 5.689C3.24 5.88 3 6.27 3 6.695V19.18c0 .836.88 1.38 1.628 1.006l3.869-1.934c.317-.159.69-.159 1.006 0l4.994 2.497c.317.158.69.158 1.006 0Z"
            />
          </svg>
          <p className="text-lg font-semibold text-ink-soft">
            No events to display
          </p>
          <p className="text-sm text-sand">
            Events with location data will appear here on the map.
          </p>
        </div>
      </div>
    );
  }

  // Determine the map centre: use the first event's coordinates, or default
  const centre: [number, number] =
    eventsWithCoords.length > 0
      ? [eventsWithCoords[0].latitude!, eventsWithCoords[0].longitude!]
      : DEFAULT_CENTER;

  return (
    <div className="h-full w-full" data-testid="event-map">
      <MapContainer
        center={centre}
        zoom={DEFAULT_ZOOM}
        className="h-full w-full"
        scrollWheelZoom
        zoomControl
      >
        <TileLayer
          attribution='&copy; <a href="https://www.openstreetmap.org/copyright">OpenStreetMap</a>'
          url="https://{s}.tile.openstreetmap.org/{z}/{x}/{y}.png"
        />

        <FitBounds events={eventsWithCoords} />

        {supercluster && (
          <ClusterLayer
            events={eventsWithCoords}
            supercluster={supercluster}
          />
        )}
      </MapContainer>
    </div>
  );
}