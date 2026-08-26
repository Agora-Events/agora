"use client";

import { Suspense } from "react";
import Link from "next/link";
import Image from "next/image";
import { Navbar } from "@/components/layout/navbar";
import { Footer } from "@/components/layout/footer";
import { TicketCard, PastEventsSection } from "@/components/wallet";
import { useWalletTickets } from "@/hooks/useWalletTickets";
import { useAuth } from "@/hooks/useAuth";
import type { WalletTicket } from "@/hooks/useWalletTickets";

// ---------------------------------------------------------------------------
// Skeletons
// ---------------------------------------------------------------------------

function TicketCardSkeleton() {
  return (
    <div className="flex items-stretch gap-4 rounded-xl border border-border-warm bg-white p-4 shadow-[-4px_4px_0_rgba(0,0,0,0.08)] animate-pulse">
      <div className="flex-shrink-0 w-20 h-20 sm:w-24 sm:h-24 rounded-lg bg-surface" />
      <div className="flex-1 min-w-0 flex flex-col justify-between gap-2">
        <div className="space-y-2">
          <div className="h-4 bg-surface rounded w-3/4" />
          <div className="h-3 bg-surface rounded w-1/2" />
          <div className="h-3 bg-surface rounded w-2/5" />
        </div>
        <div className="flex items-center gap-2">
          <div className="h-5 w-16 bg-surface rounded-full" />
        </div>
      </div>
    </div>
  );
}

function SectionSkeleton() {
  return (
    <div className="space-y-4">
      <TicketCardSkeleton />
      <TicketCardSkeleton />
    </div>
  );
}

// ---------------------------------------------------------------------------
// Empty state
// ---------------------------------------------------------------------------

function EmptyTickets({
  heading,
  subtext,
}: {
  heading: string;
  subtext: string;
}) {
  return (
    <div className="flex flex-col items-center justify-center py-14 px-6 text-center">
      <div className="w-16 h-16 rounded-full bg-surface flex items-center justify-center mb-4">
        <Image
          src="/icons/ticket.svg"
          width={28}
          height={28}
          alt="Ticket"
          className="opacity-60"
        />
      </div>
      <h3 className="font-semibold text-ink-soft mb-1">{heading}</h3>
      <p className="text-sm text-muted-text max-w-xs mb-5">{subtext}</p>
      <Link
        href="/discover"
        className="inline-flex items-center gap-2 bg-accent text-ink-soft text-sm font-semibold px-5 py-2.5 rounded-full hover:bg-accent-hover transition-colors"
      >
        Discover Events
      </Link>
    </div>
  );
}

// ---------------------------------------------------------------------------
// Section component for Upcoming Tickets
// ---------------------------------------------------------------------------

function TicketSection({
  title,
  subtitle,
  tickets,
  isLoading,
  emptyHeading,
  emptySubtext,
}: {
  title: string;
  subtitle: string;
  tickets: WalletTicket[];
  isLoading: boolean;
  emptyHeading: string;
  emptySubtext: string;
}) {
  return (
    <section className="bg-white rounded-2xl border border-border-warm shadow-sm overflow-hidden">
      {/* Header */}
      <div className="px-6 pt-6 pb-4 border-b border-border-warm">
        <h2 className="text-lg font-semibold text-ink-soft">{title}</h2>
        <p className="text-sm text-muted-text mt-0.5">{subtitle}</p>
      </div>

      {/* Body */}
      <div className="p-6">
        {isLoading ? (
          <SectionSkeleton />
        ) : tickets.length === 0 ? (
          <EmptyTickets heading={emptyHeading} subtext={emptySubtext} />
        ) : (
          <ul className="space-y-4" aria-label={title}>
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
        )}
      </div>
    </section>
  );
}

// ---------------------------------------------------------------------------
// Wallet content (requires auth context)
// ---------------------------------------------------------------------------

function WalletContent() {
  const { user, isAuthenticated, isLoading: authLoading } = useAuth();
  const { upcoming, past, poaps, isLoading: ticketsLoading } = useWalletTickets();

  const isLoading = authLoading || ticketsLoading;

  // Unauthenticated state
  if (!authLoading && !isAuthenticated) {
    return (
      <div className="flex-1 flex flex-col items-center justify-center py-24 px-4 text-center">
        <div className="w-20 h-20 rounded-full bg-surface flex items-center justify-center mb-6">
          <Image
            src="/icons/ticket.svg"
            width={36}
            height={36}
            alt="Wallet"
            className="opacity-60"
          />
        </div>
        <h2 className="text-xl font-bold text-ink-soft mb-2">
          Sign in to view your tickets
        </h2>
        <p className="text-muted-text text-sm max-w-sm mb-6">
          Connect your wallet to see your upcoming events, past tickets, and POAP collectibles.
        </p>
        <Link
          href="/auth"
          className="inline-flex items-center gap-2 bg-accent text-ink-soft text-sm font-semibold px-6 py-3 rounded-full hover:bg-accent-hover transition-colors"
        >
          Sign in
        </Link>
      </div>
    );
  }

  return (
    <div className="flex-1 w-full max-w-3xl mx-auto px-4 py-10 space-y-6">
      {/* Page heading */}
      <header>
        <h1 className="text-2xl font-bold text-ink-soft">My Wallet</h1>
        {user?.displayName && (
          <p className="text-sm text-muted-text mt-1">
            Welcome back,{" "}
            <span className="font-medium text-ink-soft">{user.displayName}</span>
          </p>
        )}
      </header>

      {/* Upcoming tickets */}
      <TicketSection
        title="Upcoming Tickets"
        subtitle="Events you're attending soon"
        tickets={upcoming}
        isLoading={isLoading}
        emptyHeading="No upcoming tickets"
        emptySubtext="You don't have any upcoming events. Discover what's on near you."
      />

      {/* Past events & POAP collectibles section (#1128) */}
      <PastEventsSection
        tickets={past}
        poaps={poaps}
        isLoading={isLoading}
      />
    </div>
  );
}

// ---------------------------------------------------------------------------
// Page
// ---------------------------------------------------------------------------

/**
 * /wallet
 *
 * Main attendee dashboard for viewing upcoming tickets, past events, and earned POAP collectibles.
 * Issues #1123, #1128
 */
export default function WalletPage() {
  return (
    <main className="flex flex-col min-h-screen bg-base">
      <Navbar />
      <Suspense>
        <WalletContent />
      </Suspense>
      <Footer />
    </main>
  );
}
