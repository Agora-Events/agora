"use client";

import React, { useState } from "react";
import Image from "next/image";
import { motion, AnimatePresence } from "framer-motion";
import { Button } from "@/components/ui/button";
import { EmptyState } from "@/components/ui/empty-state";
import { toast } from "sonner";
import { Ticket } from "@/components/ui/icons";

interface ResaleListing {
  id: string;
  sellerName: string;
  sellerAvatar: string;
  price: number;
  originalPrice: number;
  quantity: number;
  listedAt: string;
}

interface SecondaryMarketplaceTabProps {
  eventId: number;
}

// ─── Mock data ─────────────────────────────────────────────────────────────────

const mockResaleListings: ResaleListing[] = [
  {
    id: "rsl-001",
    sellerName: "Alex M.",
    sellerAvatar: "/images/pfp.png",
    price: 75,
    originalPrice: 49,
    quantity: 2,
    listedAt: "2 hours ago",
  },
  {
    id: "rsl-002",
    sellerName: "Jordan K.",
    sellerAvatar: "/images/pfp.png",
    price: 60,
    originalPrice: 49,
    quantity: 1,
    listedAt: "5 hours ago",
  },
  {
    id: "rsl-003",
    sellerName: "Sam T.",
    sellerAvatar: "/images/pfp.png",
    price: 55,
    originalPrice: 49,
    quantity: 1,
    listedAt: "1 day ago",
  },
];

// ─── Helpers ───────────────────────────────────────────────────────────────────

function formatPrice(amount: number): string {
  return `$${amount.toFixed(2)}`;
}

// ─── Component ─────────────────────────────────────────────────────────────────

export function SecondaryMarketplaceTab({
  eventId,
}: SecondaryMarketplaceTabProps) {
  const [listings] = useState<ResaleListing[]>(mockResaleListings);
  const [purchasingId, setPurchasingId] = useState<string | null>(null);

  const hasListings = listings.length > 0;

  const handlePurchase = async (listing: ResaleListing) => {
    setPurchasingId(listing.id);
    try {
      // TODO: Replace with real API call when #1145 is implemented
      const response = await fetch(`/api/v1/events/${eventId}/resale/${listing.id}/purchase`, {
        method: "POST",
        headers: { "Content-Type": "application/json" },
        body: JSON.stringify({ listingId: listing.id }),
      });

      if (!response.ok) {
        // Fallback: mock success for now
        await new Promise((resolve) => setTimeout(resolve, 800));
      }

      toast.success("Ticket purchased from marketplace!");
    } catch {
      toast.error("Failed to purchase ticket. Please try again.");
    } finally {
      setPurchasingId(null);
    }
  };

  if (!hasListings) {
    return (
      <EmptyState
        title="No resale tickets available"
        description="There are no tickets listed for resale on this event yet. Check back later or join the waitlist for primary tickets."
        icon={<Ticket size={40} className="text-black/40" />}
      />
    );
  }

  return (
    <div className="flex flex-col gap-4">
      <div className="flex items-center justify-between">
        <h3 className="text-lg font-bold text-black font-heading">
          Available Resale Tickets
        </h3>
        <span className="text-sm text-black/50 font-medium">
          {listings.length} listing{listings.length !== 1 ? "s" : ""}
        </span>
      </div>

      <AnimatePresence mode="popLayout">
        {listings.map((listing) => (
          <motion.div
            key={listing.id}
            layout
            initial={{ opacity: 0, y: 8 }}
            animate={{ opacity: 1, y: 0 }}
            exit={{ opacity: 0, y: -8 }}
            className="bg-white/50 rounded-2xl p-5 border border-black/5 flex flex-col sm:flex-row sm:items-center gap-4 sm:gap-6"
          >
            {/* Seller info */}
            <div className="flex items-center gap-3 flex-1 min-w-0">
              <div className="relative w-10 h-10 rounded-full overflow-hidden bg-white border border-black/10 shrink-0">
                <Image
                  src={listing.sellerAvatar}
                  fill
                  alt={listing.sellerName}
                  className="object-cover"
                />
              </div>
              <div className="flex flex-col min-w-0">
                <span className="font-semibold text-black truncate">
                  {listing.sellerName}
                </span>
                <span className="text-xs text-black/50">
                  Listed {listing.listedAt}
                </span>
              </div>
            </div>

            {/* Quantity */}
            <div className="flex items-center gap-1.5 text-sm text-black/60 shrink-0">
              <Ticket size={14} />
              <span>
                {listing.quantity} ticket{listing.quantity !== 1 ? "s" : ""}
              </span>
            </div>

            {/* Price & purchase */}
            <div className="flex items-center gap-4 sm:gap-6 shrink-0">
              <div className="flex flex-col items-end">
                <span className="text-lg font-bold text-black font-heading">
                  {formatPrice(listing.price)}
                </span>
                <span className="text-xs text-black/40 line-through">
                  {formatPrice(listing.originalPrice)}
                </span>
              </div>
              <Button
                variant="primary"
                onClick={() => handlePurchase(listing)}
                disabled={purchasingId === listing.id}
                className="h-11 px-6 rounded-full text-sm whitespace-nowrap"
                aria-label={`Purchase ${listing.quantity} ticket(s) from ${listing.sellerName} for ${formatPrice(listing.price)}`}
              >
                {purchasingId === listing.id ? (
                  <div
                    className="w-4 h-4 border-2 border-black/30 border-t-black rounded-full animate-spin"
                    aria-label="Processing purchase"
                  />
                ) : (
                  "Buy"
                )}
              </Button>
            </div>
          </motion.div>
        ))}
      </AnimatePresence>
    </div>
  );
}