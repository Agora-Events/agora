"use client";

import { useEffect, useState } from "react";
import { Button } from "@/components/ui/button";

type CookieConsent = "accepted" | "declined";

const CONSENT_KEY = "cookie_consent";

export function CookieBanner() {
  const [isVisible, setIsVisible] = useState(false);

  useEffect(() => {
    const timer = window.setTimeout(() => {
      setIsVisible(!localStorage.getItem(CONSENT_KEY));
    }, 0);

    return () => window.clearTimeout(timer);
  }, []);

  const saveChoice = (choice: CookieConsent) => {
    localStorage.setItem(CONSENT_KEY, choice);
    setIsVisible(false);
  };

  if (!isVisible) {
    return null;
  }

  return (
    <section
      aria-label="Cookie consent"
      className="fixed inset-x-3 bottom-3 z-50 mx-auto flex max-w-5xl flex-col gap-4 rounded-2xl border border-black bg-white p-4 text-black shadow-[-5px_5px_0_rgba(0,0,0,1)] sm:inset-x-6 sm:bottom-6 sm:flex-row sm:items-center sm:justify-between sm:p-5"
    >
      <div className="max-w-3xl">
        <h2 className="text-base font-semibold text-ink-deep">Cookies on Agora</h2>
        <p className="mt-1 text-sm leading-6 text-black/70">
          We use cookies to keep the event experience reliable and understand what is working.
          You can accept or decline non-essential cookies.
        </p>
      </div>

      <div className="flex shrink-0 gap-3">
        <Button
          type="button"
          variant="secondary"
          className="h-11 px-5 text-sm"
          onClick={() => saveChoice("declined")}
        >
          Decline
        </Button>
        <Button
          type="button"
          variant="primary"
          className="h-11 px-5 text-sm"
          onClick={() => saveChoice("accepted")}
        >
          Accept
        </Button>
      </div>
    </section>
  );
}
