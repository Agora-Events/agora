import type { Metadata } from "next";
import { Navbar } from "@/components/layout/navbar";
import { PricingSection } from "@/components/landing/pricing-section";
import { Footer } from "@/components/layout/footer";
import { buildMetadata } from "@/components/layout/seo";

export const metadata: Metadata = buildMetadata({
  title: "Pricing",
  description: "Compare Agora plans and fees for event organizers.",
  path: "/pricing",
});

export default function PricingPage() {
  return (
    <main className="flex flex-col min-h-screen bg-base">
      <Navbar />
      <div className="pt-20">
        <PricingSection />
      </div>
      <Footer />
    </main>
  );
}
