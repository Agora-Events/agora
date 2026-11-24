"use client";

import { useState } from "react";
import dynamic from "next/dynamic";
import { ErrorBoundary } from "@/components/layout/error-boundary";

const Map = dynamic(() => import("@/components/events/event-location-map"), {
  ssr: false,
  loading: () => (
    <div
      role="status"
      aria-label="Loading map"
      className="w-full h-full bg-black/5 animate-pulse flex items-center justify-center"
    >
      <span className="text-black/50 font-medium font-heading">
        Loading map...
      </span>
    </div>
  ),
});

export interface MapClientProps {
  location: string;
  category?: string;
  title?: string;
  price?: string | number;
  imageUrl?: string;
}

function MapFallback({ location }: { location: string }) {
  const mapsHref = `https://www.google.com/maps/search/?api=1&query=${encodeURIComponent(location)}`;

  return (
    <div
      role="region"
      aria-label="Venue location"
      className="w-full h-full bg-surface flex flex-col items-center justify-center gap-4 p-6 text-center"
    >
      <p className="text-xs text-black/50 font-medium">Map unavailable</p>
      <p className="font-heading font-semibold text-black text-base sm:text-lg">
        {location || "Location to be announced"}
      </p>
      {location && (
        <a
          href={mapsHref}
          target="_blank"
          rel="noopener noreferrer"
          className="inline-flex items-center justify-center rounded-full bg-black text-white text-sm font-semibold px-5 py-2.5 hover:bg-black/80 transition-colors"
        >
          Open in Google Maps
        </a>
      )}
    </div>
  );
}

export default function MapClient({
  location,
  category,
  title,
  price,
  imageUrl,
}: MapClientProps) {
  const [tilesFailed, setTilesFailed] = useState(false);

  if (tilesFailed) {
    return <MapFallback location={location} />;
  }

  return (
    <ErrorBoundary fallback={<MapFallback location={location} />}>
      <Map
        location={location}
        category={category}
        title={title}
        price={price}
        imageUrl={imageUrl}
        onTileError={() => setTilesFailed(true)}
      />
    </ErrorBoundary>
  );
}
