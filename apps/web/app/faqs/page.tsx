import type { Metadata } from "next";
import { Navbar } from "@/components/layout/navbar";
import { FAQSection } from "@/components/landing/faq-section";
import { Footer } from "@/components/layout/footer";

export const metadata: Metadata = {
  title: { absolute: "FAQs | Agora" },
  description: "Answers to common questions about tickets, payments and Stellar.",
  openGraph: {
    title: "FAQs | Agora",
    description: "Answers to common questions about tickets, payments and Stellar.",
  },
};

export default function FAQPage() {
  return (
    <main className="flex flex-col min-h-screen bg-base">
      <Navbar />
      <div className="pt-20">
        <FAQSection />
      </div>
      <Footer />
    </main>
  );
}
