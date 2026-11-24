"use client";

import Link from "next/link";
import { motion, useReducedMotion } from "framer-motion";
import { UpcomingEventsEmptyState } from "@/components/empty-state/upcoming-events-empty-state";

/**
 * Shape of a single ticket / upcoming event row fed into this component.
 * Extend as more fields become available from the API.
 */
export interface UpcomingEventTicket {
  id: string;
  eventId: string;
  title: string;
  /** ISO-8601 date-time string */
  startTime: string;
  location: string;
  imageUrl?: string;
}

interface UpcomingEventsProps {
  tickets: UpcomingEventTicket[];
}

/**
 * Dashboard section that lists the authenticated user's tickets for future
 * events.  When the ticket list is empty a friendly illustration and a
 * "Browse Events" call-to-action are shown instead of a blank area.
 */
export function UpcomingEvents({ tickets }: UpcomingEventsProps) {
  const shouldReduceMotion = useReducedMotion();

  if (tickets.length === 0) {
    return <UpcomingEventsEmptyState />;
  }

  return (
    <section aria-label="Upcoming events" className="w-full">
      <h2 className="mb-4 text-xl font-semibold text-[#131517]">
        Upcoming Events
      </h2>

      <ul className="flex flex-col gap-4">
        {tickets.map((ticket, index) => (
          <motion.li
            key={ticket.id}
            initial={shouldReduceMotion ? { opacity: 0 } : { opacity: 0, y: 8 }}
            animate={shouldReduceMotion ? { opacity: 1 } : { opacity: 1, y: 0 }}
            transition={{ duration: 0.3, delay: index * 0.05, ease: "easeOut" }}
          >
            <Link
              href={`/events/${ticket.eventId}`}
              className="flex items-center gap-4 rounded-2xl border border-black/10 bg-white p-4 shadow-[0_4px_16px_rgba(0,0,0,0.06)] transition-shadow hover:shadow-[0_8px_24px_rgba(0,0,0,0.10)] focus-visible:outline-none focus-visible:ring-2 focus-visible:ring-[#FFD21F]"
              aria-label={`View ${ticket.title}`}
            >
              {/* Thumbnail */}
              <div className="h-16 w-16 flex-shrink-0 overflow-hidden rounded-xl bg-gray-100">
                {ticket.imageUrl ? (
                  // eslint-disable-next-line @next/next/no-img-element
                  <img
                    src={ticket.imageUrl}
                    alt=""
                    aria-hidden
                    className="h-full w-full object-cover"
                    loading="lazy"
                  />
                ) : (
                  <div className="flex h-full w-full items-center justify-center">
                    <CalendarIcon className="h-7 w-7 text-gray-300" />
                  </div>
                )}
              </div>

              {/* Details */}
              <div className="min-w-0 flex-1">
                <p className="truncate font-semibold text-[#131517]">
                  {ticket.title}
                </p>
                <p className="mt-0.5 text-sm text-black/50">
                  {formatEventDate(ticket.startTime)}
                </p>
                {ticket.location && (
                  <p className="mt-0.5 flex items-center gap-1 text-xs text-black/40">
                    <LocationPinIcon className="h-3.5 w-3.5 flex-shrink-0" />
                    <span className="truncate">{ticket.location}</span>
                  </p>
                )}
              </div>

              {/* Arrow */}
              <ChevronRightIcon className="h-5 w-5 flex-shrink-0 text-black/30" />
            </Link>
          </motion.li>
        ))}
      </ul>
    </section>
  );
}

// ---------------------------------------------------------------------------
// Helpers
// ---------------------------------------------------------------------------

function formatEventDate(iso: string): string {
  try {
    return new Intl.DateTimeFormat("en-US", {
      weekday: "short",
      month: "short",
      day: "numeric",
      hour: "numeric",
      minute: "2-digit",
    }).format(new Date(iso));
  } catch {
    return iso;
  }
}

// ---------------------------------------------------------------------------
// Inline SVG icons (avoids extra icon-library dependency)
// ---------------------------------------------------------------------------

function CalendarIcon({ className }: { className?: string }) {
  return (
    <svg
      xmlns="http://www.w3.org/2000/svg"
      viewBox="0 0 24 24"
      fill="none"
      stroke="currentColor"
      strokeWidth={1.5}
      strokeLinecap="round"
      strokeLinejoin="round"
      className={className}
      aria-hidden
    >
      <rect x="3" y="4" width="18" height="18" rx="2" ry="2" />
      <line x1="16" y1="2" x2="16" y2="6" />
      <line x1="8" y1="2" x2="8" y2="6" />
      <line x1="3" y1="10" x2="21" y2="10" />
    </svg>
  );
}

function LocationPinIcon({ className }: { className?: string }) {
  return (
    <svg
      xmlns="http://www.w3.org/2000/svg"
      viewBox="0 0 24 24"
      fill="none"
      stroke="currentColor"
      strokeWidth={2}
      strokeLinecap="round"
      strokeLinejoin="round"
      className={className}
      aria-hidden
    >
      <path d="M21 10c0 7-9 13-9 13S3 17 3 10a9 9 0 0 1 18 0z" />
      <circle cx="12" cy="10" r="3" />
    </svg>
  );
}

function ChevronRightIcon({ className }: { className?: string }) {
  return (
    <svg
      xmlns="http://www.w3.org/2000/svg"
      viewBox="0 0 24 24"
      fill="none"
      stroke="currentColor"
      strokeWidth={2}
      strokeLinecap="round"
      strokeLinejoin="round"
      className={className}
      aria-hidden
    >
      <polyline points="9 18 15 12 9 6" />
    </svg>
  );
}
