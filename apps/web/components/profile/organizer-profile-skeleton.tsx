"use client";

/**
 * Full-page skeleton for the organizer profile page, shown while the
 * organizer's data is fetched to avoid a blank flash.
 *
 * Mirrors the final layout: circular avatar + two text bars, a bio line,
 * a three-item stats row, and a 3-column event card grid.
 */
export function OrganizerProfileSkeleton() {
  return (
    <div aria-hidden="true" className="flex flex-col md:flex-row gap-10 items-start animate-pulse">
      {/* Left sidebar */}
      <div className="w-full md:w-[32%] md:sticky md:top-24 flex flex-col gap-4">
        <div className="bg-white rounded-2xl p-6 flex flex-col gap-6 border border-border-warm">
          <div className="flex flex-col items-center gap-3">
            <div className="h-24 w-24 rounded-full bg-surface-alt" />
            <div className="h-5 w-36 rounded bg-surface-alt" />
            <div className="h-4 w-24 rounded bg-surface-alt" />
          </div>
          <div className="h-4 w-40 rounded bg-surface-alt" />
          <div className="flex justify-around border-t border-b border-border-warm py-4">
            <div className="h-12 w-16 rounded bg-surface-alt" />
            <div className="w-px bg-border-warm" />
            <div className="h-12 w-16 rounded bg-surface-alt" />
          </div>
        </div>
        <div className="h-12 w-full rounded-2xl bg-surface-alt border border-border-warm" />
      </div>

      {/* Right column */}
      <div className="flex-1 flex flex-col gap-10 w-full">
        <div className="bg-white rounded-3xl border-2 border-black shadow-[-8px_8px_0_rgba(0,0,0,1)] overflow-hidden">
          <div className="px-8 pt-8 pb-4 border-b-2 border-black bg-surface">
            <div className="h-7 w-44 rounded bg-surface-alt" />
            <div className="h-4 w-64 mt-2 rounded bg-surface-alt" />
          </div>
          <div className="p-8 flex flex-col gap-6">
            <div className="h-32 bg-surface-alt rounded-2xl border-2 border-black" />
            <div className="h-32 bg-surface-alt rounded-2xl border-2 border-black" />
          </div>
        </div>

        {/* 3-column event card grid */}
        <div className="grid grid-cols-1 md:grid-cols-2 lg:grid-cols-3 gap-6">
          {Array.from({ length: 3 }).map((_, i) => (
            <div
              key={i}
              className="rounded-2xl overflow-hidden border-2 border-black shadow-[-4px_4px_0_rgba(0,0,0,1)]"
            >
              <div className="aspect-16/10 w-full bg-surface-alt" />
              <div className="p-4 flex flex-col gap-3">
                <div className="h-5 w-3/4 rounded bg-surface-alt" />
                <div className="h-4 w-1/2 rounded bg-surface-alt" />
                <div className="h-4 w-2/3 rounded bg-surface-alt" />
              </div>
            </div>
          ))}
        </div>
      </div>
    </div>
  );
}
