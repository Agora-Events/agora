"use client";

import { useEffect, useState } from "react";
import { COOKIE_CONSENT_KEY } from "@/lib/constants";
import type { CookieConsent } from "@/components/layout/cookie-banner";

export default function SettingsPage() {
  const [consent, setConsent] = useState<CookieConsent | null>(null);

  useEffect(() => {
    try {
      const storedConsent = window.localStorage.getItem(COOKIE_CONSENT_KEY);
      if (storedConsent === "accepted" || storedConsent === "declined") {
        setConsent(storedConsent);
      }
    } catch {
      setConsent(null);
    }
  }, []);

  const updateConsent = (nextConsent: CookieConsent) => {
    try {
      window.localStorage.setItem(COOKIE_CONSENT_KEY, nextConsent);
    } catch {
      // Keep the selected value visible for this visit if storage is unavailable.
    }
    setConsent(nextConsent);
  };

  return (
    <main className="min-h-screen bg-[#FFFBE9] px-4 py-12">
      <div className="mx-auto max-w-3xl">
        <h1 className="mb-8 text-4xl font-bold italic">Settings</h1>
        <section
          id="cookie-preferences"
          className="rounded-xl border border-black bg-white p-6 shadow-[-4px_4px_0px_0px_rgba(0,0,0,1)]"
        >
          <div className="mb-5">
            <h2 className="text-xl font-bold">Cookie preferences</h2>
            <p className="mt-2 text-sm text-gray-600">
              Current choice: {consent ? consent[0].toUpperCase() + consent.slice(1) : "Not set"}
            </p>
          </div>
          <div className="flex flex-wrap gap-3">
            <button
              type="button"
              onClick={() => updateConsent("accepted")}
              aria-pressed={consent === "accepted"}
              className={`rounded-lg border border-black px-4 py-2 text-sm font-semibold ${consent === "accepted" ? "bg-black text-white" : "bg-white hover:bg-gray-100"}`}
            >
              Accepted
            </button>
            <button
              type="button"
              onClick={() => updateConsent("declined")}
              aria-pressed={consent === "declined"}
              className={`rounded-lg border border-black px-4 py-2 text-sm font-semibold ${consent === "declined" ? "bg-black text-white" : "bg-white hover:bg-gray-100"}`}
            >
              Declined
            </button>
          </div>
        </section>
      </div>
    </main>
  );
}
