import React from "react";

export default function EventDetailLoading() {
  return (
    <div className="flex flex-col min-h-screen bg-base animate-pulse">
      {/* Navbar skeleton */}
      <div className="w-full h-16 bg-surface border-b border-border-warm/50 mb-6" />

      <main className="grow w-full max-w-7xl mx-auto px-4 sm:px-6 lg:px-8 py-8 space-y-8">
        {/* Breadcrumb Skeleton */}
        <div className="flex items-center gap-2">
          <div className="h-4 bg-surface rounded w-20" />
          <div className="h-4 bg-surface rounded w-4" />
          <div className="h-4 bg-surface rounded w-36" />
        </div>

        {/* Hero Section + Sidebar Layout */}
        <div className="grid grid-cols-1 lg:grid-cols-3 gap-8">
          {/* Left Column: Event Main Content */}
          <div className="lg:col-span-2 space-y-6">
            {/* Event Hero Banner Image Skeleton */}
            <div className="w-full h-64 sm:h-96 bg-surface rounded-2xl border border-border-warm/50" />

            {/* Title & Metadata */}
            <div className="space-y-3">
              <div className="h-8 bg-surface rounded-lg w-3/4" />
              <div className="h-4 bg-surface rounded-md w-1/2" />
            </div>

            {/* Host / Organizer Card Skeleton */}
            <div className="flex items-center gap-4 p-4 rounded-xl bg-surface border border-border-warm/50">
              <div className="w-12 h-12 rounded-full bg-surface-alt flex-shrink-0" />
              <div className="space-y-2 flex-1">
                <div className="h-4 bg-surface-alt rounded w-32" />
                <div className="h-3 bg-surface-alt rounded w-24" />
              </div>
            </div>

            {/* Description Paragraph Skeletons */}
            <div className="space-y-3 pt-4">
              <div className="h-4 bg-surface rounded w-full" />
              <div className="h-4 bg-surface rounded w-5/6" />
              <div className="h-4 bg-surface rounded w-4/6" />
            </div>

            {/* Map Block Skeleton */}
            <div className="w-full h-60 bg-surface rounded-xl border border-border-warm/50" />
          </div>

          {/* Right Column: Sidebar Registration Box */}
          <div className="lg:col-span-1">
            <div className="p-6 rounded-2xl bg-surface border border-border-warm space-y-6 shadow-sm sticky top-24">
              <div className="space-y-2">
                <div className="h-6 bg-surface-alt rounded w-1/3" />
                <div className="h-8 bg-surface-alt rounded w-1/2" />
              </div>
              <div className="h-12 bg-accent/20 rounded-xl w-full" />
              <div className="space-y-3 pt-2">
                <div className="h-4 bg-surface-alt rounded w-3/4" />
                <div className="h-4 bg-surface-alt rounded w-2/3" />
              </div>
            </div>
          </div>
        </div>
      </main>

      {/* Footer skeleton */}
      <div className="w-full h-40 bg-surface mt-12 border-t border-border-warm/50" />
    </div>
  );
}
