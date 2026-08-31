import React from "react";
import { Navbar } from "@/components/layout/navbar";
import { Footer } from "@/components/layout/footer";

export default function MapDiscoveryLoading() {
  return (
    <main
      id="main-content"
      tabIndex={-1}
      className="flex min-h-screen flex-col bg-base"
    >
      <Navbar />

      <div className="relative flex h-full w-full grow flex-col md:flex-row lg:h-[calc(100vh-91px)] lg:overflow-hidden">
        {/* Map skeleton */}
        <div className="relative min-h-[50vh] w-full grow bg-surface/40 md:min-h-0 md:basis-3/5 lg:basis-2/3">
          <div className="absolute inset-0 flex items-center justify-center animate-pulse">
            <div className="flex flex-col items-center gap-3">
              <div className="h-14 w-14 rounded-full bg-black/10" />
              <div className="h-4 w-48 rounded-md bg-black/10" />
              <div className="h-3 w-64 rounded-md bg-black/10" />
            </div>
          </div>
        </div>

        {/* Event list skeleton */}
        <aside className="w-full shrink-0 border-t border-border-warm bg-base md:basis-2/5 md:border-l md:border-t-0 lg:basis-1/3">
          <div className="flex h-full flex-col">
            <div className="flex items-center justify-between border-b border-border-warm px-5 py-4">
              <div className="h-5 w-36 animate-pulse rounded-md bg-black/10" />
              <div className="h-6 w-6 animate-pulse rounded-full bg-black/10" />
            </div>
            <div className="grow space-y-3 overflow-y-auto p-4">
              {Array.from({ length: 5 }).map((_, i) => (
                <div
                  key={i}
                  className="flex items-start gap-3 rounded-2xl border border-border-warm p-4 animate-pulse"
                >
                  <div className="h-9 w-9 shrink-0 rounded-full bg-black/10" />
                  <div className="min-w-0 flex-1 space-y-2">
                    <div className="h-4 w-3/4 rounded-md bg-black/10" />
                    <div className="h-3 w-1/2 rounded-md bg-black/10" />
                  </div>
                </div>
              ))}
            </div>
          </div>
        </aside>
      </div>

      <Footer />
    </main>
  );
}
