"use client";

import { useState } from "react";
import Link from "next/link";
import Image from "next/image";
import { TicketCard } from "./TicketCard";
import { PoapCard, PoapCollectible } from "./PoapCard";
import type { WalletTicket } from "@/hooks/useWalletTickets";

export interface PastEventsSectionProps {
  tickets: WalletTicket[];
  poaps?: PoapCollectible[];
  isLoading: boolean;
}

type TabType = "all" | "events" | "poaps";

function SectionSkeleton() {
  return (
    <div className="grid grid-cols-1 md:grid-cols-2 gap-4 animate-pulse">
      <div className="h-32 rounded-xl bg-surface border border-border-warm" />
      <div className="h-32 rounded-xl bg-surface border border-border-warm" />
    </div>
  );
}

function EmptyPastState({
  tab,
}: {
  tab: TabType;
}) {
  const heading =
    tab === "poaps"
      ? "No POAP collectibles earned yet"
      : tab === "events"
      ? "No past events attended"
      : "No past events or POAP collectibles";

  const subtext =
    tab === "poaps"
      ? "Proof of Attendance (POAP) NFTs are automatically awarded when your ticket is scanned at an event."
      : "Events you attend will appear here alongside your earned POAP collectibles.";

  return (
    <div className="flex flex-col items-center justify-center py-14 px-6 text-center bg-surface/30 rounded-2xl border border-dashed border-border-warm">
      <div className="w-16 h-16 rounded-full bg-accent/15 flex items-center justify-center mb-4">
        <Image
          src="/icons/ticket-star.svg"
          width={32}
          height={32}
          alt=""
          className="opacity-70"
        />
      </div>
      <h3 className="font-bold text-ink-soft text-lg mb-1">{heading}</h3>
      <p className="text-sm text-muted-text max-w-sm mb-6 leading-relaxed">{subtext}</p>
      <Link
        href="/discover"
        className="inline-flex items-center gap-2 bg-accent text-ink-soft text-sm font-semibold px-6 py-2.5 rounded-full hover:bg-accent-hover transition-colors shadow-sm"
      >
        Discover Events
      </Link>
    </div>
  );
}

export function PastEventsSection({
  tickets,
  poaps = [],
  isLoading,
}: PastEventsSectionProps) {
  const [activeTab, setActiveTab] = useState<TabType>("all");

  // Synthesize POAP collectibles from checked-in tickets if explicit poaps list is empty
  const effectivePoaps: PoapCollectible[] =
    poaps.length > 0
      ? poaps
      : tickets
          .filter(
            (t) =>
              t.status.toLowerCase() === "used" ||
              t.status.toLowerCase() === "checked_in" ||
              t.status.toLowerCase() === "checkedin"
          )
          .map((t) => ({
            id: `poap-${t.id}`,
            payment_id: t.id,
            event_id: t.event_id ?? "unknown-event",
            event_title: t.event_title ?? "Attended Event",
            event_image_url: t.event_image_url,
            event_location: t.event_location,
            event_date: t.event_start_time,
            minted_at: t.created_at,
            token_id: t.id.slice(0, 12),
            is_soulbound: true,
          }));

  const showEvents = activeTab === "all" || activeTab === "events";
  const showPoaps = activeTab === "all" || activeTab === "poaps";

  const totalCount = tickets.length + effectivePoaps.length;

  return (
    <section className="bg-white rounded-2xl border border-border-warm shadow-sm overflow-hidden">
      {/* Header with Navigation Tabs */}
      <div className="px-6 pt-6 pb-4 border-b border-border-warm flex flex-col sm:flex-row sm:items-center sm:justify-between gap-4">
        <div>
          <div className="flex items-center gap-2">
            <h2 className="text-lg font-bold text-ink-soft">Past Events & Collectibles</h2>
            <span className="px-2 py-0.5 rounded-full bg-surface text-xs font-semibold text-muted-text">
              {totalCount}
            </span>
          </div>
          <p className="text-sm text-muted-text mt-0.5">
            Your event history and non-transferable POAP badges
          </p>
        </div>

        {/* Tab Filters */}
        <div className="inline-flex p-1 rounded-xl bg-surface border border-border-warm text-xs font-semibold self-start sm:self-auto">
          <button
            onClick={() => setActiveTab("all")}
            className={`px-3 py-1.5 rounded-lg transition-colors ${
              activeTab === "all"
                ? "bg-white text-ink-soft shadow-xs"
                : "text-muted-text hover:text-ink-soft"
            }`}
          >
            All ({totalCount})
          </button>
          <button
            onClick={() => setActiveTab("events")}
            className={`px-3 py-1.5 rounded-lg transition-colors ${
              activeTab === "events"
                ? "bg-white text-ink-soft shadow-xs"
                : "text-muted-text hover:text-ink-soft"
            }`}
          >
            Events ({tickets.length})
          </button>
          <button
            onClick={() => setActiveTab("poaps")}
            className={`px-3 py-1.5 rounded-lg transition-colors ${
              activeTab === "poaps"
                ? "bg-white text-ink-soft shadow-xs"
                : "text-muted-text hover:text-ink-soft"
            }`}
          >
            POAPs ({effectivePoaps.length})
          </button>
        </div>
      </div>

      {/* Body */}
      <div className="p-6 space-y-6">
        {isLoading ? (
          <SectionSkeleton />
        ) : (showEvents && tickets.length > 0) || (showPoaps && effectivePoaps.length > 0) ? (
          <div className="space-y-6">
            {/* POAP Collectibles Grid */}
            {showPoaps && effectivePoaps.length > 0 && (
              <div>
                <h3 className="text-xs font-bold uppercase tracking-wider text-muted-text mb-3 flex items-center gap-1.5">
                  <Image
                    src="/icons/ticket-star.svg"
                    width={14}
                    height={14}
                    alt=""
                  />
                  POAP NFTs ({effectivePoaps.length})
                </h3>
                <div className="grid grid-cols-1 md:grid-cols-2 gap-4">
                  {effectivePoaps.map((poap) => (
                    <PoapCard key={poap.id} poap={poap} />
                  ))}
                </div>
              </div>
            )}

            {/* Attended Event Tickets List */}
            {showEvents && tickets.length > 0 && (
              <div>
                {showPoaps && effectivePoaps.length > 0 && (
                  <h3 className="text-xs font-bold uppercase tracking-wider text-muted-text mb-3 flex items-center gap-1.5 pt-2 border-t border-border-warm">
                    <Image
                      src="/icons/calendar.svg"
                      width={14}
                      height={14}
                      alt=""
                    />
                    Event Tickets ({tickets.length})
                  </h3>
                )}
                <ul className="space-y-4" aria-label="Past Event Tickets">
                  {tickets.map((ticket) => (
                    <li key={ticket.id}>
                      <TicketCard
                        id={ticket.id}
                        status={ticket.status}
                        ticketType={ticket.ticket_tier_name ?? undefined}
                        price={
                          ticket.ticket_price !== null
                            ? ticket.ticket_price === "0.00" ||
                              ticket.ticket_price === "0"
                              ? "Free"
                              : ticket.ticket_price
                            : undefined
                        }
                        event={{
                          id: ticket.event_id ?? undefined,
                          title: ticket.event_title ?? "Unknown Event",
                          startTime: ticket.event_start_time ?? undefined,
                          location: ticket.event_location ?? undefined,
                          imageUrl: ticket.event_image_url ?? undefined,
                        }}
                      />
                    </li>
                  ))}
                </ul>
              </div>
            )}
          </div>
        ) : (
          <EmptyPastState tab={activeTab} />
        )}
      </div>
    </section>
  );
}

export default PastEventsSection;
