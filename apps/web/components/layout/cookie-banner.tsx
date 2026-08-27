"use client";

import { useEffect, useState } from "react";
import { COOKIE_CONSENT_KEY } from "@/lib/constants";

export type CookieConsent = "accepted" | "declined";

export function CookieBanner() {
  const [isVisible, setIsVisible] = useState(false);

  useEffect(() => {
    try {
      setIsVisible(window.localStorage.getItem(COOKIE_CONSENT_KEY) === null);
    } catch {
      setIsVisible(true);
    }
  }, []);

  const setConsent = (consent: CookieConsent) => {
    try {
      window.localStorage.setItem(COOKIE_CONSENT_KEY, consent);
    } catch {
      // Keep the banner dismissed for this visit if storage is unavailable.
    }
    setIsVisible(false);
  };

  if (!isVisible) {
    return null;
  }

  return (
    <aside
      role="dialog"
      aria-label="Cookie consent"
      className="fixed inset-x-4 bottom-4 z-50 mx-auto flex max-w-3xl flex-col gap-4 rounded-xl border-2 border-black bg-white p-5 shadow-[-5px_5px_0px_0px_rgba(0,0,0,1)] sm:flex-row sm:items-center sm:justify-between"
    >
      <p className="text-sm leading-relaxed">
        We use cookies to improve your experience on Agora.
      </p>
      <div className="flex shrink-0 gap-2">
        <button
          type="button"
          onClick={() => setConsent("declined")}
          className="rounded-lg border border-black px-4 py-2 text-sm font-semibold hover:bg-gray-100"
        >
          Decline
        </button>
        <button
          type="button"
          onClick={() => setConsent("accepted")}
          className="rounded-lg bg-black px-4 py-2 text-sm font-semibold text-white hover:bg-gray-800"
        >
          Accept
        </button>
      </div>
    </aside>
  );
}
