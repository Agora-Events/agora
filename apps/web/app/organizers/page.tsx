"use client";

import { Navbar } from "@/components/layout/navbar";
import { useTranslations } from "next-intl";
import { Footer } from "@/components/layout/footer";
import { OrganizerComponent } from "@/components/events/organizer-component";
import { ErrorToast } from "@/components/ui/Toast";
import { useState } from "react";

export default function OrganizersPage() {
  const t = useTranslations("discover");
  const [toastMessage, setToastMessage] = useState<string | null>(null);

  const showErrorToast = (message: string) => {
    setToastMessage(message);
  };

  return (
    <main className="flex flex-col min-h-screen bg-base">
      <Navbar />
      {toastMessage && (
        <div className="fixed top-4 right-4 z-[60]">
          <ErrorToast
            message={toastMessage}
            onDismiss={() => setToastMessage(null)}
          />
        </div>
      )}
      <div className="p-10 pl-45 hidden lg:block bg-base">
        <div className="flex justify-start items-center gap-4 p-5 pb-10">
          <h1 className="font-semibold md:text-4xl pl-3">{t("exploreOrganizers")}</h1>
        </div>
      </div>
      <OrganizerComponent onError={showErrorToast} />
      <Footer />
    </main>
  );
}