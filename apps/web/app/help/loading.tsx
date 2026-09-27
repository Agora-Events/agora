import React from "react";

export default function HelpLoading() {
  return (
    <div aria-busy="true" className="flex flex-col min-h-screen bg-base animate-pulse">
      <span className="sr-only">Loading</span>

      {/* Navbar skeleton */}
      <div className="w-full h-16 bg-surface border-b border-border-warm/50 mb-6" />

      <main className="grow w-full max-w-7xl mx-auto px-4 sm:px-6 lg:px-8 py-10 space-y-10">
        {/* Page heading */}
        <div className="space-y-3">
          <div className="h-8 bg-surface rounded-lg w-56" />
          <div className="h-5 bg-surface rounded-md w-80" />
        </div>

        {/* Search bar skeleton */}
        <div className="h-12 bg-surface rounded-xl w-full max-w-xl" />

        {/* Category grid — mirrors the 8-card grid in page.tsx */}
        <div className="grid grid-cols-1 sm:grid-cols-2 lg:grid-cols-3 xl:grid-cols-4 gap-6">
          {Array.from({ length: 8 }).map((_, i) => (
            <div
              key={i}
              className="flex flex-col gap-3 rounded-2xl border border-border-warm/50 bg-surface p-6"
            >
              {/* Icon placeholder */}
              <div className="h-10 w-10 rounded-xl bg-base" />
              {/* Title */}
              <div className="h-5 bg-base rounded-md w-3/4" />
              {/* Description */}
              <div className="h-4 bg-base rounded-md w-full" />
              <div className="h-4 bg-base rounded-md w-2/3" />
              {/* Article count badge */}
              <div className="h-4 bg-base rounded-md w-1/3 mt-auto" />
            </div>
          ))}
        </div>
      </main>

      {/* Footer skeleton */}
      <div className="w-full h-40 bg-surface mt-12 border-t border-border-warm/50" />
    </div>
  );
}
