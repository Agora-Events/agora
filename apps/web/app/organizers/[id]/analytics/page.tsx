import { AnalyticsDashboard } from "@/components/analytics/analytics-dashboard";
import { Footer } from "@/components/layout/footer";
import { Navbar } from "@/components/layout/navbar";

export default async function OrganizerAnalyticsPage({
  params,
}: {
  params: Promise<{ id: string }>;
}) {
  const { id } = await params;

  return (
    <main className="flex min-h-screen flex-col bg-base">
      <Navbar />
      <div className="mx-auto w-full max-w-7xl flex-1 px-4 py-10 sm:px-6 lg:py-16">
        <div className="mb-9">
          <p className="text-sm font-bold uppercase tracking-[0.18em] text-violet-700">Organizer insights</p>
          <h1 className="mt-2 text-4xl font-bold text-ink-deep sm:text-5xl">Event analytics</h1>
          <p className="mt-3 max-w-2xl text-gray-600">Track sales performance and understand how visitors become attendees.</p>
        </div>
        <AnalyticsDashboard organizerWallet={id} />
      </div>
      <Footer />
    </main>
  );
}
