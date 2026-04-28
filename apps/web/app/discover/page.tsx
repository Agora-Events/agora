"use client";

import { useState } from "react";
import { Navbar } from "@/components/layout/navbar";
import { CategorySection } from "@/components/events/category-section";
import { PopularEventsSection } from "@/components/events/popular-events-section";
import { OrganizerComponent } from "@/components/events/organizer-component";
import { Footer } from "@/components/layout/footer";

export default function DiscoverPage() {
  const [toastMessage, setToastMessage] = useState<string | null>(null);

  const showErrorToast = (message: string) => {
    setToastMessage(message);
    window.setTimeout(() => setToastMessage(null), 3500);
  };

  return (
    <main className="flex flex-col min-h-screen bg-[#FFFBE9]">
      <Navbar />
      {toastMessage && (
        <div className="fixed top-4 right-4 z-[60] rounded-lg bg-black px-4 py-3 text-sm text-white shadow-lg">
          {toastMessage}
        </div>
      )}
      <CategorySection onError={showErrorToast} />
      <PopularEventsSection onError={showErrorToast} />
      <OrganizerComponent onError={showErrorToast} />
      <Footer />
    </main>
  );
}
