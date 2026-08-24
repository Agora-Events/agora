"use client";

import dynamic from "next/dynamic";

const EventMap = dynamic(() => import("@/components/events/event-map"), {
  ssr: false,
  loading: () => (
    <div
      className="flex h-full w-full items-center justify-center bg-black/5 animate-pulse"
      role="status"
      aria-label="Loading map"
    >
      <span className="font-heading text-sm font-medium text-black/50">
        Loading map...
      </span>
    </div>
  ),
});

export default function EventMapClient() {
  return <EventMap />;
}