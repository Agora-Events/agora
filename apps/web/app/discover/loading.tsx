import React from "react";
import { EventCardSkeleton } from "@/components/events/event-card-skeleton";

export default function DiscoverLoading() {
  return (
    <div className="flex flex-col min-h-screen bg-base animate-pulse">
      {/* Navbar skeleton header */}
      <div className="w-full h-16 bg-surface border-b border-border-warm/50 mb-6" />

      <main className="grow w-full max-w-7xl mx-auto px-4 sm:px-6 lg:px-8 py-8 space-y-8">
        {/* Category Header Skeleton */}
        <div className="space-y-4">
          <div className="h-8 bg-surface rounded-lg w-48" />
          <div className="flex items-center gap-3 overflow-x-auto pb-2">
            {Array.from({ length: 6 }).map((_, i) => (
              <div key={i} className="h-10 w-28 bg-surface rounded-full flex-shrink-0" />
            ))}
          </div>
        </div>

        {/* Popular Events Header Skeleton */}
        <div className="flex items-center justify-between">
          <div className="h-7 bg-surface rounded-lg w-40" />
          <div className="h-5 bg-surface rounded-md w-24" />
        </div>

        {/* 6 Event Cards Grid */}
        <div className="grid grid-cols-1 md:grid-cols-2 lg:grid-cols-3 gap-6">
          {Array.from({ length: 6 }).map((_, idx) => (
            <EventCardSkeleton key={idx} />
          ))}
        </div>
      </main>

      {/* Footer skeleton */}
      <div className="w-full h-40 bg-surface mt-12 border-t border-border-warm/50" />
    </div>
  );
}
