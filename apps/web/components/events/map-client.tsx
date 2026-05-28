"use client";

import dynamic from "next/dynamic";

const Map = dynamic(() => import("@/components/events/event-location-map"), {
  ssr: false,
  loading: () => (
    <div className="w-full h-full bg-black/5 animate-pulse flex items-center justify-center">
      <span className="text-black/50 font-medium font-heading">
        Loading map...
      </span>
    </div>
  ),
});

export default function MapClient({ location }: { location: string }) {
  return <Map location={location} />;
}
