"use client";

/**
 * InfoTooltip — Issue #1577
 *
 * Small, dependency-free info icon that reveals an explanatory tooltip on
 * hover/focus. Built for inline use next to labels (e.g. ticket prices)
 * that benefit from a short clarification without cluttering the UI.
 *
 * No tooltip/overlay primitive existed in the design system yet, so this
 * keeps things minimal: a `<button>` (keyboard + screen-reader accessible)
 * paired with a `role="tooltip"` panel shown via Tailwind's `group`
 * hover/focus-within utilities — no extra JS state, no new dependency.
 */

import { Info } from "@/components/ui/icons";

export interface InfoTooltipProps {
  /** The explanatory text shown in the tooltip. */
  text: string;
  /** Accessible label for the trigger button (defaults to a generic hint). */
  label?: string;
  /** Additional classes for the trigger wrapper. */
  className?: string;
}

export function InfoTooltip({
  text,
  label = "More information",
  className = "",
}: InfoTooltipProps) {
  return (
    <span className={`relative inline-flex group ${className}`}>
      <button
        type="button"
        aria-label={label}
        className="inline-flex items-center justify-center text-black/40 hover:text-black/70 focus:text-black/70 focus:outline-none transition-colors"
      >
        <Info size={15} />
      </button>
      <span
        role="tooltip"
        className="pointer-events-none absolute bottom-full left-1/2 mb-2 w-max max-w-[220px] -translate-x-1/2 rounded-lg bg-black px-3 py-2 text-center text-xs font-medium text-white opacity-0 shadow-lg transition-opacity duration-150 group-hover:opacity-100 group-focus-within:opacity-100 z-20"
      >
        {text}
        <span className="absolute left-1/2 top-full -translate-x-1/2 border-4 border-transparent border-t-black" />
      </span>
    </span>
  );
}
