import React from "react";
import Image from "next/image";
import Link from "next/link";

interface EmptyStateProps {
  /** Heading text */
  title: string;
  /** Supporting message / body copy */
  message: string;
  /** Label for the optional call-to-action button */
  ctaLabel?: string;
  /** Where the CTA button navigates to */
  ctaLink?: string;
  /** Override the default illustration src */
  illustrationSrc?: string;
}

/**
 * EmptyState — displayed on the Discover page when no events match the
 * active filters.  Accepts a title, message, and an optional CTA that
 * links to the event-creation flow.
 *
 * Resolves issue #938.
 */
export function EmptyState({
  title,
  message,
  ctaLabel,
  ctaLink,
  illustrationSrc = "/icons/404-illustration.svg",
}: EmptyStateProps) {
  return (
    <div
      data-testid="empty-state"
      className="flex flex-col items-center justify-center gap-6 py-20 px-6 text-center"
    >
      {/* Illustration */}
      <div className="w-40 h-40 flex items-center justify-center">
        <Image
          src={illustrationSrc}
          width={160}
          height={160}
          alt="No events illustration"
          className="w-full h-full object-contain opacity-80"
        />
      </div>

      {/* Text content */}
      <div className="flex flex-col items-center gap-2 max-w-sm">
        <h3 className="text-xl font-semibold text-ink-deep">{title}</h3>
        <p className="text-sm text-muted-text leading-relaxed">{message}</p>
      </div>

      {/* Optional CTA */}
      {ctaLabel && ctaLink && (
        <Link
          href={ctaLink}
          className="inline-flex items-center gap-2 px-6 py-3 rounded-full bg-black text-white font-semibold text-sm shadow-[-4px_4px_0px_0px_rgba(0,0,0,0.4)] hover:-translate-x-[2px] hover:translate-y-[2px] active:-translate-x-[4px] active:translate-y-[4px] transition-transform"
        >
          {ctaLabel}
        </Link>
      )}
    </div>
  );
}
