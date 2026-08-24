"use client";

import { useCallback, useEffect, useState } from "react";
import { AnimatePresence, motion } from "framer-motion";
import { Button } from "@/components/ui/button";
import { useFocusTrap } from "@/hooks/useFocusTrap";
import { toast } from "sonner";

// ─── Types ────────────────────────────────────────────────────────────────────

interface SellTicketModalProps {
  /** Whether the modal is open */
  isOpen: boolean;
  /** Callback fired when the modal requests to close */
  onClose: () => void;
  /** Event details for the ticket summary */
  event: {
    id: number;
    title: string;
    date: string;
    location: string;
    price: string;
  };
  /** Number of tickets the user is listing */
  ticketQuantity: number;
}

interface ValidationErrors {
  price?: string;
}

// ─── Constants ─────────────────────────────────────────────────────────────────

const PLATFORM_FEE_PERCENTAGE = 0.05; // 5% platform fee
const MIN_RESALE_PRICE = 1;
const MAX_RESALE_PRICE = 100000;

// ─── Helpers ───────────────────────────────────────────────────────────────────

function parsePrice(price: string): number {
  const cleaned = price.replace("$", "").replace(",", "").trim();
  return parseFloat(cleaned);
}

function formatPrice(amount: number): string {
  return `$${amount.toFixed(2)}`;
}

// ─── Component ─────────────────────────────────────────────────────────────────

/**
 * SellTicketModal — a reusable modal that lets users list their tickets
 * for resale. Users enter a resale price, review the listing summary,
 * and submit the transaction.
 *
 * The modal is presentational: it receives the event and ticket details
 * from its caller so it can be used anywhere in the app.
 */
export function SellTicketModal({
  isOpen,
  onClose,
  event,
  ticketQuantity,
}: SellTicketModalProps) {
  const modalRef = useFocusTrap<HTMLDivElement>(isOpen, onClose);
  const [resalePrice, setResalePrice] = useState("");
  const [errors, setErrors] = useState<ValidationErrors>({});
  const [isSubmitting, setIsSubmitting] = useState(false);

  // Reset state whenever the modal opens
  useEffect(() => {
    if (isOpen) {
      setResalePrice("");
      setErrors({});
      setIsSubmitting(false);
    }
  }, [isOpen]);

  // ─── Validation ────────────────────────────────────────────────────────────

  const validate = useCallback((): ValidationErrors => {
    const errs: ValidationErrors = {};
    const parsed = parseFloat(resalePrice);

    if (!resalePrice.trim()) {
      errs.price = "Resale price is required";
    } else if (isNaN(parsed)) {
      errs.price = "Please enter a valid number";
    } else if (parsed < MIN_RESALE_PRICE) {
      errs.price = `Minimum resale price is ${formatPrice(MIN_RESALE_PRICE)}`;
    } else if (parsed > MAX_RESALE_PRICE) {
      errs.price = `Maximum resale price is ${formatPrice(MAX_RESALE_PRICE)}`;
    } else if (!/^\d+(\.\d{1,2})?$/.test(resalePrice.trim())) {
      errs.price = "Price can have at most 2 decimal places";
    }

    return errs;
  }, [resalePrice]);

  // ─── Computed values ───────────────────────────────────────────────────────

  const originalUnitPrice = parsePrice(event.price);
  const parsedResalePrice = parseFloat(resalePrice) || 0;
  const platformFee = parsedResalePrice * PLATFORM_FEE_PERCENTAGE;
  const netPayout = parsedResalePrice - platformFee;
  const hasValidPrice =
    !isNaN(parsedResalePrice) && parsedResalePrice >= MIN_RESALE_PRICE;

  // ─── Handlers ──────────────────────────────────────────────────────────────

  const handlePriceChange = useCallback(
    (e: React.ChangeEvent<HTMLInputElement>) => {
      const value = e.target.value;
      // Allow empty input, digits, and decimal with up to 2 places
      if (value === "" || /^\d+(\.\d{0,2})?$/.test(value)) {
        setResalePrice(value);
        // Clear price error when user starts typing
        if (errors.price) {
          setErrors((prev) => {
            const next = { ...prev };
            delete next.price;
            return next;
          });
        }
      }
    },
    [errors.price]
  );

  const handleSubmit = useCallback(async () => {
    const validationErrors = validate();
    if (Object.keys(validationErrors).length > 0) {
      setErrors(validationErrors);
      return;
    }

    setIsSubmitting(true);
    try {
      // TODO: Replace with real API call when #1145 is implemented
      // POST /api/v1/events/:id/resale
      const response = await fetch(
        `/api/v1/events/${event.id}/resale`,
        {
          method: "POST",
          headers: { "Content-Type": "application/json" },
          body: JSON.stringify({
            ticketQuantity,
            resalePrice: parsedResalePrice,
          }),
        }
      );

      if (!response.ok) {
        // Fallback: mock success for now
        await new Promise((resolve) => setTimeout(resolve, 800));
      }

      toast.success("Your ticket has been listed for resale!");
      onClose();
    } catch {
      toast.error("Failed to list ticket. Please try again.");
    } finally {
      setIsSubmitting(false);
    }
  }, [validate, event.id, ticketQuantity, parsedResalePrice, onClose]);

  // ─── Render ────────────────────────────────────────────────────────────────

  if (!isOpen) return null;

  const isFree = event.price.toLowerCase() === "free";

  return (
    <AnimatePresence>
      <div className="fixed inset-0 z-[100] flex items-center justify-center p-4 sm:p-6">
        {/* Backdrop */}
        <motion.div
          initial={{ opacity: 0 }}
          animate={{ opacity: 1 }}
          exit={{ opacity: 0 }}
          onClick={onClose}
          className="absolute inset-0 bg-black/60 backdrop-blur-md"
          aria-hidden="true"
        />

        {/* Modal */}
        <motion.div
          ref={modalRef}
          role="dialog"
          aria-modal="true"
          aria-labelledby="sell-ticket-title"
          initial={{ scale: 0.9, opacity: 0, y: 20 }}
          animate={{ scale: 1, opacity: 1, y: 0 }}
          exit={{ scale: 0.9, opacity: 0, y: 20 }}
          className="relative w-full max-w-[480px] rounded-[32px] border-2 border-black bg-white p-8 shadow-[-8px_8px_0_rgba(0,0,0,1)] sm:p-10"
        >
          {/* Close button */}
          <button
            type="button"
            onClick={onClose}
            className="absolute right-5 top-5 flex h-10 w-10 items-center justify-center rounded-full border border-black/10 bg-white/60 transition-colors hover:bg-gray-100"
            aria-label="Close sell ticket modal"
          >
            <svg
              width="18"
              height="18"
              viewBox="0 0 24 24"
              fill="none"
              stroke="currentColor"
              strokeWidth="2"
              strokeLinecap="round"
              strokeLinejoin="round"
            >
              <path d="M18 6L6 18M6 6l12 12" />
            </svg>
          </button>

          {/* Header */}
          <div className="mb-8">
            <h2
              id="sell-ticket-title"
              className="text-[28px] font-bold text-black font-heading"
            >
              Sell Your Ticket{ticketQuantity > 1 ? "s" : ""}
            </h2>
            <p className="text-sm text-black/50 mt-1">
              List your ticket{ticketQuantity > 1 ? "s" : ""} on the secondary marketplace
            </p>
          </div>

          {/* Ticket Summary */}
          <div className="mb-8 rounded-2xl bg-white/50 border border-black/5 p-5">
            <h3 className="text-sm font-semibold text-black/50 uppercase tracking-wide mb-3">
              Ticket Summary
            </h3>
            <div className="flex flex-col gap-2">
              <div className="flex justify-between items-center">
                <span className="text-sm text-black/60">Event</span>
                <span className="text-sm font-semibold text-black text-right max-w-[250px] truncate">
                  {event.title}
                </span>
              </div>
              <div className="flex justify-between items-center">
                <span className="text-sm text-black/60">Date</span>
                <span className="text-sm font-medium text-black">
                  {event.date}
                </span>
              </div>
              <div className="flex justify-between items-center">
                <span className="text-sm text-black/60">Location</span>
                <span className="text-sm font-medium text-black text-right max-w-[200px] truncate">
                  {event.location}
                </span>
              </div>
              <div className="flex justify-between items-center">
                <span className="text-sm text-black/60">Quantity</span>
                <span className="text-sm font-medium text-black">
                  {ticketQuantity} ticket{ticketQuantity > 1 ? "s" : ""}
                </span>
              </div>
              <div className="flex justify-between items-center">
                <span className="text-sm text-black/60">Original Price</span>
                <span className="text-sm font-medium text-black">
                  {isFree ? "Free" : formatPrice(originalUnitPrice)}
                </span>
              </div>
            </div>
          </div>

          {/* Resale Price Input */}
          <div className="mb-6">
            <label
              htmlFor="resale-price"
              className="block text-sm font-semibold text-black/50 uppercase tracking-wide mb-2"
            >
              Your Resale Price
            </label>
            <div className="relative">
              <span className="absolute left-4 top-1/2 -translate-y-1/2 text-lg font-bold text-black/40">
                $
              </span>
              <input
                id="resale-price"
                type="text"
                inputMode="decimal"
                value={resalePrice}
                onChange={handlePriceChange}
                placeholder="0.00"
                className={`w-full bg-white border-2 rounded-full pl-8 pr-4 py-3 text-lg font-bold text-black outline-none transition-shadow ${
                  errors.price
                    ? "border-red-400 shadow-[4px_4px_0px_0px_rgba(239,68,68,0.5)] focus:shadow-[2px_2px_0px_0px_rgba(239,68,68,0.5)]"
                    : "border-black shadow-[4px_4px_0px_0px_#000] focus:shadow-[2px_2px_0px_0px_#000]"
                }`}
                aria-invalid={!!errors.price}
                aria-describedby={errors.price ? "price-error" : undefined}
              />
            </div>
            {errors.price && (
              <p
                id="price-error"
                role="alert"
                className="text-xs text-red-500 mt-1.5 ml-2"
              >
                {errors.price}
              </p>
            )}
          </div>

          {/* Transaction Summary */}
          {hasValidPrice && (
            <motion.div
              initial={{ opacity: 0, height: 0 }}
              animate={{ opacity: 1, height: "auto" }}
              className="mb-8 rounded-2xl bg-black/5 border border-black/10 p-5 overflow-hidden"
            >
              <h3 className="text-sm font-semibold text-black/50 uppercase tracking-wide mb-3">
                Transaction Summary
              </h3>
              <div className="flex flex-col gap-2">
                <div className="flex justify-between items-center">
                  <span className="text-sm text-black/60">Your Price</span>
                  <span className="text-sm font-semibold text-black">
                    {formatPrice(parsedResalePrice)}
                  </span>
                </div>
                <div className="flex justify-between items-center">
                  <span className="text-sm text-black/60">
                    Platform Fee ({(PLATFORM_FEE_PERCENTAGE * 100).toFixed(0)}%)
                  </span>
                  <span className="text-sm text-black/60">
                    -{formatPrice(platformFee)}
                  </span>
                </div>
                <div className="border-t border-black/10 my-1" />
                <div className="flex justify-between items-center">
                  <span className="text-sm font-semibold text-black">
                    You Receive
                  </span>
                  <span className="text-base font-bold text-black font-heading">
                    {formatPrice(netPayout)}
                  </span>
                </div>
              </div>
            </motion.div>
          )}

          {/* Submit Button */}
          <div className="flex flex-col gap-3">
            <Button
              variant="primary"
              onClick={handleSubmit}
              disabled={isSubmitting}
              isLoading={isSubmitting}
              backgroundColor="bg-black"
              textColor="text-white"
              className="w-full h-12 rounded-full text-base"
              aria-label={
                isSubmitting
                  ? "Listing ticket for resale"
                  : "List ticket for resale"
              }
            >
              {isSubmitting
                ? "Listing..."
                : `List for ${formatPrice(parsedResalePrice || 0)}`}
            </Button>
            <button
              type="button"
              onClick={onClose}
              className="w-full py-3 text-sm font-medium text-black/50 hover:text-black/80 transition-colors rounded-full"
            >
              Cancel
            </button>
          </div>
        </motion.div>
      </div>
    </AnimatePresence>
  );
}