"use client";

import React, { useEffect, useState, useMemo } from "react";
import { MapContainer, TileLayer, Marker, Popup, useMap } from "react-leaflet";
import L from "leaflet";
import "leaflet/dist/leaflet.css";

// Prevent default icon path issues with webpack
if (L?.Icon?.Default?.prototype) {
  delete (L.Icon.Default.prototype as unknown as Record<string, unknown>)
    ._getIconUrl;
  L.Icon.Default.mergeOptions({
    iconRetinaUrl:
      "https://unpkg.com/leaflet@1.9.4/dist/images/marker-icon-2x.png",
    iconUrl: "https://unpkg.com/leaflet@1.9.4/dist/images/marker-icon.png",
    shadowUrl: "https://unpkg.com/leaflet@1.9.4/dist/images/marker-shadow.png",
  });
}

/**
 * Returns a custom SVG DivIcon styled for the given event category.
 */
export function getCategoryIcon(category?: string) {
  const cat = (category || "").toLowerCase();
  let bg = "#6366F1"; // Default Indigo
  let iconSvg = `<svg width="16" height="16" viewBox="0 0 24 24" fill="none" stroke="white" stroke-width="2.5" stroke-linecap="round" stroke-linejoin="round"><path d="M20 10c0 6-8 12-8 12s-8-6-8-12a8 8 0 0 1 16 0Z"/><circle cx="12" cy="10" r="3"/></svg>`;

  if (cat.includes("music") || cat.includes("concert")) {
    bg = "#8B5CF6"; // Purple
    iconSvg = `<svg width="16" height="16" viewBox="0 0 24 24" fill="none" stroke="white" stroke-width="2.5" stroke-linecap="round" stroke-linejoin="round"><path d="M9 18V5l12-2v13"/><circle cx="6" cy="18" r="3"/><circle cx="18" cy="16" r="3"/></svg>`;
  } else if (
    cat.includes("tech") ||
    cat.includes("code") ||
    cat.includes("hack")
  ) {
    bg = "#2563EB"; // Blue
    iconSvg = `<svg width="16" height="16" viewBox="0 0 24 24" fill="none" stroke="white" stroke-width="2.5" stroke-linecap="round" stroke-linejoin="round"><polyline points="16 18 22 12 16 6"/><polyline points="8 6 2 12 8 18"/></svg>`;
  } else if (
    cat.includes("workshop") ||
    cat.includes("edu") ||
    cat.includes("learn")
  ) {
    bg = "#F59E0B"; // Amber
    iconSvg = `<svg width="16" height="16" viewBox="0 0 24 24" fill="none" stroke="white" stroke-width="2.5" stroke-linecap="round" stroke-linejoin="round"><path d="M14.7 6.3a1 1 0 0 0 0 1.4l1.6 1.6a1 1 0 0 0 1.4 0l3.77-3.77a6 6 0 0 1-7.94 7.94l-6.91 6.91a2.12 2.12 0 0 1-3-3l6.91-6.91a6 6 0 0 1 7.94-7.94l-3.76 3.76z"/></svg>`;
  } else if (
    cat.includes("sport") ||
    cat.includes("fitness") ||
    cat.includes("game")
  ) {
    bg = "#10B981"; // Emerald
    iconSvg = `<svg width="16" height="16" viewBox="0 0 24 24" fill="none" stroke="white" stroke-width="2.5" stroke-linecap="round" stroke-linejoin="round"><path d="M6 9H4.5a2.5 2.5 0 0 1 0-5H6"/><path d="M18 9h1.5a2.5 2.5 0 0 0 0-5H18"/><path d="M4 22h16"/><path d="M10 14.66V17c0 .55-.45 1-1 1H7c-.55 0-1 .45-1 1v1h12v-1c0-.55-.45-1-1-1h-2c-.55 0-1-.45-1-1v-2.34"/><path d="M18 4H6v7a6 6 0 0 0 12 0V4Z"/></svg>`;
  }

  const html = `
    <div class="custom-category-pin" style="position: relative; width: 36px; height: 44px; display: flex; align-items: center; justify-content: center; cursor: pointer; filter: drop-shadow(0 4px 6px rgba(0,0,0,0.25));" data-category="${cat || "default"}">
      <svg width="36" height="44" viewBox="0 0 36 44" fill="none" xmlns="http://www.w3.org/2000/svg" style="position: absolute; top: 0; left: 0;">
        <path d="M18 0C8.05888 0 0 8.05888 0 18C0 29.5 15.5 42.5 17.15 43.85C17.65 44.25 18.35 44.25 18.85 43.85C20.5 42.5 36 29.5 36 18C36 8.05888 27.9411 0 18 0Z" fill="${bg}"/>
        <circle cx="18" cy="18" r="14" fill="rgba(255,255,255,0.2)"/>
      </svg>
      <div style="position: relative; z-index: 1; margin-bottom: 8px; display: flex; align-items: center; justify-content: center;">
        ${iconSvg}
      </div>
    </div>
  `;

  if (typeof L !== "undefined" && typeof L.divIcon === "function") {
    return L.divIcon({
      html,
      className: "custom-category-marker",
      iconSize: [36, 44],
      iconAnchor: [18, 44],
      popupAnchor: [0, -46],
    });
  }

  return undefined as unknown as L.DivIcon;
}

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

export interface EventLocationMapProps {
  location: string;
  category?: string;
  title?: string;
  price?: string | number;
  imageUrl?: string;
  onTileError?: () => void;
}

export default function EventLocationMap({
  location,
  category,
  title,
  price,
  imageUrl,
  onTileError,
}: EventLocationMapProps) {
  const [coords, setCoords] = useState<[number, number] | null>(null);
  const [isFallback, setIsFallback] = useState(false);
  const [isLoading, setIsLoading] = useState(true);

  const markerIcon = useMemo(() => getCategoryIcon(category), [category]);

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

        const results = Array.isArray(data)
          ? data
          : data.results && Array.isArray(data.results)
          ? data.results
          : [];

        if (results.length > 0) {
          const lat = parseFloat(results[0].lat);
          const lon = parseFloat(results[0].lon);
          if (!isNaN(lat) && !isNaN(lon)) {
            const newCoords: [number, number] = [lat, lon];
            geocodeCache.set(location, newCoords);
            if (isMounted) {
              setCoords(newCoords);
              setIsFallback(false);
            }
          } else {
            geocodeCache.set(location, null);
            if (isMounted) {
              setCoords(FALLBACK_COORDS);
              setIsFallback(true);
            }
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
      <div
        role="status"
        aria-label="Loading map"
        className="w-full h-full bg-black/5 animate-pulse flex items-center justify-center"
      >
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
        <div
          role="alert"
          className="absolute top-2 left-1/2 -translate-x-1/2 z-10 bg-white/90 rounded-full px-4 py-1 text-xs text-black/60 font-medium shadow-sm pointer-events-none"
        >
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
          eventHandlers={onTileError ? { tileerror: onTileError } : undefined}
        />
        {/* Only place a precise marker when we have real coordinates */}
        {!isFallback && (
          <Marker position={mapCoords} icon={markerIcon}>
            <Popup className="event-marker-popup" closeButton={false}>
              <div
                data-testid="marker-popup-content"
                className="w-[220px] p-2 font-sans overflow-hidden"
              >
                {imageUrl && (
                  <div className="w-full h-24 mb-2 rounded-lg overflow-hidden relative bg-black/10">
                    <img
                      src={imageUrl}
                      alt={title || "Event thumbnail"}
                      className="w-full h-full object-cover"
                    />
                  </div>
                )}
                {category && (
                  <span
                    data-testid="popup-category"
                    className="inline-block text-[10px] uppercase font-bold tracking-wider px-2 py-0.5 rounded-full bg-black/5 text-black/70 mb-1"
                  >
                    {category}
                  </span>
                )}
                <h4
                  data-testid="popup-title"
                  className="font-heading font-semibold text-sm text-black leading-snug line-clamp-2"
                >
                  {title || "Event Location"}
                </h4>
                <p
                  data-testid="popup-location"
                  className="text-xs text-black/60 truncate mt-0.5"
                >
                  {location}
                </p>
                {price !== undefined && price !== null && (
                  <div
                    data-testid="popup-price"
                    className="mt-2 pt-1.5 border-t border-black/10 flex items-center justify-between"
                  >
                    <span className="text-[11px] text-black/50 font-medium">
                      Price
                    </span>
                    <span className="text-xs font-bold text-black">
                      {typeof price === "number" ? `$${price}` : price}
                    </span>
                  </div>
                )}
              </div>
            </Popup>
          </Marker>
        )}
        <ChangeView center={mapCoords} zoom={mapZoom} />
      </MapContainer>
    </div>
  );
}
