"use client";

import { useEffect, useState } from "react";
import { ChevronUp } from "@/components/ui/icons";

/**
 * A floating "Back to top" button that appears once the user has scrolled
 * past one viewport. Smooth-scrolls back to the top, honouring the user's
 * `prefers-reduced-motion` preference by falling back to an instant jump.
 *
 * Uses a passive scroll listener that is removed on unmount to avoid leaks.
 */
export function BackToTop() {
  const [visible, setVisible] = useState(false);

  useEffect(() => {
    const onScroll = () => {
      setVisible(window.scrollY > window.innerHeight);
    };

    // Initial state in case the page is loaded mid-scroll (e.g. bfcache).
    onScroll();

    window.addEventListener("scroll", onScroll, { passive: true });
    return () => window.removeEventListener("scroll", onScroll);
  }, []);

  const scrollToTop = () => {
    const prefersReducedMotion = window.matchMedia("(prefers-reduced-motion: reduce)").matches;

    window.scrollTo({
      top: 0,
      behavior: prefersReducedMotion ? "instant" : "smooth",
    });
  };

  return (
    <button
      type="button"
      aria-label="Back to top"
      onClick={scrollToTop}
      className={[
        "fixed bottom-6 right-6 z-50 flex h-12 w-12 items-center justify-center rounded-full",
        "border-2 border-black bg-[#FDDA23] text-black shadow-[-3px_3px_0px_0px_rgba(0,0,0,1)]",
        "transition-all duration-200 focus-visible:outline-none focus-visible:ring-4 focus-visible:ring-[#FDDA23]/60",
        "hover:-translate-x-[1px] hover:translate-y-[1px] hover:shadow-[-1px_1px_0px_0px_rgba(0,0,0,1)]",
        "active:translate-y-[2px] active:shadow-none",
        visible ? "opacity-100 pointer-events-auto" : "opacity-0 pointer-events-none",
      ].join(" ")}
    >
      <ChevronUp size={24} className="text-black" />
    </button>
  );
}
