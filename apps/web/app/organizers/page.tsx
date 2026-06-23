"use client";

import { Navbar } from "@/components/layout/navbar";
import { Footer } from "@/components/layout/footer";
import { OrganizerComponent } from "@/components/events/organizer-component";
import { toast } from "sonner";

export default function OrganizersPage() {
  const showErrorToast = (message: string) => toast.error(message);

  return (
    <main className="flex flex-col min-h-screen bg-base">
      <Navbar />
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