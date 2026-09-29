import React from "react";

export default function OrganizerAnalyticsLoading() {
  return (
    <div aria-busy="true" className="flex flex-col min-h-screen bg-base animate-pulse">
      <span className="sr-only">Loading</span>

      {/* Navbar skeleton */}
      <div className="w-full h-16 bg-surface border-b border-border-warm/50" />

      <div className="mx-auto w-full max-w-7xl flex-1 px-4 py-10 sm:px-6 lg:py-16 space-y-10">
        {/* Page heading */}
        <div className="mb-9 space-y-2">
          <div className="h-4 bg-surface rounded w-32" />
          <div className="h-10 bg-surface rounded-lg w-64 mt-2" />
          <div className="h-5 bg-surface rounded-md w-96 mt-1" />
        </div>

        {/* KPI cards row */}
        <div className="grid grid-cols-1 sm:grid-cols-2 lg:grid-cols-4 gap-6">
          {Array.from({ length: 4 }).map((_, i) => (
            <div
              key={i}
              className="rounded-2xl border border-border-warm/50 bg-surface p-6 space-y-3"
            >
              <div className="h-4 bg-base rounded w-2/3" />
              <div className="h-8 bg-base rounded-lg w-1/2" />
              <div className="h-3 bg-base rounded w-3/4" />
            </div>
          ))}
        </div>

        {/* Charts row */}
        <div className="grid grid-cols-1 lg:grid-cols-2 gap-6">
          {Array.from({ length: 2 }).map((_, i) => (
            <div
              key={i}
              className="rounded-2xl border border-border-warm/50 bg-surface p-6 space-y-4"
            >
              <div className="h-5 bg-base rounded-lg w-40" />
              <div className="h-48 bg-base rounded-xl w-full" />
            </div>
          ))}
        </div>

        {/* Table / list skeleton */}
        <div className="rounded-2xl border border-border-warm/50 bg-surface p-6 space-y-4">
          <div className="h-5 bg-base rounded-lg w-36" />
          {Array.from({ length: 5 }).map((_, i) => (
            <div key={i} className="flex items-center gap-4">
              <div className="h-4 bg-base rounded w-1/4" />
              <div className="h-4 bg-base rounded w-1/3" />
              <div className="h-4 bg-base rounded w-1/5 ml-auto" />
            </div>
          ))}
        </div>
      </div>

      {/* Footer skeleton */}
      <div className="w-full h-40 bg-surface border-t border-border-warm/50" />
    </div>
  );
}
