"use client";

// apps/web/components/recommendations/RecommendedEvents.tsx
//
// Drop this anywhere in the discovery feed or homepage:
//   <RecommendedEvents />
//
// Requires:
//   - useRecommendedEvents hook (same directory)
//   - Tailwind CSS
//   - next/image, next/link
//   - User auth context that exposes `isLoggedIn`

import Image from "next/image";
import Link from "next/link";
import { useRecommendedEvents, RecommendedEvent } from "@/hooks/useRecommendedEvents";

// ── Helpers ───────────────────────────────────────────────────────────────────

function formatDate(iso: string) {
  return new Date(iso).toLocaleDateString("en-US", {
    month: "short",
    day: "numeric",
    year: "numeric",
  });
}

function formatTime(iso: string) {
  return new Date(iso).toLocaleTimeString("en-US", {
    hour: "numeric",
    minute: "2-digit",
  });
}

function formatPrice(price: number | null) {
  if (price === null || price === 0) return "Free";
  return `$${price.toFixed(2)}`;
}

// ── Sub-components ────────────────────────────────────────────────────────────

function SkeletonCard() {
  return (
    <div className="group relative flex flex-col overflow-hidden rounded-2xl border border-neutral-100 bg-white shadow-sm">
      <div className="h-44 w-full animate-pulse bg-neutral-100" />
      <div className="flex flex-1 flex-col gap-3 p-4">
        <div className="h-3 w-20 animate-pulse rounded-full bg-neutral-100" />
        <div className="h-5 w-3/4 animate-pulse rounded-lg bg-neutral-100" />
        <div className="h-3 w-1/2 animate-pulse rounded-full bg-neutral-100" />
        <div className="mt-auto flex items-center justify-between">
          <div className="h-3 w-16 animate-pulse rounded-full bg-neutral-100" />
          <div className="h-8 w-24 animate-pulse rounded-xl bg-neutral-100" />
        </div>
      </div>
    </div>
  );
}

function EventCard({ event }: { event: RecommendedEvent }) {
  const soldOut = event.tickets_remaining === 0;

  return (
    <Link
      href={`/events/${event.slug}`}
      className="group relative flex flex-col overflow-hidden rounded-2xl border border-neutral-100 bg-white shadow-sm transition-all duration-300 hover:-translate-y-0.5 hover:shadow-lg hover:border-violet-100"
    >
      {/* Banner */}
      <div className="relative h-44 w-full overflow-hidden bg-gradient-to-br from-violet-50 to-indigo-100">
        {event.banner_url ? (
          <Image
            src={event.banner_url}
            alt={event.title}
            fill
            className="object-cover transition-transform duration-500 group-hover:scale-105"
            sizes="(max-width: 640px) 100vw, (max-width: 1024px) 50vw, 33vw"
          />
        ) : (
          // Fallback gradient when no banner
          <div className="absolute inset-0 flex items-center justify-center">
            <span className="text-4xl opacity-30">🎟</span>
          </div>
        )}

        {/* Category pill */}
        <span className="absolute left-3 top-3 rounded-full bg-white/90 px-2.5 py-1 text-[11px] font-semibold tracking-wide text-violet-700 shadow-sm backdrop-blur-sm">
          {event.category_name}
        </span>

        {/* Sold-out overlay */}
        {soldOut && (
          <div className="absolute inset-0 flex items-center justify-center bg-black/50">
            <span className="rounded-lg bg-black/70 px-3 py-1 text-sm font-bold text-white">
              Sold Out
            </span>
          </div>
        )}
      </div>

      {/* Content */}
      <div className="flex flex-1 flex-col gap-2 p-4">
        {/* Date */}
        <p className="flex items-center gap-1.5 text-xs text-neutral-400">
          <CalendarIcon />
          {formatDate(event.start_time)} · {formatTime(event.start_time)}
        </p>

        {/* Title */}
        <h3 className="line-clamp-2 text-[15px] font-bold leading-snug text-neutral-900 group-hover:text-violet-700 transition-colors">
          {event.title}
        </h3>

        {/* Location */}
        {event.location && (
          <p className="flex items-center gap-1 text-xs text-neutral-400">
            <PinIcon />
            <span className="truncate">{event.location}</span>
          </p>
        )}

        {/* Organizer */}
        <div className="mt-1 flex items-center gap-2">
          {event.organizer_avatar ? (
            <Image
              src={event.organizer_avatar}
              alt={event.organizer_name}
              width={20}
              height={20}
              className="rounded-full object-cover"
            />
          ) : (
            <div className="flex h-5 w-5 items-center justify-center rounded-full bg-violet-100 text-[10px] font-bold text-violet-600">
              {event.organizer_name.charAt(0).toUpperCase()}
            </div>
          )}
          <span className="text-xs text-neutral-500">{event.organizer_name}</span>
        </div>

        {/* Price + CTA */}
        <div className="mt-auto flex items-center justify-between pt-3">
          <span className="text-sm font-bold text-neutral-800">
            {formatPrice(event.min_price)}
          </span>

          <span
            className={`rounded-xl px-3 py-1.5 text-xs font-semibold transition-colors ${
              soldOut
                ? "bg-neutral-100 text-neutral-400"
                : "bg-violet-600 text-white group-hover:bg-violet-700"
            }`}
          >
            {soldOut ? "Waitlist" : "Get Tickets"}
          </span>
        </div>
      </div>
    </Link>
  );
}

// ── Main component ────────────────────────────────────────────────────────────

interface RecommendedEventsProps {
  /** Pass false when user is not authenticated to skip the API call */
  isLoggedIn?: boolean;
  limit?: number;
  className?: string;
}

export default function RecommendedEvents({
  isLoggedIn = true,
  limit = 12,
  className = "",
}: RecommendedEventsProps) {
  const { events, personalised, basedOnCategories, isLoading, isError } =
    useRecommendedEvents(limit, isLoggedIn);

  // Nothing to show if logged out
  if (!isLoggedIn) return null;

  // Error state — silent fail, don't break the page
  if (isError) return null;

  // Resolved: no recommendations at all
  if (!isLoading && events.length === 0) return null;

  return (
    <section className={`w-full ${className}`}>
      {/* Section header */}
      <div className="mb-5 flex flex-col gap-1 sm:flex-row sm:items-end sm:justify-between">
        <div>
          <h2 className="text-xl font-extrabold tracking-tight text-neutral-900">
            {personalised ? "Recommended for You" : "Upcoming Events"}
          </h2>

          {personalised && basedOnCategories.length > 0 && (
            <p className="mt-1 text-sm text-neutral-500">
              Based on your interest in{" "}
              {basedOnCategories.map((cat, i) => (
                <span key={cat}>
                  <span className="font-medium text-violet-600">{cat}</span>
                  {i < basedOnCategories.length - 1 ? ", " : ""}
                </span>
              ))}
            </p>
          )}
        </div>

        <Link
          href="/events"
          className="mt-2 whitespace-nowrap text-sm font-semibold text-violet-600 hover:text-violet-700 sm:mt-0"
        >
          Browse all →
        </Link>
      </div>

      {/* Grid */}
      <div className="grid grid-cols-1 gap-4 sm:grid-cols-2 lg:grid-cols-3 xl:grid-cols-4">
        {isLoading
          ? Array.from({ length: 4 }).map((_, i) => <SkeletonCard key={i} />)
          : events.map((event) => <EventCard key={event.id} event={event} />)}
      </div>
    </section>
  );
}

// ── Inline icon components (no extra dependencies) ────────────────────────────

function CalendarIcon() {
  return (
    <svg
      width="12" height="12" viewBox="0 0 24 24"
      fill="none" stroke="currentColor" strokeWidth="2"
      strokeLinecap="round" strokeLinejoin="round"
      className="flex-shrink-0"
    >
      <rect x="3" y="4" width="18" height="18" rx="2" ry="2" />
      <line x1="16" y1="2" x2="16" y2="6" />
      <line x1="8" y1="2" x2="8" y2="6" />
      <line x1="3" y1="10" x2="21" y2="10" />
    </svg>
  );
}

function PinIcon() {
  return (
    <svg
      width="11" height="11" viewBox="0 0 24 24"
      fill="none" stroke="currentColor" strokeWidth="2"
      strokeLinecap="round" strokeLinejoin="round"
      className="flex-shrink-0"
    >
      <path d="M21 10c0 7-9 13-9 13S3 17 3 10a9 9 0 0 1 18 0z" />
      <circle cx="12" cy="10" r="3" />
    </svg>
  );
}