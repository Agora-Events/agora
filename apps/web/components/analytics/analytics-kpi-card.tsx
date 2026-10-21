"use client";

import { useState, useEffect } from "react";

type AnalyticsData = {
  eventId: string;
  eventTitle: string;
  generatedAt: string;
  kpi: {
    totalTicketsSold: number;
    totalRevenue: number;
    remainingTickets: number;
    sellThroughRate: number;
    uniqueBuyers: number;
    ticketPrice: number;
    totalCapacity: number;
  };
  ticketTierPopularity: {
    tier: string;
    label: string;
    eventPrice: number;
    sold: number;
  }[];
  attendanceTrends: {
    date: string;
    ticketsSold: number;
    revenue: number;
  }[];
};

type KpiCardProps = {
  title: string;
  value: string | number;
  subtitle?: string;
  icon: React.ReactNode;
  accent?: boolean;
};

export function AnalyticsKpiCard({ title, value, subtitle, icon, accent = false }: KpiCardProps) {
  return (
    <div className="rounded-2xl border-2 border-black bg-white p-5 shadow-[-6px_6px_0_rgba(0,0,0,1)] flex flex-col gap-3">
      <div className="flex items-center justify-between">
        <span className="text-xs font-semibold uppercase tracking-wide text-gray-500">{title}</span>
        <div className={`w-9 h-9 rounded-lg flex items-center justify-center ${accent ? "bg-accent" : "bg-surface"}`}>
          {icon}
        </div>
      </div>
      <div>
        <p className="text-2xl font-bold text-ink-deep leading-none">{value}</p>
        {subtitle && <p className="text-xs text-gray-500 mt-1.5">{subtitle}</p>}
      </div>
    </div>
  );
}

export function AnalyticsChartPlaceholder({ title, description, height = "h-64" }: { title: string; description?: string; height?: string }) {
  return (
    <div className={`${height} rounded-2xl border-2 border-black bg-surface p-6 shadow-[-6px_6px_0_rgba(0,0,0,1)] flex flex-col items-center justify-center text-center`}>
      <div className="w-12 h-12 rounded-full bg-white border-2 border-black flex items-center justify-center mb-4">
        <svg width="24" height="24" viewBox="0 0 24 24" fill="none" stroke="currentColor" strokeWidth="2" className="text-gray-400">
          <path d="M3 3v18h18" strokeLinecap="round" strokeLinejoin="round" />
          <path d="M7 16l4-6 4 4 5-8" strokeLinecap="round" strokeLinejoin="round" />
        </svg>
      </div>
      <h3 className="text-lg font-bold text-ink-deep italic">{title}</h3>
      {description && <p className="text-sm text-gray-500 mt-1 max-w-sm">{description}</p>}
      <span className="text-xs text-gray-400 mt-4 border border-gray-300 rounded-full px-3 py-1">Coming soon</span>
    </div>
  );
}

type UseAnalyticsOptions = {
  eventId?: string;
  refreshInterval?: number;
};

export function useAnalytics(options: UseAnalyticsOptions = {}) {
  const { eventId, refreshInterval } = options;
  const [data, setData] = useState<AnalyticsData | null>(null);
  const [loading, setLoading] = useState(true);
  const [error, setError] = useState<string | null>(null);

  useEffect(() => {
    if (!eventId) {
      setLoading(false);
      return;
    }

    // Captured with an explicit `string` type so the nested fetchAnalytics
    // closure below doesn't lose the narrowing from the guard above — a
    // `function` declaration's captured variables aren't re-narrowed by TS.
    const currentEventId: string = eventId;

    let cancelled = false;

    async function fetchAnalytics() {
      setLoading(true);
      setError(null);

      try {
        const res = await fetch(`/api/v1/events/${encodeURIComponent(currentEventId)}/analytics`);
        if (!res.ok) {
          const body = await res.json().catch(() => ({}));
          throw new Error(body.error || `Request failed with status ${res.status}`);
        }
        const json = await res.json();
        if (!cancelled) {
          setData(json);
        }
      } catch (err) {
        if (!cancelled) {
          setError(err instanceof Error ? err.message : "Failed to load analytics");
        }
      } finally {
        if (!cancelled) {
          setLoading(false);
        }
      }
    }

    fetchAnalytics();

    if (refreshInterval && refreshInterval > 0) {
      const intervalId = setInterval(fetchAnalytics, refreshInterval);
      return () => {
        cancelled = true;
        clearInterval(intervalId);
      };
    }

    return () => {
      cancelled = true;
    };
  }, [eventId, refreshInterval]);

  return { data, loading, error };
}

export { type AnalyticsData, type KpiCardProps };
