"use client";

import { Navbar } from "@/components/layout/navbar";
import { Footer } from "@/components/layout/footer";
import { OrganizerComponent } from "@/components/events/organizer-component";
import { useState } from "react";

export default function OrganizersPage() {
  const [toastMessage, setToastMessage] = useState<string | null>(null);

  const showErrorToast = (message: string) => {
    setToastMessage(message);
    window.setTimeout(() => setToastMessage(null), 3500);
  };

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
          <h1 className="font-semibold md:text-4xl pl-3">Explore organizers</h1>
        </div>
      </div>
      <OrganizerComponent onError={showErrorToast} />
      <Footer />
    </main>
  );
}