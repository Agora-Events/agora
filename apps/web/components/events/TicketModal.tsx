"use client";

import React, { useState, useEffect } from "react";
import { motion, AnimatePresence } from "framer-motion";
import { QRCodeSVG } from "qrcode.react";
import { toast } from "sonner";
import { X, Minus, Plus, Ticket, ArrowRight, CheckCircle2, Gift } from "@/components/ui/icons";
import Image from "next/image";
import { Button } from "@/components/ui/button";
import { useFocusTrap } from "@/hooks/useFocusTrap";
import { CheckoutAttribution, getCheckoutAttribution } from "@/utils/attribution";

// ─── Types ────────────────────────────────────────────────────────────────────

interface TicketModalProps {
  isOpen: boolean;
  onClose: () => void;
  event: {
    id: number;
    title: string;
    price: string;
    location: string;
    date: string;
    /** When 0 the modal switches to Waitlist mode instead of Purchase mode. */
    availableQuantity?: number;
  };
  initialQuantity: number;
}

/** The three distinct modal views. */
type ModalView = "purchase" | "purchased" | "waitlist_success";

// ─── Waitlist icon ────────────────────────────────────────────────────────────

function ClockIcon({ size = 24, className }: { size?: number; className?: string }) {
  return (
    <svg
      xmlns="http://www.w3.org/2000/svg"
      width={size}
      height={size}
      viewBox="0 0 24 24"
      fill="none"
      stroke="currentColor"
      strokeWidth={2}
      strokeLinecap="round"
      strokeLinejoin="round"
      className={className}
      aria-hidden="true"
    >
      <circle cx="12" cy="12" r="10" />
      <polyline points="12 6 12 12 16 14" />
    </svg>
  );
}

// ─── Component ────────────────────────────────────────────────────────────────

export function TicketModal({ isOpen, onClose, event, initialQuantity }: TicketModalProps) {
  const isSoldOut = event.availableQuantity === 0;

  const [view, setView] = useState<ModalView>(isSoldOut ? "purchase" : "purchase");
  const [quantity, setQuantity] = useState(initialQuantity);
  const [isPurchasing, setIsPurchasing] = useState(false);
  const [isJoiningWaitlist, setIsJoiningWaitlist] = useState(false);
  const [purchasedTicket, setPurchasedTicket] = useState<{ id: string } | null>(null);
  const [waitlistPosition, setWaitlistPosition] = useState<number | null>(null);
  const [recipientWallet, setRecipientWallet] = useState<string>("");
  const [isGiftMode, setIsGiftMode] = useState(false);

  const modalRef = useFocusTrap<HTMLDivElement>(isOpen);

  const isFree = event.price.toLowerCase() === "free";
  const unitPrice = isFree ? 0 : parseFloat(event.price.replace("$", ""));
  const totalPrice = unitPrice * quantity;

  // Reset internal state whenever the modal opens/closes or soldOut flips.
  useEffect(() => {
    if (isOpen) {
      setView("purchase");
      setPurchasedTicket(null);
      setWaitlistPosition(null);
      setIsGiftMode(false);
      setRecipientWallet("");
      setQuantity(initialQuantity);
    }
  }, [isOpen, initialQuantity]);

  // Keyboard & scroll lock
  useEffect(() => {
    if (!isOpen) return;
    const handleKeyDown = (e: KeyboardEvent) => {
      if (e.key === "Escape") onClose();
    };
    document.body.style.overflow = "hidden";
    window.addEventListener("keydown", handleKeyDown);
    return () => {
      document.body.style.overflow = "";
      window.removeEventListener("keydown", handleKeyDown);
    };
  }, [isOpen, onClose]);

  // ── Purchase handler ────────────────────────────────────────────────────────
  const handleConfirmPurchase = async () => {
    setIsPurchasing(true);
    try {
      const requestBody: {
        eventId: string;
        quantity: number;
        buyerWallet: string;
        recipientWallet?: string;
        attribution?: CheckoutAttribution;
      } = {
        eventId: event.id.toString(),
        quantity: quantity,
        buyerWallet: "GBUYERMOCKADDRESS1234567890STEL",
      };

      requestBody.attribution = getCheckoutAttribution();

      if (isGiftMode && recipientWallet.trim()) {
        requestBody.recipientWallet = recipientWallet.trim();
      }

      const response = await fetch("/api/payments/ticket", {
        method: "POST",
        headers: { "Content-Type": "application/json" },
        body: JSON.stringify(requestBody),
      });

      const data = await response.json();
      if (!response.ok) throw new Error(data.error || "Failed to purchase ticket");

      // Client-side XDR signature prompt via Freighter (Issue #1086)
      if (data.transactionXdr && data.requiresSignature) {
        try {
          const freighter = await import("@stellar/freighter-api");
          if (await freighter.isConnected()) {
            toast.info("Please sign the transaction in your Freighter wallet...");
            await freighter.signTransaction(data.transactionXdr, {
              networkPassphrase: process.env.NEXT_PUBLIC_STELLAR_NETWORK_PASSPHRASE || "Test SDF Network ; September 2015",
            });
          }
        } catch (signErr) {
          console.warn("Freighter wallet interaction:", signErr);
        }
      }

      setPurchasedTicket({ id: data.ticketId });
      setView("purchased");
      toast.success(
        isGiftMode && recipientWallet.trim()
          ? "Ticket purchased as a gift! The recipient will see it in their wallet."
          : "Ticket purchased successfully!",
      );
    } catch (error: unknown) {
      toast.error(error instanceof Error ? error.message : "Something went wrong. Please try again.");
    } finally {
      setIsPurchasing(false);
    }
  };

  // ── Waitlist handler ────────────────────────────────────────────────────────
  const handleJoinWaitlist = async () => {
    setIsJoiningWaitlist(true);
    try {
      const response = await fetch(`/api/v1/events/${event.id}/waitlist`, {
        method: "POST",
        headers: { "Content-Type": "application/json" },
      });

      const data = await response.json();
      if (!response.ok) throw new Error(data.error || "Failed to join waitlist");

      setWaitlistPosition(data.position);
      setView("waitlist_success");
      toast.success(`You're #${data.position} on the waitlist!`);
    } catch (error: unknown) {
      toast.error(error instanceof Error ? error.message : "Something went wrong. Please try again.");
    } finally {
      setIsJoiningWaitlist(false);
    }
  };

  return (
    <AnimatePresence>
      {isOpen && (
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
            aria-labelledby="ticket-modal-title"
            aria-describedby="ticket-modal-subtitle"
            initial={{ scale: 0.9, opacity: 0, y: 20 }}
            animate={{ scale: 1, opacity: 1, y: 0 }}
            exit={{ scale: 0.9, opacity: 0, y: 20 }}
            className="relative w-full max-w-[500px] bg-base rounded-[32px] overflow-hidden border border-black/10 shadow-2xl z-10"
          >
            {/* Close button */}
            <button
              type="button"
              onClick={onClose}
              className="absolute top-6 right-6 w-10 h-10 rounded-full bg-white/50 hover:bg-white transition-colors flex items-center justify-center border border-black/5 z-10"
              aria-label="Close modal"
            >
              <X size={20} className="text-black" />
            </button>

            {/* ── Purchase view ─────────────────────────────────────────── */}
            {view === "purchase" && !isSoldOut && (
              <div className="p-8 sm:p-10 flex flex-col gap-8">
                <div className="flex flex-col gap-2">
                  <div className="flex items-center gap-2 text-accent font-bold uppercase tracking-wider text-sm">
                    <Ticket size={16} aria-hidden="true" />
                    <span>Confirm Ticket</span>
                  </div>
                  <h2
                    id="ticket-modal-title"
                    className="text-[28px] sm:text-[32px] font-bold text-black font-heading leading-tight"
                  >
                    {event.title}
                  </h2>
                  <p id="ticket-modal-subtitle" className="text-black/60 font-medium">
                    {event.date} • {event.location}
                  </p>
                </div>

                <div className="bg-white/50 rounded-2xl p-6 border border-black/5 flex flex-col gap-6">
                  {/* Quantity */}
                  <div className="flex justify-between items-center">
                    <span className="text-lg font-bold text-black">Quantity</span>
                    <div className="flex items-center gap-4">
                      <button
                        type="button"
                        onClick={() => setQuantity(Math.max(1, quantity - 1))}
                        className="w-10 h-10 rounded-full bg-white border border-black/10 flex items-center justify-center hover:bg-accent transition-colors"
                        aria-label="Decrease quantity"
                      >
                        <Minus size={18} />
                      </button>
                      <span
                        className="text-xl font-bold w-6 text-center"
                        aria-live="polite"
                        aria-atomic="true"
                      >
                        {quantity}
                      </span>
                      <button
                        type="button"
                        onClick={() => setQuantity(quantity + 1)}
                        className="w-10 h-10 rounded-full bg-white border border-black/10 flex items-center justify-center hover:bg-accent transition-colors"
                        aria-label="Increase quantity"
                      >
                        <Plus size={18} />
                      </button>
                    </div>
                  </div>

                  <div className="h-[1px] bg-black/5 w-full" aria-hidden="true" />

                  {/* Gift toggle */}
                  <div className="flex justify-between items-center">
                    <div className="flex items-center gap-2">
                      <Gift size={20} className="text-black/70" aria-hidden="true" />
                      <span className="text-lg font-bold text-black">Gift to someone?</span>
                    </div>
                    <button
                      type="button"
                      role="switch"
                      aria-checked={isGiftMode}
                      onClick={() => {
                        setIsGiftMode(!isGiftMode);
                        if (isGiftMode) setRecipientWallet("");
                      }}
                      className={`w-14 h-8 rounded-full transition-colors relative ${
                        isGiftMode ? "bg-accent" : "bg-gray-300"
                      }`}
                      aria-label="Gift mode"
                    >
                      <div
                        className={`absolute top-1 w-6 h-6 bg-white rounded-full shadow-md transition-transform ${
                          isGiftMode ? "translate-x-7" : "translate-x-1"
                        }`}
                      />
                    </button>
                  </div>

                  {isGiftMode && (
                    <div className="flex flex-col gap-2">
                      <label htmlFor="recipientWallet" className="text-sm font-bold text-black/70">
                        Recipient Wallet Address
                      </label>
                      <input
                        id="recipientWallet"
                        type="text"
                        value={recipientWallet}
                        onChange={(e) => setRecipientWallet(e.target.value)}
                        placeholder="G... (Stellar address)"
                        className="w-full px-4 py-3 rounded-xl border border-black/10 bg-white focus:outline-none focus:ring-2 focus:ring-accent font-mono text-sm"
                      />
                      <p className="text-xs text-black/50">
                        The ticket will be sent to this wallet address
                      </p>
                    </div>
                  )}

                  <div className="h-[1px] bg-black/5 w-full" aria-hidden="true" />

                  {/* Total */}
                  <div className="flex justify-between items-center">
                    <span className="text-lg font-bold text-black">Total Price</span>
                    <span className="text-2xl font-bold text-black font-heading">
                      {isFree ? "FREE" : `$${totalPrice.toFixed(2)}`}
                    </span>
                  </div>
                </div>

                <Button
                  variant="primary"
                  onClick={handleConfirmPurchase}
                  disabled={isPurchasing}
                  className="w-full h-16 rounded-full text-xl disabled:opacity-70 disabled:cursor-not-allowed"
                >
                  {isPurchasing ? (
                    <div
                      className="w-6 h-6 border-2 border-black/30 border-t-black rounded-full animate-spin"
                      aria-label="Processing purchase"
                    />
                  ) : (
                    <>
                      <span>Confirm Purchase</span>
                      <ArrowRight
                        size={24}
                        className="group-hover:translate-x-1 transition-transform"
                        aria-hidden="true"
                      />
                    </>
                  )}
                </Button>
              </div>
            )}

            {/* ── Sold-out / Join Waitlist view ──────────────────────────── */}
            {view === "purchase" && isSoldOut && (
              <div className="p-8 sm:p-10 flex flex-col gap-8">
                <div className="flex flex-col gap-2">
                  {/* Sold-out chip */}
                  <div className="flex items-center gap-2 text-error font-bold uppercase tracking-wider text-sm">
                    <ClockIcon size={16} />
                    <span>Sold Out</span>
                  </div>
                  <h2
                    id="ticket-modal-title"
                    className="text-[28px] sm:text-[32px] font-bold text-black font-heading leading-tight"
                  >
                    {event.title}
                  </h2>
                  <p id="ticket-modal-subtitle" className="text-black/60 font-medium">
                    {event.date} • {event.location}
                  </p>
                </div>

                {/* Info box */}
                <div
                  className="bg-white/60 rounded-2xl p-6 border border-black/5 flex flex-col gap-4"
                  role="status"
                  aria-live="polite"
                >
                  <p className="text-base font-semibold text-black">
                    All tickets for this event have been claimed.
                  </p>
                  <p className="text-sm text-black/60 leading-relaxed">
                    Join the waitlist and we'll notify you automatically if a
                    spot becomes available. You won't be charged anything now.
                  </p>

                  {/* What happens section */}
                  <ul className="flex flex-col gap-2 pt-2 border-t border-black/5">
                    {[
                      "You'll receive an email if a ticket is released",
                      "Your queue position is reserved instantly",
                      "No payment required to join",
                    ].map((item) => (
                      <li key={item} className="flex items-start gap-2 text-sm text-black/70">
                        <span
                          className="w-5 h-5 rounded-full bg-accent flex items-center justify-center text-xs font-bold shrink-0 mt-0.5"
                          aria-hidden="true"
                        >
                          ✓
                        </span>
                        {item}
                      </li>
                    ))}
                  </ul>
                </div>

                <Button
                  variant="primary"
                  onClick={handleJoinWaitlist}
                  disabled={isJoiningWaitlist}
                  className="w-full h-16 rounded-full text-xl disabled:opacity-70 disabled:cursor-not-allowed"
                  aria-label="Join the waitlist for this event"
                >
                  {isJoiningWaitlist ? (
                    <div
                      className="w-6 h-6 border-2 border-black/30 border-t-black rounded-full animate-spin"
                      aria-label="Joining waitlist"
                    />
                  ) : (
                    <>
                      <ClockIcon size={22} aria-hidden="true" />
                      <span>Join Waitlist</span>
                    </>
                  )}
                </Button>
              </div>
            )}

            {/* ── Purchased success view ─────────────────────────────────── */}
            {view === "purchased" && purchasedTicket && (
              <div className="p-8 sm:p-10 flex flex-col items-center text-center gap-8">
                <motion.div
                  initial={{ scale: 0.5, opacity: 0 }}
                  animate={{ scale: 1, opacity: 1 }}
                  className="w-20 h-20 rounded-full bg-green-100 flex items-center justify-center text-green-600"
                >
                  <CheckCircle2 size={48} aria-hidden="true" />
                </motion.div>

                <div className="flex flex-col gap-2">
                  <h2 className="text-3xl font-bold text-black font-heading">Ticket Minted!</h2>
                  <p className="text-black/60 font-medium">
                    {isGiftMode && recipientWallet.trim()
                      ? `Your gift ticket has been sent to ${recipientWallet.slice(0, 8)}...${recipientWallet.slice(-4)} on the Stellar network.`
                      : "Your ticket has been successfully registered on the Stellar network."}
                  </p>
                </div>

                <div className="bg-white p-6 rounded-3xl shadow-xl border border-black/5 flex flex-col items-center gap-4">
                  <QRCodeSVG value={purchasedTicket.id} size={200} level="H" includeMargin />
                  <div className="flex flex-col gap-1">
                    <span className="text-xs font-bold text-black/40 uppercase tracking-widest">
                      Ticket ID
                    </span>
                    <span className="font-mono text-sm font-bold text-black">
                      {purchasedTicket.id}
                    </span>
                  </div>
                </div>

                <Button
                  variant="primary"
                  onClick={onClose}
                  className="w-full h-14 rounded-full text-lg"
                >
                  Done
                </Button>
              </div>
            )}

            {/* ── Waitlist success view ──────────────────────────────────── */}
            {view === "waitlist_success" && waitlistPosition !== null && (
              <div className="p-8 sm:p-10 flex flex-col items-center text-center gap-8">
                <motion.div
                  initial={{ scale: 0.5, opacity: 0 }}
                  animate={{ scale: 1, opacity: 1 }}
                  className="w-20 h-20 rounded-full bg-accent flex items-center justify-center"
                >
                  <ClockIcon size={40} className="text-black" />
                </motion.div>

                <div className="flex flex-col gap-2">
                  <h2 className="text-3xl font-bold text-black font-heading">
                    You&apos;re on the list!
                  </h2>
                  <p className="text-black/60 font-medium">
                    We&apos;ll notify you if a spot opens up for{" "}
                    <strong>{event.title}</strong>.
                  </p>
                </div>

                {/* Queue position badge */}
                <div
                  className="bg-white border-2 border-black rounded-3xl shadow-[-4px_4px_0_rgba(0,0,0,1)] px-10 py-6 flex flex-col items-center gap-1"
                  aria-label={`Your waitlist position is number ${waitlistPosition}`}
                >
                  <span className="text-xs font-bold text-black/40 uppercase tracking-widest">
                    Your Position
                  </span>
                  <span className="text-6xl font-extrabold text-ink-deep leading-none">
                    #{waitlistPosition}
                  </span>
                  <span className="text-sm text-black/50 font-medium">in the queue</span>
                </div>

                <p className="text-xs text-black/40 max-w-xs">
                  You can leave the waitlist at any time from your profile page.
                </p>

                <Button
                  variant="primary"
                  onClick={onClose}
                  className="w-full h-14 rounded-full text-lg"
                >
                  Done
                </Button>
              </div>
            )}

            {/* Background watermark */}
            <div className="absolute -right-10 -bottom-10 opacity-[0.03] pointer-events-none -rotate-12 z-0">
              <Image src="/icons/stellar-logo.svg" width={300} height={300} alt="" aria-hidden="true" />
            </div>
          </motion.div>
        </div>
      )}
    </AnimatePresence>
  );
}
