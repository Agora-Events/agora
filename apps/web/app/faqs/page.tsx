import { Navbar } from "@/components/layout/navbar";
import { FAQSection } from "@/components/landing/faq-section";
import { Footer } from "@/components/layout/footer";

export default function FAQPage() {
  return (
    <main className="flex flex-col min-h-screen bg-[#FFFBE9]">
      <Navbar />
      <div className="pt-20">
        <FAQSection />
      </div>
      <Footer />
    </main>
  );
}
