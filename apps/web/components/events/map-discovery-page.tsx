"use client";

import { useState, useEffect } from "react";
import { Navbar } from "@/components/layout/navbar";
import { Footer } from "@/components/layout/footer";
import LoadingBar from "@/components/ui/loading-bar";

/**
 * Map Discovery Page — renders the `/discover/map` route layout.
 *
 * Displays a header, a placeholder map container where the interactive
 * map will be integrated (see #1139), and loading / ready states.
 *
 * #1138 — Map Discovery Page
 */
export function MapDiscoveryPage() {
  const [isLoading, setIsLoading] = useState(true);

  // Simulate initial data loading (e.g. fetching map data from backend).
  // The placeholder appears immediately after the brief loading phase.
  useEffect(() => {
    const timer = setTimeout(() => setIsLoading(false), 800);
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

        {/* Map container — placeholder ready for #1139 integration */}
        <div
          className="relative w-full overflow-hidden rounded-2xl border border-border-warm bg-surface"
          style={{ minHeight: 400 }}
          role="region"
          aria-label="Event discovery map"
        >
          {isLoading ? (
            /* Loading state */
            <div className="flex h-full w-full items-center justify-center py-20">
              <div className="flex flex-col items-center gap-4">
                <LoadingBar />
                <span className="text-sm font-medium text-sand">
                  Loading map data...
                </span>
              </div>
            </div>
          ) : (
            /* Placeholder ready for interactive map (#1139) */
            <div
              className="flex h-full w-full flex-col items-center justify-center py-20"
              data-testid="map-placeholder"
            >
              <svg
                className="mb-4 h-16 w-16 text-sand"
                fill="none"
                viewBox="0 0 24 24"
                stroke="currentColor"
                strokeWidth={1.5}
                aria-hidden="true"
              >
                <path
                  strokeLinecap="round"
                  strokeLinejoin="round"
                  d="M9 6.75V15m6-6v8.25m.503 3.498 4.875-2.437c.381-.19.622-.58.622-1.006V4.82c0-.836-.88-1.38-1.628-1.006l-3.869 1.934c-.317.159-.69.159-1.006 0L9.503 3.252a1.125 1.125 0 0 0-1.006 0L3.622 5.689C3.24 5.88 3 6.27 3 6.695V19.18c0 .836.88 1.38 1.628 1.006l3.869-1.934c.317-.159.69-.159 1.006 0l4.994 2.497c.317.158.69.158 1.006 0Z"
                />
              </svg>
              <p className="text-lg font-semibold text-ink-soft">
                Map ready to explore
              </p>
              <p className="mt-1 text-sm text-sand">
                Zoom and pan controls will be available after map integration.
              </p>
            </div>
          )}
        </div>

        {/* Event count hint */}
        <div className="mt-6 text-sm text-sand">
          <p>
            Interactive event markers, clustering, and search are coming soon.
          </p>
        </div>
      </div>

      <Footer />
    </main>
  );
}