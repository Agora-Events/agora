"use client";

import React, { useEffect, useState } from "react";
import { MapContainer, TileLayer, Marker, useMap } from "react-leaflet";
import L from "leaflet";
import "leaflet/dist/leaflet.css";

// Prevent default icon path issues with webpack
// eslint-disable-next-line @typescript-eslint/no-explicit-any
delete (L.Icon.Default.prototype as any)._getIconUrl;
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

// Dynamically center the map when coordinates change
function ChangeView({ center }: { center: [number, number] }) {
  const map = useMap();
  useEffect(() => {
    map.setView(center, map.getZoom());
  }, [center, map]);
  return null;
}

const geocodeCache = new Map<string, [number, number] | null>();

interface EventLocationMapProps {
  location: string;
}

export default function EventLocationMap({ location }: EventLocationMapProps) {
  const [coords, setCoords] = useState<[number, number] | null>(null);
  const [isLoading, setIsLoading] = useState(true);
  const [error, setError] = useState(false);

  useEffect(() => {
    let isMounted = true;

    async function geocodeLocation() {
      if (!location) {
        if (isMounted) {
          setError(true);
          setIsLoading(false);
        }
        return;
      }

      if (geocodeCache.has(location)) {
        const cached = geocodeCache.get(location);
        if (isMounted) {
          setCoords(cached || null);
          setError(!cached);
          setIsLoading(false);
        }
        return;
      }

      try {
        const response = await fetch(
          `https://nominatim.openstreetmap.org/search?format=json&q=${encodeURIComponent(
            location,
          )}`,
          {
            headers: {
              "User-Agent": "AgoraApp/1.0 (contact@agora-demo.com)",
            },
          },
        );
        const data = await response.json();

        if (data && data.length > 0) {
          const lat = parseFloat(data[0].lat);
          const lon = parseFloat(data[0].lon);
          const newCoords: [number, number] = [lat, lon];
          geocodeCache.set(location, newCoords);
          if (isMounted) {
            setCoords(newCoords);
            setError(false);
          }
        } else {
          geocodeCache.set(location, null);
          if (isMounted) setError(true);
        }
      } catch (err) {
        console.error("Geocoding error:", err);
        if (isMounted) setError(true);
      } finally {
        if (isMounted) setIsLoading(false);
      }
    }

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

  if (error || !coords) {
    return (
      <div className="w-full h-full bg-black/5 flex items-center justify-center">
        <span className="text-black/50 font-medium font-heading">
          Location not found
        </span>
      </div>
    );
  }

  return (
    <div className="w-full h-full relative z-0">
      <MapContainer
        center={coords}
        zoom={13}
        scrollWheelZoom={false}
        className="w-full h-full z-0!"
        style={{ zIndex: 0 }}
      >
        <TileLayer
          attribution='&copy; <a href="https://www.openstreetmap.org/copyright">OpenStreetMap</a>'
          url="https://{s}.tile.openstreetmap.org/{z}/{x}/{y}.png"
        />
        <Marker position={coords} icon={customIcon} />
        <ChangeView center={coords} />
      </MapContainer>
    </div>
  );
}
