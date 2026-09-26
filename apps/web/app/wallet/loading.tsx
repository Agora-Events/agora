import React from "react";

function TicketCardSkeletonItem() {
  return (
    <div className="flex items-stretch gap-4 rounded-xl border border-border-warm bg-white p-4 shadow-[-4px_4px_0_rgba(0,0,0,0.08)] animate-pulse">
      {/* Thumbnail */}
      <div className="flex-shrink-0 w-20 h-20 sm:w-24 sm:h-24 rounded-lg bg-surface" />
      {/* Ticket Details */}
      <div className="flex-1 min-w-0 flex flex-col justify-between gap-2">
        <div className="space-y-2">
          <div className="h-4 bg-surface rounded w-3/4" />
          <div className="h-3 bg-surface rounded w-1/2" />
          <div className="h-3 bg-surface rounded w-2/5" />
        </div>
        <div className="flex items-center justify-between gap-2">
          <div className="h-5 w-16 bg-surface rounded-full" />
          <div className="h-4 w-12 bg-surface rounded" />
        </div>
      </div>
    </div>
  );
}

export default function WalletLoading() {
  return (
    <div className="flex flex-col min-h-screen bg-base animate-pulse">
      {/* Navbar skeleton */}
      <div className="w-full h-16 bg-surface border-b border-border-warm/50 mb-6" />

      <main className="grow w-full max-w-5xl mx-auto px-4 sm:px-6 lg:px-8 py-8 space-y-8">
        {/* Wallet Header Skeleton */}
        <div className="space-y-2">
          <div className="h-8 bg-surface rounded-lg w-40" />
          <div className="h-4 bg-surface rounded-md w-64" />
        </div>

        {/* Wallet Tabs Skeleton */}
        <div className="flex items-center gap-4 border-b border-border-warm/60 pb-3">
          <div className="h-8 bg-surface rounded-lg w-32" />
          <div className="h-8 bg-surface rounded-lg w-28" />
        </div>

        {/* 3 Ticket Card Skeletons */}
        <div className="space-y-4">
          <TicketCardSkeletonItem />
          <TicketCardSkeletonItem />
          <TicketCardSkeletonItem />
        </div>
      </main>

      {/* Footer skeleton */}
      <div className="w-full h-40 bg-surface mt-12 border-t border-border-warm/50" />
    </div>
  );
}
