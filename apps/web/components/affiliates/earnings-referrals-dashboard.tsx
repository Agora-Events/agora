"use client";

import { useState, useEffect } from "react";
import { AnalyticsKpiCard } from "@/components/analytics/analytics-kpi-card";

// ─── Types ────────────────────────────────────────────────────────────────────

interface EarningsReferralsData {
  totalEarnings: number;
  pendingPayouts: number;
  withdrawable: number;
  lifetimeClicks: number;
  totalSales: number;
  conversionRate: number;
  referrals: ReferralLink[];
  payouts: PayoutEntry[];
}

interface ReferralLink {
  id: string;
  eventName: string;
  clicks: number;
  sales: number;
  commission: number;
  createdAt: string;
}

interface PayoutEntry {
  id: string;
  amount: number;
  status: "pending" | "completed" | "failed";
  date: string;
  method: string;
}

// ─── Mock placeholder data (ready for backend integration) ─────────────────────

const MOCK_DATA: EarningsReferralsData = {
  totalEarnings: 0,
  pendingPayouts: 0,
  withdrawable: 0,
  lifetimeClicks: 0,
  totalSales: 0,
  conversionRate: 0,
  referrals: [],
  payouts: [],
};

// ─── Loading skeleton ──────────────────────────────────────────────────────────

function EarningsSummarySkeleton() {
  return (
    <div className="grid grid-cols-1 sm:grid-cols-2 lg:grid-cols-4 gap-6">
      {[1, 2, 3, 4].map((i) => (
        <div
          key={i}
          className="h-32 animate-pulse rounded-2xl border-2 border-black bg-surface shadow-[-6px_6px_0_rgba(0,0,0,1)]"
          role="status"
          aria-label="Loading metric"
        />
      ))}
    </div>
  );
}

function ReferralTableSkeleton() {
  return (
    <div className="space-y-4">
      {[1, 2, 3].map((i) => (
        <div
          key={i}
          className="h-16 animate-pulse rounded-xl border-2 border-black bg-surface"
          role="status"
          aria-label="Loading referral row"
        />
      ))}
    </div>
  );
}

function PayoutHistorySkeleton() {
  return (
    <div className="h-40 animate-pulse rounded-2xl border-2 border-black bg-surface" role="status" aria-label="Loading payout history" />
  );
}

// ─── Empty state ───────────────────────────────────────────────────────────────

function NoActivityState() {
  return (
    <div className="rounded-3xl border-2 border-black bg-white p-10 text-center shadow-[-5px_5px_0_rgba(0,0,0,1)] flex flex-col items-center justify-center">
      <div className="w-16 h-16 rounded-full bg-surface border-2 border-black flex items-center justify-center mb-4">
        <svg width="28" height="28" viewBox="0 0 24 24" fill="none" stroke="currentColor" strokeWidth="2" className="text-gray-400">
          <path d="M12 2v20M17 5H9.5a3.5 3.5 0 0 0 0 7h5a3.5 3.5 0 0 1 0 7H6" strokeLinecap="round" strokeLinejoin="round"/>
        </svg>
      </div>
      <h3 className="text-xl font-bold text-ink-deep mb-2">No Affiliate Activity Yet</h3>
      <p className="text-gray-500 max-w-md mx-auto">
        Share your referral links to start tracking earnings, clicks, and commissions. Generate a referral link below to get started.
      </p>
    </div>
  );
}

// ─── Placeholder section wrapper ────────────────────────────────────────────────

function PlaceholderSection({ title, description, height = "h-64" }: { title: string; description?: string; height?: string }) {
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

// ─── Referral Performance Table ─────────────────────────────────────────────────

function ReferralPerformanceTable({ referrals }: { referrals: ReferralLink[] }) {
  if (referrals.length === 0) {
    return (
      <div className="text-center py-8 text-gray-500">
        <p className="text-sm">No referral links generated yet.</p>
        <p className="text-xs mt-1">Generate a referral link to start tracking performance.</p>
      </div>
    );
  }

  return (
    <div className="overflow-x-auto">
      <table className="w-full text-left text-sm">
        <thead>
          <tr className="border-b-2 border-black text-xs uppercase tracking-wider text-gray-500">
            <th className="pb-3 pr-4 font-semibold">Event</th>
            <th className="pb-3 pr-4 font-semibold">Clicks</th>
            <th className="pb-3 pr-4 font-semibold">Sales</th>
            <th className="pb-3 pr-4 font-semibold">Commission</th>
            <th className="pb-3 font-semibold">Created</th>
          </tr>
        </thead>
        <tbody>
          {referrals.map((ref) => (
            <tr key={ref.id} className="border-b border-black/10">
              <td className="py-3 pr-4 font-medium text-ink-deep">{ref.eventName}</td>
              <td className="py-3 pr-4">{ref.clicks.toLocaleString()}</td>
              <td className="py-3 pr-4">{ref.sales}</td>
              <td className="py-3 pr-4 font-medium text-green-600">${ref.commission.toFixed(2)}</td>
              <td className="py-3 text-gray-500">{ref.createdAt}</td>
            </tr>
          ))}
        </tbody>
      </table>
    </div>
  );
}

// ─── Payout History Table ───────────────────────────────────────────────────────

function PayoutHistoryTable({ payouts }: { payouts: PayoutEntry[] }) {
  if (payouts.length === 0) {
    return (
      <div className="text-center py-8 text-gray-500">
        <p className="text-sm">No payouts yet.</p>
        <p className="text-xs mt-1">Earnings will be listed here once they are processed.</p>
      </div>
    );
  }

  return (
    <div className="overflow-x-auto">
      <table className="w-full text-left text-sm">
        <thead>
          <tr className="border-b-2 border-black text-xs uppercase tracking-wider text-gray-500">
            <th className="pb-3 pr-4 font-semibold">Date</th>
            <th className="pb-3 pr-4 font-semibold">Amount</th>
            <th className="pb-3 pr-4 font-semibold">Method</th>
            <th className="pb-3 font-semibold">Status</th>
          </tr>
        </thead>
        <tbody>
          {payouts.map((p) => (
            <tr key={p.id} className="border-b border-black/10">
              <td className="py-3 pr-4 text-gray-600">{p.date}</td>
              <td className="py-3 pr-4 font-medium text-ink-deep">${p.amount.toFixed(2)}</td>
              <td className="py-3 pr-4 text-gray-500">{p.method}</td>
              <td className="py-3">
                <span
                  className={`inline-flex items-center rounded-full px-2.5 py-0.5 text-xs font-medium ${
                    p.status === "completed"
                      ? "bg-green-100 text-green-700"
                      : p.status === "pending"
                        ? "bg-yellow-100 text-yellow-700"
                        : "bg-red-100 text-red-700"
                  }`}
                >
                  {p.status}
                </span>
              </td>
            </tr>
          ))}
        </tbody>
      </table>
    </div>
  );
}

// ─── Main Dashboard Component ───────────────────────────────────────────────────

interface EarningsReferralsDashboardProps {
  affiliateId?: string;
}

export function EarningsReferralsDashboard({ affiliateId = "default" }: EarningsReferralsDashboardProps) {
  const [data, setData] = useState<EarningsReferralsData | null>(null);
  const [isLoading, setIsLoading] = useState(true);
  const [error, setError] = useState<string | null>(null);

  useEffect(() => {
    let cancelled = false;

    async function fetchData() {
      setIsLoading(true);
      setError(null);

      try {
        const res = await fetch(`/api/affiliates/${encodeURIComponent(affiliateId)}/metrics`);
        if (!res.ok) {
          // Backend not ready yet — use placeholder mock data
          if (!cancelled) {
            setData(MOCK_DATA);
            setIsLoading(false);
          }
          return;
        }
        const json = await res.json();
        if (!cancelled) {
          // Transform API response into full dashboard data shape
          // (placeholder — expand when #1150 backend is complete)
          setData({
            totalEarnings: json.totalCommissionEarned ?? 0,
            pendingPayouts: 0,
            withdrawable: json.totalCommissionEarned ?? 0,
            lifetimeClicks: json.referralClicks ?? 0,
            totalSales: json.ticketSales ?? 0,
            conversionRate: json.conversionRate ?? 0,
            referrals: [],
            payouts: [],
          });
        }
      } catch {
        if (!cancelled) {
          // Network error — fall back to mock
          setData(MOCK_DATA);
        }
      } finally {
        if (!cancelled) {
          setIsLoading(false);
        }
      }
    }

    fetchData();
    return () => { cancelled = true; };
  }, [affiliateId]);

  // ── Loading state ──────────────────────────────────────────────────────────

  if (isLoading) {
    return (
      <div className="space-y-8">
        {/* Earnings summary skeleton */}
        <div>
          <div className="mb-4 h-6 w-48 animate-pulse rounded bg-surface" />
          <EarningsSummarySkeleton />
        </div>

        {/* Referral performance skeleton */}
        <div>
          <div className="mb-4 h-6 w-56 animate-pulse rounded bg-surface" />
          <ReferralTableSkeleton />
        </div>

        {/* Payout history skeleton */}
        <div>
          <div className="mb-4 h-6 w-40 animate-pulse rounded bg-surface" />
          <PayoutHistorySkeleton />
        </div>
      </div>
    );
  }

  // ── Error state ────────────────────────────────────────────────────────────

  if (error) {
    return (
      <div className="rounded-3xl border-2 border-black bg-white p-10 text-center shadow-[-5px_5px_0_rgba(0,0,0,1)]" role="alert">
        <p className="font-bold text-ink-deep">Dashboard could not be loaded.</p>
        <p className="mt-1 text-sm text-gray-500">Refresh the page to try again.</p>
      </div>
    );
  }

  // ── Empty state (no activity) ──────────────────────────────────────────────

  if (!data || (data.totalEarnings === 0 && data.lifetimeClicks === 0 && data.totalSales === 0)) {
    return (
      <div className="space-y-8">
        <EarningsSummarySkeleton />
        <NoActivityState />
      </div>
    );
  }

  // ── Data state ─────────────────────────────────────────────────────────────

  return (
    <div className="space-y-10">
      {/* Earnings Summary */}
      <section>
        <h2 className="text-2xl font-bold text-ink-deep mb-5">Earnings Summary</h2>
        <div className="grid grid-cols-1 sm:grid-cols-2 lg:grid-cols-4 gap-6">
          <AnalyticsKpiCard
            title="Total Earnings"
            value={`$${data.totalEarnings.toLocaleString(undefined, { minimumFractionDigits: 2, maximumFractionDigits: 2 })}`}
            icon={
              <svg width="20" height="20" viewBox="0 0 24 24" fill="none" stroke="currentColor" strokeWidth="2">
                <path d="M12 2v20M17 5H9.5a3.5 3.5 0 0 0 0 7h5a3.5 3.5 0 0 1 0 7H6" strokeLinecap="round" strokeLinejoin="round"/>
              </svg>
            }
            accent={true}
          />
          <AnalyticsKpiCard
            title="Pending Payouts"
            value={`$${data.pendingPayouts.toLocaleString(undefined, { minimumFractionDigits: 2, maximumFractionDigits: 2 })}`}
            icon={
              <svg width="20" height="20" viewBox="0 0 24 24" fill="none" stroke="currentColor" strokeWidth="2">
                <path d="M12 6v6l4 2" strokeLinecap="round" strokeLinejoin="round"/>
                <circle cx="12" cy="12" r="10" strokeLinecap="round" strokeLinejoin="round"/>
              </svg>
            }
          />
          <AnalyticsKpiCard
            title="Withdrawable"
            value={`$${data.withdrawable.toLocaleString(undefined, { minimumFractionDigits: 2, maximumFractionDigits: 2 })}`}
            subtitle="Available for withdrawal"
            icon={
              <svg width="20" height="20" viewBox="0 0 24 24" fill="none" stroke="currentColor" strokeWidth="2">
                <rect x="1" y="4" width="22" height="16" rx="2" ry="2" strokeLinecap="round" strokeLinejoin="round"/>
                <line x1="1" y1="10" x2="23" y2="10" strokeLinecap="round" strokeLinejoin="round"/>
              </svg>
            }
          />
          <AnalyticsKpiCard
            title="Lifetime Clicks"
            value={data.lifetimeClicks.toLocaleString()}
            subtitle={`${data.conversionRate.toFixed(1)}% conversion`}
            icon={
              <svg width="20" height="20" viewBox="0 0 24 24" fill="none" stroke="currentColor" strokeWidth="2">
                <path d="M15 3h6v6M9 21H3v-6M21 3l-7 7M3 21l7-7" strokeLinecap="round" strokeLinejoin="round"/>
              </svg>
            }
          />
        </div>
      </section>

      {/* Referral Performance */}
      <section>
        <div className="flex items-center justify-between mb-5">
          <h2 className="text-2xl font-bold text-ink-deep">Referral Performance</h2>
          <span className="text-xs text-gray-400 border border-gray-300 rounded-full px-3 py-1">Data integration pending</span>
        </div>
        <div className="rounded-2xl border-2 border-black bg-white p-6 shadow-[-6px_6px_0_rgba(0,0,0,1)]">
          <ReferralPerformanceTable referrals={data.referrals} />
        </div>
      </section>

      {/* Payout History */}
      <section>
        <div className="flex items-center justify-between mb-5">
          <h2 className="text-2xl font-bold text-ink-deep">Payout History</h2>
          <span className="text-xs text-gray-400 border border-gray-300 rounded-full px-3 py-1">Coming soon</span>
        </div>
        <PlaceholderSection
          title="Payout History"
          description="Payout records will appear here once the affiliate registration system is live."
          height="h-40"
        />
      </section>
    </div>
  );
}