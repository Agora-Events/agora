"use client";

import dynamic from "next/dynamic";

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

export default function MapClient({
  location,
  category,
  title,
  price,
  imageUrl,
}: MapClientProps) {
  return (
    <Map
      location={location}
      category={category}
      title={title}
      price={price}
      imageUrl={imageUrl}
    />
  );
}
