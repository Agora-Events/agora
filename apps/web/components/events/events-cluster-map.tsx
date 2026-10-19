"use client";

import React, { useCallback, useEffect, useMemo, useState } from "react";
import { MapContainer, Marker, TileLayer, useMap, useMapEvents } from "react-leaflet";
import L from "leaflet";
import "leaflet/dist/leaflet.css";
import type { BBox } from "geojson";
import {
  buildClusterIndex,
  clusterMarkerSize,
  expansionZoom,
  formatClusterCount,
  getVisibleClusters,
  type ClusterableEvent,
  type EventPointProperties,
} from "@/lib/cluster-events";

const eventIcon = new L.Icon({
  iconUrl: "/icons/map-pin.svg",
  iconSize: [40, 40],
  iconAnchor: [20, 40],
  popupAnchor: [0, -40],
});

/**
 * Cluster badge. Built with `divIcon` rather than an image so the count can be
 * rendered as live text — it has to stay readable and re-render as the count
 * changes on zoom.
 */
function clusterIcon(count: number): L.DivIcon {
  const size = clusterMarkerSize(count);
  return L.divIcon({
    html: `<span style="width:${size}px;height:${size}px">${formatClusterCount(count)}</span>`,
    className: "agora-cluster-marker",
    iconSize: [size, size],
    iconAnchor: [size / 2, size / 2],
  });
}

interface ClusterLayerProps {
  events: ClusterableEvent[];
  onSelectEvent?: (eventId: string) => void;
}

/**
 * Recomputes the visible clusters whenever the viewport changes.
 *
 * The spatial index is built once per event list; only the cheap viewport
 * query re-runs on pan and zoom, which is what keeps large datasets smooth.
 */
function ClusterLayer({ events, onSelectEvent }: ClusterLayerProps) {
  const map = useMap();
  const index = useMemo(() => buildClusterIndex(events), [events]);
  const [viewport, setViewport] = useState<{ bounds: BBox; zoom: number } | null>(null);

  const readViewport = useCallback(() => {
    const bounds = map.getBounds();
    setViewport({
      bounds: [
        bounds.getWest(),
        bounds.getSouth(),
        bounds.getEast(),
        bounds.getNorth(),
      ] as BBox,
      zoom: map.getZoom(),
    });
  }, [map]);

  useEffect(() => {
    readViewport();
  }, [readViewport, index]);

  useMapEvents({
    moveend: readViewport,
    zoomend: readViewport,
  });

  if (!viewport) return null;

  const clusters = getVisibleClusters(index, viewport.bounds, viewport.zoom);

  return (
    <>
      {clusters.map((feature) => {
        const [lng, lat] = feature.geometry.coordinates;
        const props = feature.properties as
          | EventPointProperties
          | { cluster: true; cluster_id: number; point_count: number };

        if ("cluster" in props && props.cluster) {
          const { cluster_id: clusterId, point_count: count } = props;
          return (
            <Marker
              key={`cluster-${clusterId}`}
              position={[lat, lng]}
              icon={clusterIcon(count)}
              eventHandlers={{
                click: () => {
                  // Zooming to the expansion zoom is what makes a cluster feel
                  // like a container rather than a dead marker.
                  map.setView([lat, lng], expansionZoom(index, clusterId, map.getZoom()));
                },
              }}
            />
          );
        }

        const single = props as EventPointProperties;
        return (
          <Marker
            key={`event-${single.eventId}`}
            position={[lat, lng]}
            icon={eventIcon}
            title={single.title}
            eventHandlers={{
              click: () => onSelectEvent?.(single.eventId),
            }}
          />
        );
      })}
    </>
  );
}

export interface EventsClusterMapProps {
  events: ClusterableEvent[];
  center?: [number, number];
  zoom?: number;
  onSelectEvent?: (eventId: string) => void;
}

/**
 * Discovery map that groups nearby events into clusters when zoomed out and
 * reveals individual pins as the user zooms in (Issue #1140).
 */
export default function EventsClusterMap({
  events,
  center = [20, 0],
  zoom = 2,
  onSelectEvent,
}: EventsClusterMapProps) {
  return (
    <div className="w-full h-full relative z-0">
      <MapContainer
        center={center}
        zoom={zoom}
        scrollWheelZoom
        className="w-full h-full z-0!"
        style={{ zIndex: 0 }}
      >
        <TileLayer
          attribution='&copy; <a href="https://www.openstreetmap.org/copyright">OpenStreetMap</a>'
          url="https://{s}.tile.openstreetmap.org/{z}/{x}/{y}.png"
        />
        <ClusterLayer events={events} onSelectEvent={onSelectEvent} />
      </MapContainer>
    </div>
  );
}
