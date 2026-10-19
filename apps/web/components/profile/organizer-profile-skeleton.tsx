export function OrganizerProfileSkeleton() {
  return (
    <div
      data-testid="organizer-profile-skeleton"
      aria-busy="true"
      aria-live="polite"
      className="w-full max-w-6xl mx-auto px-4 py-10 md:py-20"
    >
      <div aria-hidden="true" className="flex flex-col md:flex-row gap-10 items-start">
        <div className="w-full md:w-[32%] md:sticky md:top-24 flex flex-col gap-4">
          <aside className="bg-white rounded-2xl p-6 flex flex-col gap-6 shadow-sm border border-border-warm animate-pulse">
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
          </aside>
        </div>

        <div className="flex-1 flex flex-col gap-8 w-full">
          <section className="bg-white rounded-3xl border-2 border-black shadow-[-8px_8px_0_rgba(0,0,0,1)] overflow-hidden animate-pulse">
            <div className="px-8 pt-8 pb-4 border-b-2 border-black bg-surface">
              <div className="h-8 w-48 rounded bg-surface-alt" />
              <div className="mt-2 h-4 w-56 rounded bg-surface-alt" />
            </div>

            <div className="p-8">
              <div className="grid grid-cols-1 md:grid-cols-2 xl:grid-cols-3 gap-6">
                {Array.from({ length: 3 }).map((_, index) => (
                  <div key={index} className="overflow-hidden rounded-2xl border-2 border-black bg-surface-alt">
                    <div className="h-48 w-full bg-surface-alt" />
                    <div className="space-y-4 p-5">
                      <div className="h-4 w-20 rounded bg-surface" />
                      <div className="h-6 w-3/4 rounded bg-surface" />
                      <div className="h-4 w-full rounded bg-surface" />
                      <div className="h-4 w-5/6 rounded bg-surface" />
                    </div>
                  </div>
                ))}
              </div>
            </div>
          </section>
        </div>
      </div>
    </div>
  );
}
