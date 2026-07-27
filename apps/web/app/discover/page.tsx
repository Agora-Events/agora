"use client";

import { useState, useEffect } from "react";
import { Navbar } from "@/components/layout/navbar";
import { CategorySection } from "@/components/events/category-section";
import { PopularEventsSection } from "@/components/events/popular-events-section";
import { OrganizerComponent } from "@/components/events/organizer-component";
import { Footer } from "@/components/layout/footer";
import { EmptyState } from "@/components/ui/EmptyState";
import { fetchOrganizers, type DiscoverOrganizer } from "@/utils/api";

export default function DiscoverPage() {
  const [toastMessage, setToastMessage] = useState<string | null>(null);
  const [activeCategory, setActiveCategory] = useState<string>("All");
  const [selectedOrganizer, setSelectedOrganizer] = useState<string | null>(null);
  const [allOrganizers, setAllOrganizers] = useState<DiscoverOrganizer[]>([]);
  const [hasEvents, setHasEvents] = useState<boolean>(true);

  const showErrorToast = (message: string) => {
    setToastMessage(message);
    window.setTimeout(() => setToastMessage(null), 3500);
  };

  // Load all organizers for filtering
  useEffect(() => {
    const loadOrganizers = async () => {
      try {
        const data = await fetchOrganizers();
        setAllOrganizers(data);
      } catch {
        setAllOrganizers([]);
      }
    };

    loadOrganizers();
  }, []);

  return (
    <main className="flex flex-col min-h-screen bg-base">
      <Navbar />
      {toastMessage && (
        <div className="fixed top-4 right-4 z-[60] rounded-lg bg-black px-4 py-3 text-sm text-white shadow-lg">
          {toastMessage}
        </div>
      )}
      <div className="p-10 pl-45 hidden lg:block bg-base">
        <div className="flex justify-start items-center gap-4 p-5 pb-10">
          <h1 className="font-semibold md:text-4xl pl-3">Explore events</h1>
        </div>
      </div>
      <CategorySection 
        activeCategory={activeCategory} 
        onCategoryChange={setActiveCategory} 
        onError={showErrorToast} 
      />
      <PopularEventsSection 
        activeCategory={activeCategory} 
        onError={showErrorToast}
        onEventsChange={(count) => setHasEvents(count > 0)}
      />
      {!hasEvents && (
        <div className="px-4 lg:px-10 py-8">
          <EmptyState
            title="No events found"
            message="There are no events matching your current filters. Try a different category or be the first to create one!"
            ctaLabel="Create an Event"
            ctaLink="/events/create"
          />
        </div>
      )}
      <div className="p-10 pl-45 hidden lg:block bg-base">
        <div className="flex justify-start items-center gap-4 p-5 pb-10">
          <h2 className="font-semibold md:text-2xl pl-3">Filter by organizer</h2>
        </div>
      </div>
      <div className="px-4 lg:px-10 pb-8">
        <div className="flex flex-wrap gap-2 mb-6">
          <button
            onClick={() => setSelectedOrganizer(null)}
            className={`px-4 py-2 rounded-full text-sm font-medium transition-colors ${!selectedOrganizer ? 'bg-violet-600 text-white' : 'bg-gray-100 text-gray-700 hover:bg-gray-200'}`}
          >
            All Organizers
          </button>
          {allOrganizers.slice(0, 5).map((organizer) => (
            <button
              key={organizer.id}
              onClick={() => setSelectedOrganizer(organizer.id)}
              className={`px-4 py-2 rounded-full text-sm font-medium transition-colors ${selectedOrganizer === organizer.id ? 'bg-violet-600 text-white' : 'bg-gray-100 text-gray-700 hover:bg-gray-200'}`}
            >
              {organizer.title}
            </button>
          ))}
        </div>
      </div>
      <OrganizerComponent onError={showErrorToast} />
      <Footer />
    </main>
  );
}