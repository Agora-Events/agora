import { Navbar } from "@/components/layout/navbar";
import { PricingSection } from "@/components/landing/pricing-section";
import { Footer } from "@/components/layout/footer";

export default function PricingPage() {
  return (
    <main className="flex flex-col min-h-screen bg-[#FFFBE9]">
      <Navbar />
      <div className="pt-20">
        <PricingSection />
      </div>
      <Footer />
    </main>
  );
}
