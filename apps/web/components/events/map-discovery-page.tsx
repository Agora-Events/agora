"use client";

import { useState, useEffect } from "react";
import { Navbar } from "@/components/layout/navbar";
import { Footer } from "@/components/layout/footer";
import LoadingBar from "@/components/ui/loading-bar";
import EventMapClient from "@/components/events/event-map-client";

/**
 * Map Discovery Page — renders the `/discover/map` route layout.
 *
 * Displays a header, an interactive event map (see #1139), and
 * loading / ready states.
 *
 * Tasks:
 * - #1138 — Map Discovery Page (route + layout + placeholder)
 * - #1139 — Interactive Event Map integration
 */
export function MapDiscoveryPage() {
  const [isLoading, setIsLoading] = useState(true);

  // Brief initial load phase for page layout
  useEffect(() => {
    const timer = setTimeout(() => setIsLoading(false), 400);
    return () => clearTimeout(timer);
  }, []);

  return (
    <main className="flex min-h-screen flex-col bg-base">
      <Navbar />

      <div className="mx-auto w-full max-w-7xl flex-1 px-4 py-8 sm:px-6 lg:py-12">
        {/* Header section */}
        <div className="mb-8">
          <p className="text-sm font-bold uppercase tracking-[0.18em] text-accent-dark">
            Map Discovery
          </p>
          <h1 className="mt-2 text-4xl font-bold text-ink-deep sm:text-5xl">
            Explore events on the map
          </h1>
          <p className="mt-3 max-w-2xl text-sand">
            Discover events happening near you. Browse by location, zoom in
            for details, and find your next experience.
          </p>
        </div>

        {/* Interactive map container */}
        <div
          className="relative w-full overflow-hidden rounded-2xl border border-border-warm bg-surface"
          style={{ minHeight: 500 }}
          role="region"
          aria-label="Event discovery map"
        >
          {isLoading ? (
            /* Initial page layout loading state */
            <div className="flex h-full w-full items-center justify-center py-20">
              <div className="flex flex-col items-center gap-4">
                <LoadingBar />
                <span className="text-sm font-medium text-sand">
                  Loading map data...
                </span>
              </div>
            </div>
          ) : (
            /* Interactive map with event markers */
            <div className="h-full w-full" data-testid="map-container">
              <EventMapClient />
            </div>
          )}
        </div>

        {/* Event count info */}
        <div className="mt-6 text-sm text-sand">
          <p>
            Click on markers to see event details. Zoom in for a closer look
            at specific areas.
          </p>
        </div>
      </div>

      <Footer />
    </main>
  );
}