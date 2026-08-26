"use client";

import React, { useEffect, useState } from "react";
import { MapContainer, TileLayer, Marker, useMap } from "react-leaflet";
import L from "leaflet";
import "leaflet/dist/leaflet.css";

// Prevent default icon path issues with webpack
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
 * Fallback coordinates used when geocoding returns no results.
 * Defaults to a world-level view centred on 0, 0 (zoom 2).
 */
const FALLBACK_COORDS: [number, number] = [20, 0];
const FALLBACK_ZOOM = 2;

// Client-side in-process cache to avoid redundant proxy calls for the same
// location string within a single page session.
const geocodeCache = new Map<string, [number, number] | null>();

// Dynamically centre the map when coordinates change
function ChangeView({
  center,
  zoom,
}: {
  center: [number, number];
  zoom: number;
}) {
  const map = useMap();
  useEffect(() => {
    map.setView(center, zoom);
  }, [center, zoom, map]);
  return null;
}

interface EventLocationMapProps {
  location: string;
}

export default function EventLocationMap({ location }: EventLocationMapProps) {
  const [coords, setCoords] = useState<[number, number] | null>(null);
  const [isFallback, setIsFallback] = useState(false);
  const [isLoading, setIsLoading] = useState(true);

  useEffect(() => {
    let isMounted = true;

    async function geocodeLocation() {
      if (!location) {
        if (isMounted) {
          setCoords(FALLBACK_COORDS);
          setIsFallback(true);
          setIsLoading(false);
        }
        return;
      }

      // Client-side cache hit
      if (geocodeCache.has(location)) {
        const cached = geocodeCache.get(location);
        if (isMounted) {
          if (cached) {
            setCoords(cached);
            setIsFallback(false);
          } else {
            setCoords(FALLBACK_COORDS);
            setIsFallback(true);
          }
          setIsLoading(false);
        }
        return;
      }

      try {
        // Route through the server-side proxy — no direct Nominatim calls.
        const response = await fetch(
          `/api/geocode?location=${encodeURIComponent(location)}`,
        );

        if (!response.ok) {
          throw new Error(`Geocode proxy returned HTTP ${response.status}`);
        }

        const data = await response.json();

        if (data.results && data.results.length > 0) {
          const { lat, lon } = data.results[0];
          const newCoords: [number, number] = [lat, lon];
          geocodeCache.set(location, newCoords);
          if (isMounted) {
            setCoords(newCoords);
            setIsFallback(false);
          }
        } else {
          // No results — use city/world-level fallback map.
          geocodeCache.set(location, null);
          if (isMounted) {
            setCoords(FALLBACK_COORDS);
            setIsFallback(true);
          }
        }
      } catch (err) {
        console.error("Geocoding error:", err);
        if (isMounted) {
          setCoords(FALLBACK_COORDS);
          setIsFallback(true);
        }
      } finally {
        if (isMounted) setIsLoading(false);
      }
    }

    // Debounce rapid prop changes (e.g. while the user types a location)
    const timeoutId = setTimeout(() => {
      geocodeLocation();
    }, 300);

    return () => {
      isMounted = false;
      clearTimeout(timeoutId);
    };
  }, [location]);

  if (isLoading) {
    return (
      <div className="w-full h-full bg-black/5 animate-pulse flex items-center justify-center">
        <span className="text-black/50 font-medium font-heading">
          Loading map...
        </span>
      </div>
    );
  }

  // coords is guaranteed non-null here (set to FALLBACK_COORDS on error)
  const mapCoords = coords ?? FALLBACK_COORDS;
  const mapZoom = isFallback ? FALLBACK_ZOOM : 13;

  return (
    <div className="w-full h-full relative z-0">
      {isFallback && (
        <div className="absolute top-2 left-1/2 -translate-x-1/2 z-10 bg-white/90 rounded-full px-4 py-1 text-xs text-black/60 font-medium shadow-sm pointer-events-none">
          Exact location unavailable — showing approximate area
        </div>
      )}
      <MapContainer
        center={mapCoords}
        zoom={mapZoom}
        scrollWheelZoom={false}
        className="w-full h-full z-0!"
        style={{ zIndex: 0 }}
      >
        <TileLayer
          attribution='&copy; <a href="https://www.openstreetmap.org/copyright">OpenStreetMap</a>'
          url="https://{s}.tile.openstreetmap.org/{z}/{x}/{y}.png"
        />
        {/* Only place a precise marker when we have real coordinates */}
        {!isFallback && <Marker position={mapCoords} icon={customIcon} />}
        <ChangeView center={mapCoords} zoom={mapZoom} />
      </MapContainer>
    </div>
  );
}
