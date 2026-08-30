"use client";

import React, { useMemo } from "react";
import useSWR from "swr";
import Link from "next/link";
import Image from "next/image";
import EventsClusterMapClient from "@/components/events/events-cluster-map-client";
import { Button } from "@/components/ui/button";
import type { ClusterableEvent } from "@/lib/cluster-events";

type MapEvent = {
  id: string;
  title: string;
  location: string;
  category: string;
  startsAt: string;
  latitude: number | null;
  longitude: number | null;
};

const fetcher = (url: string) => fetch(url).then((res) => res.json());

/**
 * `map-pin.svg` used by the interactive cluster map and as a legend hint.
 * Keeping the reference here documents the intended integration point.
 */
const MAP_PIN_ICON = "/icons/map-pin.svg";

/**
 * Extracts events that carry a valid WGS84 coordinate so they can be placed
 * on the discovery map. Events without coordinates are intentionally omitted
 * (a pin silently rendered at the wrong place is worse than no pin).
 */
function toClusterableEvents(events: MapEvent[] | undefined): ClusterableEvent[] {
  if (!Array.isArray(events)) return [];

  return events
    .filter(
      (event) =>
        typeof event.latitude === "number" &&
        typeof event.longitude === "number" &&
        Number.isFinite(event.latitude) &&
        Number.isFinite(event.longitude),
    )
    .map((event) => ({
      id: event.id,
      title: event.title,
      lat: event.latitude as number,
      lng: event.longitude as number,
    }));
}

function formatDate(iso: string): string {
  return new Date(iso).toLocaleDateString(undefined, {
    month: "short",
    day: "numeric",
    year: "numeric",
  });
}

interface MapDiscoveryViewProps {
  initialEvents?: MapEvent[];
}

/**
 * Discovery map page (Issue #1138).
 *
 * Renders a responsive full-bleed map with a collapsible list of mapped
 * events. The interactive clustered map (`EventsClusterMapClient`) is wired
 * to any events that expose coordinates; until coordinate data is populated
 * in the API response, a placeholder panel is shown over the map container so
 * the layout is ready for integration with the data source.
 */
export default function MapDiscoveryView({ initialEvents }: MapDiscoveryViewProps) {
  const { data, isLoading, error } = useSWR<{ items: MapEvent[] }>(
    "/api/events",
    fetcher,
    { fallbackData: initialEvents ? { items: initialEvents } : undefined },
  );

  const mappedEvents = useMemo(
    () => toClusterableEvents(data?.items),
    [data?.items],
  );

  const [selectedEventId, setSelectedEventId] = React.useState<string | null>(null);

  const hasCoordinates = mappedEvents.length > 0;

  const mappedList = useMemo(
    () => (data?.items ?? []).filter((event) => toClusterableEvents([event]).length > 0),
    [data?.items],
  );

  if (isLoading && !initialEvents) {
    return (
      <div
        data-testid="map-loading"
        className="w-full h-full bg-surface/50 animate-pulse flex items-center justify-center"
      >
        <span className="text-ink-soft/60 font-medium font-heading">
          Loading map...
        </span>
      </div>
    );
  }

  return (
    <div className="relative flex h-full w-full flex-col md:flex-row">
      {/* Interactive map container — ready for integration with coordinate data. */}
      <div
        data-testid="map-container"
        className="relative min-h-[50vh] w-full grow md:min-h-0 md:basis-3/5 lg:basis-2/3"
      >
        {hasCoordinates ? (
          <EventsClusterMapClient
            events={mappedEvents}
            onSelectEvent={setSelectedEventId}
          />
        ) : (
          <div className="absolute inset-0 overflow-hidden bg-muted">
            <div
              className="absolute inset-0 opacity-40"
              style={{
                backgroundImage:
                  "radial-gradient(circle, rgb(0 0 0 / 0.12) 1px, transparent 1px)",
                backgroundSize: "24px 24px",
              }}
              aria-hidden="true"
            />
            <div className="absolute inset-0 flex items-center justify-center p-6">
              <div className="max-w-md rounded-3xl border border-border-warm bg-base p-8 text-center shadow-[0_16px_48px_rgb(0_0_0/0.08)]">
                <div className="mx-auto mb-4 flex h-16 w-16 items-center justify-center rounded-full bg-surface">
                  <Image
                    src={MAP_PIN_ICON}
                    alt=""
                    width={32}
                    height={32}
                    aria-hidden="true"
                  />
                </div>
                <h2 className="text-xl font-semibold text-ink-deep">
                  Map discovery is coming soon
                </h2>
                <p className="mt-2 text-sm leading-relaxed text-ink-deep/60">
                  The interactive map container is ready. Once events expose
                  their coordinates, they will appear here as clustered pins
                  you can explore and zoom into.
                </p>
                <div className="mt-6 flex flex-wrap items-center justify-center gap-3">
                  <Link href="/discover">
                    <Button
                      backgroundColor="bg-white"
                      textColor="text-black"
                      shadowColor="rgba(0,0,0,1)"
                    >
                      Browse Discover Events
                    </Button>
                  </Link>
                </div>
              </div>
            </div>
          </div>
        )}

        {hasCoordinates && (
          <div className="pointer-events-none absolute bottom-4 left-4 z-[500] rounded-full border border-border-warm bg-base px-4 py-2 text-xs font-medium text-ink-soft shadow-sm">
            <span className="inline-flex h-2 w-2 rounded-full bg-accent" />
            <span className="ml-2">
              {mappedEvents.length} event{mappedEvents.length === 1 ? "" : "s"} on the map
            </span>
          </div>
        )}

        {error && (
          <div className="absolute inset-x-4 top-4 z-[500] rounded-2xl border border-error/30 bg-error/10 px-4 py-3 text-sm text-error">
            Unable to load map events. Please try again later.
          </div>
        )}
      </div>

      {/* Collapsible list of mapped events. Stacks below the map on mobile. */}
      <aside
        data-testid="map-event-list"
        className="w-full shrink-0 border-t border-border-warm bg-base md:basis-2/5 md:border-l md:border-t-0 lg:basis-1/3"
      >
        <div className="flex h-full flex-col">
          <div className="flex items-center justify-between border-b border-border-warm px-5 py-4">
            <h2 className="text-lg font-semibold text-ink">Events on the map</h2>
            <span className="inline-flex min-w-6 items-center justify-center rounded-full bg-black px-2 py-0.5 text-xs font-semibold text-white">
              {mappedEvents.length}
            </span>
          </div>

          <div className="grow overflow-y-auto p-4">
            {mappedEvents.length === 0 ? (
              <div className="flex h-full flex-col items-center justify-center gap-3 py-10 text-center">
                <p className="text-sm text-ink-deep/60">
                  No mapped events yet. Events with coordinates will appear
                  here.
                </p>
              </div>
            ) : (
              <ul className="space-y-3">
                {mappedList.map((event) => (
                  <li key={event.id}>
                    <Link
                      href={`/events/${event.id}`}
                      onMouseEnter={() => setSelectedEventId(event.id)}
                      onFocus={() => setSelectedEventId(event.id)}
                      className={`block rounded-2xl border p-4 transition-colors ${
                        selectedEventId === event.id
                          ? "border-black bg-surface"
                          : "border-border-warm bg-base hover:border-black/30"
                      }`}
                    >
                      <div className="flex items-start gap-3">
                        <div className="mt-0.5 flex h-9 w-9 shrink-0 items-center justify-center rounded-full bg-surface">
                          <Image
                            src={MAP_PIN_ICON}
                            alt=""
                            width={18}
                            height={18}
                            aria-hidden="true"
                          />
                        </div>
                        <div className="min-w-0">
                          <p className="truncate text-sm font-semibold text-ink">
                            {event.title}
                          </p>
                          <p className="mt-0.5 text-xs text-ink-deep/55">
                            {formatDate(event.startsAt)}
                          </p>
                        </div>
                      </div>
                    </Link>
                  </li>
                ))}
              </ul>
            )}
          </div>
        </div>
      </aside>
    </div>
  );
}
