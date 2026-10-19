"use client";

import { Suspense, useState, useMemo } from "react";
import Link from "next/link";
import Image from "next/image";
import { useRouter, useSearchParams } from "next/navigation";
import { Navbar } from "@/components/layout/navbar";
import { Footer } from "@/components/layout/footer";
import { TicketCard, PastEventsSection } from "@/components/wallet";
import { SellTicketModal } from "@/components/wallet/SellTicketModal";
import { useWalletTickets } from "@/hooks/useWalletTickets";
import { useAuth } from "@/hooks/useAuth";
import type { WalletTicket } from "@/hooks/useWalletTickets";
import { TestnetFaucetCard } from "@/components/wallet/testnet-faucet-card";

type SortOption = "soonest" | "latest";

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
  onSellTicket,
}: {
  title: string;
  subtitle: string;
  tickets: WalletTicket[];
  isLoading: boolean;
  emptyHeading: string;
  emptySubtext: string;
  onSellTicket?: (ticket: WalletTicket) => void;
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
                  onSell={onSellTicket ? () => onSellTicket(ticket) : undefined}
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
  const { upcoming, past, poaps, isLoading: ticketsLoading, mutate } = useWalletTickets();
  const [selectedSellTicket, setSelectedSellTicket] = useState<WalletTicket | null>(null);
  const router = useRouter();
  const searchParams = useSearchParams();

  const isLoading = authLoading || ticketsLoading;

  // Get sort preference from URL, default to "soonest"
  const sortParam = searchParams.get("sort");
  const sort: SortOption = sortParam === "latest" ? "latest" : "soonest";

  const handleSortChange = (value: SortOption) => {
    const params = new URLSearchParams(searchParams.toString());
    if (value === "latest") {
      params.set("sort", "latest");
    } else {
      params.delete("sort");
    }
    const query = params.toString();
    router.replace(query ? `/wallet?${query}` : "/wallet", { scroll: false });
  };

  // Sort upcoming tickets on the client side
  const sortedUpcoming = useMemo(() => {
    const tickets = [...upcoming];
    if (sort === "latest") {
      tickets.sort((a, b) => {
        const aTime = a.event_start_time ? new Date(a.event_start_time).getTime() : 0;
        const bTime = b.event_start_time ? new Date(b.event_start_time).getTime() : 0;
        return bTime - aTime; // Latest first
      });
    } else {
      // Soonest first (default - already sorted by API, but ensure consistent ordering)
      tickets.sort((a, b) => {
        const aTime = a.event_start_time ? new Date(a.event_start_time).getTime() : 0;
        const bTime = b.event_start_time ? new Date(b.event_start_time).getTime() : 0;
        return aTime - bTime;
      });
    }
    return tickets;
  }, [upcoming, sort]);

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

  const isTestnet = process.env.NEXT_PUBLIC_STELLAR_NETWORK === "TESTNET";

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

      {/* Testnet faucet card — Issue #1492.
          Only rendered when NEXT_PUBLIC_STELLAR_NETWORK === "TESTNET". */}
      {isTestnet && user?.walletAddress && (
        <TestnetFaucetCard publicKey={user.walletAddress} />
      )}
      {/* Sort control */}
      <div className="flex items-center justify-end gap-2">
        <label htmlFor="ticket-sort" className="text-xs font-medium text-muted-text whitespace-nowrap">
          Sort by
        </label>
        <select
          id="ticket-sort"
          value={sort}
          onChange={(e) => handleSortChange(e.target.value as SortOption)}
          className="rounded-xl border border-border-warm bg-white px-3 py-1.5 text-xs font-semibold text-ink-soft shadow-xs focus:outline-none focus:ring-2 focus:ring-accent transition-all cursor-pointer"
          aria-label="Sort upcoming tickets by event date"
        >
          <option value="soonest">Soonest first</option>
          <option value="latest">Latest first</option>
        </select>
      </div>

      {/* Upcoming tickets */}
      <TicketSection
        title="Upcoming Tickets"
        subtitle="Events you're attending soon"
        tickets={sortedUpcoming}
        isLoading={isLoading}
        emptyHeading="No upcoming tickets"
        emptySubtext="You don't have any upcoming events. Discover what's on near you."
        onSellTicket={(ticket) => setSelectedSellTicket(ticket)}
      />

      {/* Past events & POAP collectibles section (#1128) */}
      <PastEventsSection
        tickets={past}
        poaps={poaps}
        isLoading={isLoading}
      />

      {/* Resale Ticket Modal */}
      {selectedSellTicket && (
        <SellTicketModal
          isOpen={Boolean(selectedSellTicket)}
          onClose={() => setSelectedSellTicket(null)}
          ticket={selectedSellTicket}
          onSuccess={() => mutate()}
        />
      )}
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
