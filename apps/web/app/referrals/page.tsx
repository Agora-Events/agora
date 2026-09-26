"use client";

import React, { useState } from "react";
import { Navbar } from "@/components/layout/navbar";
import { Footer } from "@/components/layout/footer";
import { ReferralLinkGenerator } from "@/components/affiliates/referral-link-generator";

interface ReferralMetric {
  id: string;
  label: string;
  value: string;
  change: string;
  isPositive: boolean;
}

const mockMetrics: ReferralMetric[] = [
  { id: "clicks", label: "Total Referral Clicks", value: "1,248", change: "+14.2%", isPositive: true },
  { id: "conversions", label: "Ticket Sales via Link", value: "84", change: "+8.5%", isPositive: true },
  { id: "rate", label: "Conversion Rate", value: "6.7%", change: "+0.8%", isPositive: true },
  { id: "earnings", label: "Total Commission Earned", value: "$420.00", change: "+22.4%", isPositive: true },
];

interface RecentReferral {
  id: string;
  eventTitle: string;
  buyerAddress: string;
  commission: string;
  date: string;
  status: "completed" | "pending";
}

const mockRecentReferrals: RecentReferral[] = [
  { id: "ref-1", eventTitle: "Stellar Dev Summit 2026", buyerAddress: "G9X1...82KA", commission: "$15.00", date: "2 hours ago", status: "completed" },
  { id: "ref-2", eventTitle: "Web3 Hackathon Bay Area", buyerAddress: "G4B2...990L", commission: "$10.00", date: "5 hours ago", status: "completed" },
  { id: "ref-3", eventTitle: "Soroban Smart Contracts Workshop", buyerAddress: "G7M3...11PP", commission: "$7.50", date: "1 day ago", status: "pending" },
];

export default function ReferralDashboardPage() {
  const [metrics] = useState<ReferralMetric[]>(mockMetrics);
  const [referrals] = useState<RecentReferral[]>(mockRecentReferrals);

  return (
    <main className="flex min-h-screen flex-col bg-base">
      <Navbar />

      <div className="mx-auto w-full max-w-7xl flex-1 px-4 py-8 sm:px-6 lg:py-12 space-y-8">
        {/* Page Header */}
        <div className="border-b border-border-warm pb-6">
          <p className="text-xs font-bold uppercase tracking-[0.2em] text-accent">Affiliate Portal</p>
          <h1 className="mt-1 text-3xl font-extrabold text-ink-soft sm:text-4xl">Earnings & Referrals</h1>
          <p className="mt-2 text-sm text-muted-text max-w-2xl">
            Track performance metrics for your unique event referral links, analyze conversion rates, and view payout earnings.
          </p>
        </div>

        {/* KPI Metrics Grid */}
        <div className="grid grid-cols-1 sm:grid-cols-2 lg:grid-cols-4 gap-4">
          {metrics.map((metric) => (
            <div
              key={metric.id}
              className="rounded-2xl bg-white p-5 border border-border-warm shadow-[-4px_4px_0_rgba(0,0,0,0.06)] space-y-2"
            >
              <p className="text-xs font-semibold text-muted-text">{metric.label}</p>
              <div className="flex items-baseline justify-between">
                <span className="text-2xl font-bold text-ink-soft">{metric.value}</span>
                <span className="text-xs font-bold text-emerald-600 bg-emerald-50 px-2 py-0.5 rounded-full">
                  {metric.change}
                </span>
              </div>
            </div>
          ))}
        </div>

        {/* Referral Link Generator */}
        <div className="rounded-2xl bg-white p-6 border border-border-warm shadow-sm">
          <h2 className="text-lg font-bold text-ink-soft mb-2">Create & Share Referral Links</h2>
          <p className="text-xs text-muted-text mb-4">
            Select an event to generate your tracked referral link and start earning 5% commission on every ticket purchase.
          </p>
          <ReferralLinkGenerator defaultAffiliateCode="agora-partner" />
        </div>

        {/* Recent Referrals & Commission Breakdown Table */}
        <div className="rounded-2xl bg-white border border-border-warm shadow-sm overflow-hidden">
          <div className="px-6 py-4 border-b border-border-warm flex items-center justify-between">
            <h2 className="text-base font-bold text-ink-soft">Recent Referral Activity</h2>
            <span className="text-xs font-semibold text-muted-text">Showing last 3 transactions</span>
          </div>

          <div className="overflow-x-auto">
            <table className="w-full text-left text-xs">
              <thead className="bg-surface text-muted-text border-b border-border-warm font-semibold">
                <tr>
                  <th className="py-3 px-6">Event Title</th>
                  <th className="py-3 px-6">Buyer Wallet</th>
                  <th className="py-3 px-6">Commission</th>
                  <th className="py-3 px-6">Date</th>
                  <th className="py-3 px-6 text-right">Status</th>
                </tr>
              </thead>
              <tbody className="divide-y divide-border-warm/60 text-ink-soft">
                {referrals.map((item) => (
                  <tr key={item.id} className="hover:bg-surface/50 transition-colors">
                    <td className="py-3.5 px-6 font-semibold">{item.eventTitle}</td>
                    <td className="py-3.5 px-6 font-mono text-muted-text">{item.buyerAddress}</td>
                    <td className="py-3.5 px-6 font-bold text-accent">{item.commission}</td>
                    <td className="py-3.5 px-6 text-muted-text">{item.date}</td>
                    <td className="py-3.5 px-6 text-right">
                      <span
                        className={`inline-flex items-center px-2.5 py-0.5 rounded-full text-[10px] font-bold ${
                          item.status === "completed"
                            ? "bg-emerald-100 text-emerald-800"
                            : "bg-amber-100 text-amber-800"
                        }`}
                      >
                        {item.status.toUpperCase()}
                      </span>
                    </td>
                  </tr>
                ))}
              </tbody>
            </table>
          </div>
        </div>
      </div>

      <Footer />
    </main>
  );
}
