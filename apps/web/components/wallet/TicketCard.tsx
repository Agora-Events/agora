"use client";

import Image from "next/image";
import { LazyImage } from "@/components/ui/LazyImage";

// ---------------------------------------------------------------------------
// Types
// ---------------------------------------------------------------------------

/** Status values that a ticket can carry. */
export type TicketStatus = "active" | "used" | "cancelled" | string;

export interface TicketCardProps {
  /** Unique ticket identifier. */
  id: string;
  /** Event the ticket belongs to. */
  event: {
    id?: string;
    title: string;
    /** ISO date-time string for the event start. */
    startTime?: string;
    location?: string;
    imageUrl?: string;
  };
  /** Name of the ticket tier, e.g. "General Admission" or "VIP". */
  ticketType?: string;
  /** Lifecycle status of the ticket. */
  status: TicketStatus;
  /** Optional price label, e.g. "25.00" or "Free". */
  price?: string;
  /** Called when the user clicks the Sell Ticket button. */
  onSell?: () => void;
  /** Called when the user taps/clicks the card. */
  onClick?: () => void;
  /** Additional Tailwind classes applied to the outer wrapper. */
  className?: string;
}

// ---------------------------------------------------------------------------
// Helpers
// ---------------------------------------------------------------------------

const STATUS_CONFIG: Record<
  string,
  { label: string; className: string }
> = {
  active: {
    label: "Active",
    className: "bg-accent text-ink-soft",
  },
  used: {
    label: "Used",
    className: "bg-surface text-muted-text",
  },
  cancelled: {
    label: "Cancelled",
    className: "bg-error/10 text-error",
  },
};

function getStatusConfig(status: TicketStatus) {
  return (
    STATUS_CONFIG[status] ?? {
      label: status.charAt(0).toUpperCase() + status.slice(1),
      className: "bg-surface text-muted-text",
    }
  );
}

function formatEventDate(iso?: string): string {
  if (!iso) return "";
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
// Component
// ---------------------------------------------------------------------------

/**
 * TicketCard
 *
 * Displays a single ticket's summary — event image, title, date, venue, ticket
 * type, and status badge. Designed to be reused across the Wallet dashboard and
 * anywhere else tickets need to be listed.
 *
 * @example
 * ```tsx
 * <TicketCard
 *   id="abc-123"
 *   event={{ title: "Stellar Summit", startTime: "2026-09-01T10:00:00Z", location: "San Francisco" }}
 *   ticketType="General Admission"
 *   status="active"
 * />
 * ```
 */
export function TicketCard({
  id,
  event,
  ticketType,
  status,
  price,
  onSell,
  onClick,
  className = "",
}: TicketCardProps) {
  const statusCfg = getStatusConfig(status);
  const formattedDate = formatEventDate(event.startTime);
  const isUsedOrCancelled = status === "used" || status === "cancelled";

  return (
    <article
      role={onClick ? "button" : undefined}
      tabIndex={onClick ? 0 : undefined}
      aria-label={`Ticket for ${event.title}`}
      onClick={onClick}
      onKeyDown={onClick ? (e) => e.key === "Enter" && onClick() : undefined}
      data-ticket-id={id}
      className={[
        "flex items-stretch gap-4 rounded-xl border border-border-warm bg-white p-4",
        "shadow-[-4px_4px_0_rgba(0,0,0,0.08)]",
        "transition-transform duration-150",
        onClick ? "cursor-pointer hover:scale-[1.01] focus-visible:outline" : "",
        isUsedOrCancelled ? "opacity-60" : "",
        className,
      ]
        .filter(Boolean)
        .join(" ")}
    >
      {/* ── Event thumbnail ── */}
      <div className="flex-shrink-0 w-20 h-20 sm:w-24 sm:h-24 rounded-lg overflow-hidden bg-surface">
        {event.imageUrl ? (
          <LazyImage
            src={event.imageUrl}
            alt={event.title}
            width={96}
            height={96}
            className="object-cover w-full h-full"
          />
        ) : (
          <div className="w-full h-full flex items-center justify-center bg-surface-alt">
            <Image
              src="/icons/ticket.svg"
              width={32}
              height={32}
              alt=""
              aria-hidden="true"
            />
          </div>
        )}
      </div>

      {/* ── Content ── */}
      <div className="flex-1 min-w-0 flex flex-col justify-between">
        <div>
          {/* Title */}
          <p className="font-semibold text-sm sm:text-base text-ink-soft leading-snug line-clamp-2">
            {event.title}
          </p>

          {/* Date */}
          {formattedDate && (
            <p className="mt-1 flex items-center gap-1 text-xs text-muted-text">
              <Image
                src="/icons/calendar.svg"
                width={12}
                height={12}
                alt=""
                aria-hidden="true"
              />
              <span>{formattedDate}</span>
            </p>
          )}

          {/* Venue */}
          {event.location && (
            <p className="mt-0.5 flex items-center gap-1 text-xs text-muted-text">
              <Image
                src={
                  event.location.toLowerCase().includes("discord")
                    ? "/icons/discord.svg"
                    : "/icons/location.svg"
                }
                width={12}
                height={12}
                alt=""
                aria-hidden="true"
              />
              <span className="truncate">{event.location}</span>
            </p>
          )}

          {/* Ticket type */}
          {ticketType && (
            <p className="mt-1 text-xs text-muted-text">
              <span className="font-medium text-ink-soft">{ticketType}</span>
            </p>
          )}
        </div>

        {/* Footer: status badge + price + sell action */}
        <div className="mt-2 flex items-center justify-between flex-wrap gap-2">
          <div className="flex items-center gap-2">
            <span
              className={`inline-flex items-center px-2.5 py-0.5 rounded-full text-[11px] font-semibold ${statusCfg.className}`}
            >
              {statusCfg.label}
            </span>

            {price !== undefined && (
              <span className="text-xs font-semibold text-ink-soft">
                {price === "0" || price?.toLowerCase() === "free"
                  ? "Free"
                  : `$${price}`}
              </span>
            )}
          </div>

          {/* Sell Ticket action button */}
          {onSell && status === "active" && (
            <button
              type="button"
              onClick={(e) => {
                e.stopPropagation();
                onSell();
              }}
              className="inline-flex items-center gap-1 text-xs font-semibold px-3 py-1 rounded-lg bg-surface border border-border-warm text-ink-soft hover:bg-surface-alt transition-colors"
              aria-label={`Sell ticket for ${event.title}`}
            >
              <span>Sell Ticket</span>
            </button>
          )}
        </div>
      </div>
    </article>
  );
}


export default TicketCard;
