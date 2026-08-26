"use client";

import { Suspense } from "react";
import { useRouter, useSearchParams } from "next/navigation";
import { Navbar } from "@/components/layout/navbar";
import { CategorySection } from "@/components/events/category-section";
import { PopularEventsSection } from "@/components/events/popular-events-section";
import { OrganizerComponent } from "@/components/events/organizer-component";
import { Footer } from "@/components/layout/footer";

function DiscoverContent() {
  const router = useRouter();
  const searchParams = useSearchParams();
  const category = searchParams.get("category") ?? "";
  const organizer = searchParams.get("organizer") ?? "";

  const updateFilter = (name: "category" | "organizer", value: string) => {
    const params = new URLSearchParams(searchParams.toString());

    if (value) {
      params.set(name, value);
    } else {
      params.delete(name);
    }

    const query = params.toString();
    router.replace(query ? `/discover?${query}` : "/discover", { scroll: false });
  };

  return (
    <main className="flex flex-col min-h-screen bg-[#FFFBE9]">
      <Navbar />
      <CategorySection selectedCategory={category} onCategoryChange={(value) => updateFilter("category", value)} />
      <PopularEventsSection category={category} onCategoryChange={(value) => updateFilter("category", value)} />
      <OrganizerComponent selectedOrganizer={organizer} onOrganizerChange={(value) => updateFilter("organizer", value)} />
      <Footer />
    </main>
  );
}

export default function DiscoverPage() {
  return (
    <Suspense>
      <DiscoverContent />
    </Suspense>
  );
}
