import { Navbar } from "@/components/layout/navbar";
import { CategorySection } from "@/components/events/category-section";
import { PopularEventsSection } from "@/components/events/popular-events-section";
import { OrganizerComponent } from "@/components/events/organizer-component";
import { Footer } from "@/components/layout/footer";

export default function DiscoverPage() {
  return (
    <main className="flex flex-col min-h-screen bg-[#FFFBE9]">
      <Navbar />
      <CategorySection />
      <PopularEventsSection />
      <OrganizerComponent/>
      <Footer />
    </main>
  );
}
