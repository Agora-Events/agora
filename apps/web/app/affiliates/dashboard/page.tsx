import { AffiliateMetricsDashboard } from "@/components/analytics/affiliate-metrics-dashboard";
import { Footer } from "@/components/layout/footer";
import { Navbar } from "@/components/layout/navbar";

export default function AffiliateDashboardPage() {
  return (
    <main className="flex min-h-screen flex-col bg-base">
      <Navbar />
      <div className="mx-auto w-full max-w-7xl flex-1 px-4 py-10 sm:px-6 lg:py-16">
        <div className="mb-9">
          <p className="text-sm font-bold uppercase tracking-[0.18em] text-violet-700">Affiliate Insights</p>
          <h1 className="mt-2 text-4xl font-bold text-ink-deep sm:text-5xl">Performance Metrics</h1>
          <p className="mt-3 max-w-2xl text-gray-600">Track your referral clicks, sales, and total commission earned.</p>
        </div>
        
        <AffiliateMetricsDashboard affiliateId="demo123" />
        
        {/* We can also render an empty state one right below to show empty state handling, but we'll stick to a standard dashboard */}
      </div>
      <Footer />
    </main>
  );
}
