"use client";

import useSWR from "swr";
import { AnalyticsKpiCard } from "./analytics-kpi-card";

interface AffiliateMetrics {
  referralClicks: number;
  ticketSales: number;
  conversionRate: number;
  totalCommissionEarned: number;
}

const fetcher = async (url: string): Promise<AffiliateMetrics> => {
  const response = await fetch(url);
  if (!response.ok) throw new Error("Unable to load affiliate metrics");
  return response.json();
};

export function AffiliateMetricsDashboard({ affiliateId = "default" }: { affiliateId?: string }) {
  const { data, error, isLoading } = useSWR<AffiliateMetrics>(
    `/api/affiliates/${encodeURIComponent(affiliateId)}/metrics`,
    fetcher
  );

  if (isLoading) {
    return (
      <div className="grid grid-cols-1 md:grid-cols-2 lg:grid-cols-4 gap-6">
        {[1, 2, 3, 4].map((i) => (
          <div key={i} className="h-32 animate-pulse rounded-2xl border-2 border-black bg-surface shadow-[-6px_6px_0_rgba(0,0,0,1)]" role="status" aria-label="Loading metric" />
        ))}
      </div>
    );
  }

  if (error || !data) {
    return (
      <div className="rounded-3xl border-2 border-black bg-white p-10 text-center shadow-[-5px_5px_0_rgba(0,0,0,1)]" role="alert">
        <p className="font-bold text-ink-deep">Affiliate metrics could not be loaded.</p>
        <p className="mt-1 text-sm text-gray-500">Refresh the page to try again.</p>
      </div>
    );
  }

  const hasData = data.referralClicks > 0 || data.ticketSales > 0 || data.totalCommissionEarned > 0;

  if (!hasData) {
    return (
      <div className="rounded-3xl border-2 border-black bg-white p-10 text-center shadow-[-5px_5px_0_rgba(0,0,0,1)] flex flex-col items-center justify-center">
        <div className="w-16 h-16 rounded-full bg-surface border-2 border-black flex items-center justify-center mb-4">
          <svg width="28" height="28" viewBox="0 0 24 24" fill="none" stroke="currentColor" strokeWidth="2" className="text-gray-400">
            <path d="M12 2v20M17 5H9.5a3.5 3.5 0 0 0 0 7h5a3.5 3.5 0 0 1 0 7H6" strokeLinecap="round" strokeLinejoin="round"/>
          </svg>
        </div>
        <h3 className="text-xl font-bold text-ink-deep mb-2">No Affiliate Activity Yet</h3>
        <p className="text-gray-500 max-w-md mx-auto">
          Share your referral links to start tracking clicks, sales, and earning commissions.
        </p>
      </div>
    );
  }

  return (
    <div className="grid grid-cols-1 md:grid-cols-2 lg:grid-cols-4 gap-6">
      <AnalyticsKpiCard
        title="Referral Clicks"
        value={data.referralClicks.toLocaleString()}
        icon={
          <svg width="20" height="20" viewBox="0 0 24 24" fill="none" stroke="currentColor" strokeWidth="2">
            <path d="M15 3h6v6M9 21H3v-6M21 3l-7 7M3 21l7-7" strokeLinecap="round" strokeLinejoin="round" />
          </svg>
        }
      />
      
      <AnalyticsKpiCard
        title="Ticket Sales"
        value={data.ticketSales.toLocaleString()}
        icon={
          <svg width="20" height="20" viewBox="0 0 24 24" fill="none" stroke="currentColor" strokeWidth="2">
            <path d="M16 4h2a2 2 0 0 1 2 2v14a2 2 0 0 1-2 2H6a2 2 0 0 1-2-2V6a2 2 0 0 1 2-2h2" strokeLinecap="round" strokeLinejoin="round" />
            <rect x="8" y="2" width="8" height="4" rx="1" ry="1" strokeLinecap="round" strokeLinejoin="round" />
          </svg>
        }
      />
      
      <AnalyticsKpiCard
        title="Conversion Rate"
        value={`${data.conversionRate.toFixed(1)}%`}
        icon={
          <svg width="20" height="20" viewBox="0 0 24 24" fill="none" stroke="currentColor" strokeWidth="2">
            <path d="M22 12h-4l-3 9L9 3l-3 9H2" strokeLinecap="round" strokeLinejoin="round" />
          </svg>
        }
      />
      
      <AnalyticsKpiCard
        title="Commission Earned"
        value={`$${data.totalCommissionEarned.toLocaleString(undefined, { minimumFractionDigits: 2, maximumFractionDigits: 2 })}`}
        accent={true}
        icon={
          <svg width="20" height="20" viewBox="0 0 24 24" fill="none" stroke="currentColor" strokeWidth="2">
            <path d="M12 2v20M17 5H9.5a3.5 3.5 0 0 0 0 7h5a3.5 3.5 0 0 1 0 7H6" strokeLinecap="round" strokeLinejoin="round" />
          </svg>
        }
      />
    </div>
  );
}
