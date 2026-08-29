"use client";

import { useState, useEffect } from "react";
import { Navbar } from "@/components/layout/navbar";
import { Footer } from "@/components/layout/footer";
import { ErrorBoundary } from "@/components/layout/error-boundary";
import {
  AnalyticsKpiCard,
  AnalyticsChartPlaceholder,
  useAnalytics,
} from "@/components/analytics/analytics-kpi-card";
import Image from "next/image";

interface EventOption {
  id: string;
  title: string;
}

function formatCurrency(value: number) {
  return new Intl.NumberFormat("en-US", {
    style: "currency",
    currency: "USD",
    minimumFractionDigits: 0,
    maximumFractionDigits: 2,
  }).format(value);
}

function formatPercent(value: number) {
  return `${(value * 100).toFixed(1)}%`;
}

function TicketIcon() {
  return (
    <svg width="18" height="18" viewBox="0 0 24 24" fill="none" stroke="currentColor" strokeWidth="2.2">
      <path d="M2 9l10-6 10 6-10 6L2 9z" strokeLinecap="round" strokeLinejoin="round" />
      <path d="M12 3v18" strokeLinecap="round" />
      <path d="M2 13l10 5 10-5" strokeLinecap="round" strokeLinejoin="round" />
    </svg>
  );
}

function RevenueIcon() {
  return (
    <svg width="18" height="18" viewBox="0 0 24 24" fill="none" stroke="currentColor" strokeWidth="2.2">
      <circle cx="12" cy="12" r="10" />
      <path d="M12 6v12M8 10l4-2 4 2" strokeLinecap="round" strokeLinejoin="round" />
    </svg>
  );
}

function RemainingIcon() {
  return (
    <svg width="18" height="18" viewBox="0 0 24 24" fill="none" stroke="currentColor" strokeWidth="2.2">
      <rect x="3" y="3" width="18" height="18" rx="3" />
      <path d="M9 12l2 2 4-4" strokeLinecap="round" strokeLinejoin="round" />
    </svg>
  );
}

function SellThroughIcon() {
  return (
    <svg width="18" height="18" viewBox="0 0 24 24" fill="none" stroke="currentColor" strokeWidth="2.2">
      <path d="M22 12h-4l-3 9L9 3l-3 9H2" strokeLinecap="round" strokeLinejoin="round" />
    </svg>
  );
}

function BuyersIcon() {
  return (
    <svg width="18" height="18" viewBox="0 0 24 24" fill="none" stroke="currentColor" strokeWidth="2.2">
      <path d="M17 21v-2a4 4 0 00-4-4H5a4 4 0 00-4 4v2" strokeLinecap="round" strokeLinejoin="round" />
      <circle cx="9" cy="7" r="4" />
      <path d="M23 21v-2a4 4 0 00-3-3.87M16 3.13a4 4 0 010 7.75" strokeLinecap="round" strokeLinejoin="round" />
    </svg>
  );
}

function EventSelector({
  events,
  selectedId,
  onChange,
}: {
  events: EventOption[];
  selectedId: string;
  onChange: (id: string) => void;
}) {
  return (
    <div className="flex flex-col sm:flex-row sm:items-center gap-3">
      <label htmlFor="event-select" className="text-sm font-semibold text-gray-600 whitespace-nowrap">
        Select Event
      </label>
      <select
        id="event-select"
        value={selectedId}
        onChange={(e) => onChange(e.target.value)}
        className="rounded-xl border-2 border-black bg-white px-4 py-2.5 text-sm font-medium text-ink-deep shadow-[-3px_3px_0_rgba(0,0,0,1)] focus:outline-none focus:ring-2 focus:ring-accent focus:ring-offset-2 transition-all cursor-pointer"
      >
        {events.map((evt) => (
          <option key={evt.id} value={evt.id}>
            {evt.title}
          </option>
        ))}
      </select>
    </div>
  );
}

function ChartErrorBoundary({ children }: { children: React.ReactNode }) {
  const [retryKey, setRetryKey] = useState(0);

  return (
    <ErrorBoundary
      key={retryKey}
      fallback={
        <div className="rounded-2xl border-2 border-black bg-white p-6 shadow-[-4px_4px_0_rgba(0,0,0,1)] text-center flex flex-col items-center justify-center min-h-[200px]">
          <p className="text-base font-semibold text-ink-deep mb-3">This chart couldn't be loaded</p>
          <button
            type="button"
            onClick={() => setRetryKey((prev) => prev + 1)}
            className="inline-flex items-center justify-center px-4 py-2 text-sm font-semibold text-white bg-black rounded-xl border border-black hover:bg-gray-800 transition-colors shadow-[-2px_2px_0_rgba(0,0,0,1)] cursor-pointer"
          >
            Try again
          </button>
        </div>
      }
    >
      {children}
    </ErrorBoundary>
  );
}

export default function OrganizerAnalyticsDashboard() {
  const [events, setEvents] = useState<EventOption[]>([]);
  const [selectedEventId, setSelectedEventId] = useState<string>("");
  const [eventsLoading, setEventsLoading] = useState(true);
  const [eventsError, setEventsError] = useState<string | null>(null);

  const { data, loading, error } = useAnalytics({
    eventId: selectedEventId || undefined,
  });

  useEffect(() => {
    let cancelled = false;

    async function loadEvents() {
      setEventsLoading(true);
      setEventsError(null);

      try {
        const res = await fetch("/api/v1/events?type=my&tab=hosting");
        if (!res.ok) {
          const body = await res.json().catch(() => ({}));
          throw new Error(body.error || `Request failed with status ${res.status}`);
        }
        const json = await res.json();
        const items: EventOption[] = (json.items || []).map((evt: { id: string; title: string }) => ({
          id: evt.id,
          title: evt.title,
        }));

        if (!cancelled) {
          setEvents(items);
          if (items.length > 0 && !selectedEventId) {
            setSelectedEventId(items[0].id);
          }
        }
      } catch (err) {
        if (!cancelled) {
          setEventsError(err instanceof Error ? err.message : "Failed to load events");
        }
      } finally {
        if (!cancelled) {
          setEventsLoading(false);
        }
      }
    }

    loadEvents();

    return () => {
      cancelled = true;
    };
  }, []);

  const hasData = data && !loading && !error;

  return (
    <main className="flex flex-col min-h-screen bg-base">
      <Navbar />

      <div className="flex-1 w-full">
        <div className="max-w-6xl mx-auto px-4 py-8 md:py-14">
          {/* Header */}
          <div className="flex flex-col md:flex-row md:items-end md:justify-between gap-6 mb-8">
            <div>
              <h1 className="text-3xl md:text-4xl font-bold text-ink-deep italic">Analytics Dashboard</h1>
              <p className="text-sm text-gray-500 mt-1.5">
                {hasData
                  ? `Insights for "${data.eventTitle}"`
                  : "Select an event to view its analytics"}
              </p>
            </div>

            {events.length > 0 && (
              <EventSelector
                events={events}
                selectedId={selectedEventId}
                onChange={setSelectedEventId}
              />
            )}
          </div>

          {/* Events loading state */}
          {eventsLoading && (
            <div className="grid grid-cols-1 sm:grid-cols-2 lg:grid-cols-3 gap-5">
              {Array.from({ length: 6 }).map((_, i) => (
                <div key={i} className="h-28 rounded-2xl border-2 border-black bg-white animate-pulse shadow-[-4px_4px_0_rgba(0,0,0,0.08)]" />
              ))}
            </div>
          )}

          {/* Events error state */}
          {eventsError && (
            <div className="rounded-2xl border-2 border-black bg-red-50 p-6 shadow-[-4px_4px_0_rgba(0,0,0,1)]">
              <p className="text-sm font-medium text-red-700">Failed to load events: {eventsError}</p>
            </div>
          )}

          {/* No events state */}
          {!eventsLoading && !eventsError && events.length === 0 && (
            <div className="rounded-2xl border-2 border-black bg-white p-12 shadow-[-6px_6px_0_rgba(0,0,0,1)] text-center">
              <div className="w-16 h-16 rounded-full bg-surface mx-auto flex items-center justify-center mb-4 border-2 border-black">
                <Image src="/icons/calendar.svg" width={32} height={32} alt="Calendar" className="invert" />
              </div>
              <h3 className="text-lg font-bold text-ink-deep italic mb-1">No hosted events yet</h3>
              <p className="text-sm text-gray-500 max-w-sm mx-auto">
                Create your first event to start tracking analytics and gaining insights.
              </p>
            </div>
          )}

          {/* Analytics content */}
          {!eventsLoading && selectedEventId && (
            <>
              {/* Loading analytics state */}
              {loading && (
                <div className="grid grid-cols-1 sm:grid-cols-2 lg:grid-cols-3 gap-5">
                  {Array.from({ length: 6 }).map((_, i) => (
                    <div key={i} className="h-28 rounded-2xl border-2 border-black bg-white animate-pulse shadow-[-4px_4px_0_rgba(0,0,0,0.08)]" />
                  ))}
                </div>
              )}

              {/* Analytics error state */}
              {error && (
                <div className="rounded-2xl border-2 border-black bg-red-50 p-6 shadow-[-4px_4px_0_rgba(0,0,0,1)]">
                  <p className="text-sm font-medium text-red-700">{error}</p>
                </div>
              )}

              {/* Analytics data display */}
              {hasData && (
                <div className="flex flex-col gap-8">
                  {/* KPI cards */}
                  <section aria-label="Key metrics">
                    <div className="grid grid-cols-1 sm:grid-cols-2 lg:grid-cols-3 gap-5">
                      <AnalyticsKpiCard
                        title="Tickets Sold"
                        value={data.kpi.totalTicketsSold.toLocaleString()}
                        subtitle={`of ${data.kpi.totalCapacity.toLocaleString()} total capacity`}
                        icon={<TicketIcon />}
                        accent
                      />
                      <AnalyticsKpiCard
                        title="Total Revenue"
                        value={formatCurrency(data.kpi.totalRevenue)}
                        subtitle={`at $${data.kpi.ticketPrice.toFixed(2)} per ticket`}
                        icon={<RevenueIcon />}
                      />
                      <AnalyticsKpiCard
                        title="Remaining Tickets"
                        value={data.kpi.remainingTickets.toLocaleString()}
                        subtitle={data.kpi.remainingTickets === 0 ? "Sold out" : "still available"}
                        icon={<RemainingIcon />}
                      />
                      <AnalyticsKpiCard
                        title="Sell-Through Rate"
                        value={formatPercent(data.kpi.sellThroughRate)}
                        subtitle={`${data.kpi.uniqueBuyers} unique buyer${data.kpi.uniqueBuyers !== 1 ? "s" : ""}`}
                        icon={<SellThroughIcon />}
                      />
                      <AnalyticsKpiCard
                        title="Unique Buyers"
                        value={data.kpi.uniqueBuyers.toLocaleString()}
                        subtitle="distinct purchasers"
                        icon={<BuyersIcon />}
                      />
                    </div>
                  </section>

                  {/* Charts section */}
                  <section aria-label="Analytics charts" className="grid grid-cols-1 lg:grid-cols-2 gap-6">
                    <ChartErrorBoundary>
                      <AnalyticsChartPlaceholder
                        title="Attendance Trends"
                        description="Ticket sales over time. Shows how tickets are selling day by day leading up to your event."
                        height="h-72"
                      />
                    </ChartErrorBoundary>
                    <ChartErrorBoundary>
                      <AnalyticsChartPlaceholder
                        title="Ticket Tier Popularity"
                        description="Breakdown of ticket tiers by price range. Understand which price points attract the most buyers."
                        height="h-72"
                      />
                    </ChartErrorBoundary>
                  </section>
                </div>
              )}
            </>
          )}
        </div>
      </div>

      <Footer />
    </main>
  );
}
