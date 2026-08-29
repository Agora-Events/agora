"use client";

import dynamic from "next/dynamic";
import type { ClusterableEvent } from "@/lib/cluster-events";

/**
 * Leaflet touches `window` at import time, so the clustered map is loaded
 * client-side only — the same pattern `map-client.tsx` uses for the
 * single-event map.
 */
const ClusterMap = dynamic(
  () => import("@/components/events/events-cluster-map"),
  {
    ssr: false,
    loading: () => (
      <div className="w-full h-full bg-black/5 animate-pulse flex items-center justify-center">
        <span className="text-black/50 font-medium font-heading">
          Loading map...
        </span>
      </div>
    ),
  },
);

export default function EventsClusterMapClient({
  events,
  onSelectEvent,
}: {
  events: ClusterableEvent[];
  onSelectEvent?: (eventId: string) => void;
}) {
  return <ClusterMap events={events} onSelectEvent={onSelectEvent} />;
}
