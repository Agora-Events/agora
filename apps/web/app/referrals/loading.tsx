import React from "react";

export default function ReferralsLoading() {
  return (
    <div className="flex flex-col min-h-screen bg-base animate-pulse">
      {/* Navbar Skeleton */}
      <div className="w-full h-16 bg-surface border-b border-border-warm/50 mb-6" />

      <main className="grow w-full max-w-7xl mx-auto px-4 sm:px-6 lg:px-8 py-8 space-y-8">
        {/* Title Block Skeleton */}
        <div className="space-y-3 border-b border-border-warm pb-6">
          <div className="h-4 bg-surface rounded w-28" />
          <div className="h-8 bg-surface rounded-lg w-64" />
          <div className="h-4 bg-surface rounded-md w-full max-w-lg" />
        </div>

        {/* 4 KPI Metric Card Skeletons */}
        <div className="grid grid-cols-1 sm:grid-cols-2 lg:grid-cols-4 gap-4">
          {Array.from({ length: 4 }).map((_, idx) => (
            <div
              key={idx}
              className="h-28 rounded-2xl bg-surface border border-border-warm/60 p-5 space-y-3"
            >
              <div className="h-3 bg-surface-alt rounded w-24" />
              <div className="h-6 bg-surface-alt rounded w-16" />
            </div>
          ))}
        </div>

        {/* Link Generator Box Skeleton */}
        <div className="h-44 rounded-2xl bg-surface border border-border-warm/60 p-6 space-y-4" />

        {/* Table Skeleton */}
        <div className="h-64 rounded-2xl bg-surface border border-border-warm/60 p-6 space-y-3" />
      </main>

      {/* Footer Skeleton */}
      <div className="w-full h-40 bg-surface mt-12 border-t border-border-warm/50" />
    </div>
  );
}
