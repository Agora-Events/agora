"use client";

import useSWR from "swr";
import { DailyTicketSalesChart, DailyTicketSalesPoint } from "./daily-ticket-sales-chart";
import { RevenueComparisonChart, RevenueComparisonPoint } from "./revenue-comparison-chart";
import { SalesFunnel, SalesFunnelData } from "./sales-funnel";
import { TicketTierPopularityChart, TicketTierPopularityPoint } from "./ticket-tier-popularity-chart";

interface AnalyticsResponse {
  funnel: SalesFunnelData;
  dailySales: DailyTicketSalesPoint[];
  revenueComparison: RevenueComparisonPoint[];
  ticketTiers: TicketTierPopularityPoint[];
}

const fetcher = async (url: string): Promise<AnalyticsResponse> => {
  const response = await fetch(url);
  if (!response.ok) throw new Error("Unable to load analytics");
  return response.json();
};

export function AnalyticsDashboard({ organizerWallet }: { organizerWallet: string }) {
  const { data, error, isLoading } = useSWR<AnalyticsResponse>(
    `/api/analytics?organizerWallet=${encodeURIComponent(organizerWallet)}`,
    fetcher,
  );

  if (isLoading) {
    return <div className="h-96 animate-pulse rounded-3xl border-2 border-black bg-surface" role="status" aria-label="Loading analytics" />;
  }

  if (error || !data) {
    return (
      <div className="rounded-3xl border-2 border-black bg-white p-10 text-center shadow-[-5px_5px_0_rgba(0,0,0,1)]" role="alert">
        <p className="font-bold text-ink-deep">Analytics could not be loaded.</p>
        <p className="mt-1 text-sm text-gray-500">Refresh the page to try again.</p>
      </div>
    );
  }

  return (
    <div className="grid min-w-0 gap-7 lg:grid-cols-2">
      <div className="lg:col-span-2"><SalesFunnel data={data.funnel} /></div>
      <RevenueComparisonChart data={data.revenueComparison} />
      <TicketTierPopularityChart data={data.ticketTiers} />
      <div className="lg:col-span-2"><DailyTicketSalesChart data={data.dailySales} /></div>
    </div>
  );
}
