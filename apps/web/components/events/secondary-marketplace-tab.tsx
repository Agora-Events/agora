"use client";

import React, { useState } from "react";
import Image from "next/image";
import { motion, AnimatePresence } from "framer-motion";
import { Button } from "@/components/ui/button";
import { EmptyState } from "@/components/ui/empty-state";
import { toast } from "sonner";
import { Ticket } from "@/components/ui/icons";
import { purchaseResaleTicketWithFreighter } from "@/utils/stellar";

interface ResaleListing {
  id: string;
  sellerName: string;
  sellerAvatar: string;
  price: number;
  priceStroops: string;
  quantity: number;
  listedAt: string;
}

interface SecondaryMarketplaceTabProps {
  eventId: string | number;
}

// ─── Helpers ───────────────────────────────────────────────────────────────────

function formatPrice(amount: number): string {
  return `$${amount.toFixed(2)}`;
}

// ─── Component ─────────────────────────────────────────────────────────────────

export function SecondaryMarketplaceTab({
  eventId,
}: SecondaryMarketplaceTabProps) {
  const [listings, setListings] = useState<ResaleListing[]>([]);
  const [isLoading, setIsLoading] = useState<boolean>(true);
  const [loadError, setLoadError] = useState("");
  const [loadAttempt, setLoadAttempt] = useState(0);
  const [purchasingId, setPurchasingId] = useState<string | null>(null);

  React.useEffect(() => {
    let isMounted = true;
    async function fetchResaleListings() {
      try {
        setIsLoading(true);
        setLoadError("");
        const res = await fetch(`/api/v1/events/${eventId}/resale`);
        const json = await res.json().catch(() => ({}));
        if (!res.ok) {
          throw new Error(json.message || "Could not load resale listings.");
        }
        if (isMounted && Array.isArray(json.data)) {
          setListings(json.data);
        }
      } catch (error) {
        if (isMounted) {
          setLoadError(
            error instanceof Error ? error.message : "Could not load resale listings.",
          );
        }
      } finally {
        if (isMounted) setIsLoading(false);
      }
    }
    fetchResaleListings();
    return () => {
      isMounted = false;
    };
  }, [eventId, loadAttempt]);

  const hasListings = listings.length > 0;

  const handlePurchase = async (listing: ResaleListing) => {
    setPurchasingId(listing.id);
    try {
      const transactionHash = await purchaseResaleTicketWithFreighter(
        listing.id,
        listing.priceStroops,
      );
      setListings((current) => current.filter((item) => item.id !== listing.id));
      toast.success("Resale purchase confirmed on Stellar.", {
        description: `Transaction ${transactionHash.slice(0, 12)}…`,
      });
    } catch (error) {
      toast.error(
        error instanceof Error ? error.message : "Resale purchase failed.",
      );
    } finally {
      setPurchasingId(null);
    }
  };

  if (isLoading) {
    return (
      <p role="status" className="py-8 text-center text-sm text-black/50">
        Loading resale tickets...
      </p>
    );
  }

  if (loadError) {
    return (
      <div role="alert" className="py-8 text-center">
        <p className="text-sm text-red-700">{loadError}</p>
        <button
          type="button"
          onClick={() => setLoadAttempt((attempt) => attempt + 1)}
          className="mt-3 text-sm font-semibold text-black underline underline-offset-4"
        >
          Try again
        </button>
      </div>
    );
  }

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
              </div>
              <Button
                variant="primary"
                onClick={() => handlePurchase(listing)}
                disabled={purchasingId !== null}
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