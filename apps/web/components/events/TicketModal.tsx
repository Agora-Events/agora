"use client";

import React, { useState } from "react";
import { motion, AnimatePresence } from "framer-motion";
import { QRCodeSVG } from "qrcode.react";
import { toast } from "sonner";
import { X, Minus, Plus, Ticket, ArrowRight, CheckCircle2 } from "lucide-react";
import Image from "next/image";

interface TicketModalProps {
  isOpen: boolean;
  onClose: () => void;
  event: {
    id: number;
    title: string;
    price: string;
    location: string;
    date: string;
  };
  initialQuantity: number;
}

export function TicketModal({ isOpen, onClose, event, initialQuantity }: TicketModalProps) {
  const [quantity, setQuantity] = useState(initialQuantity);
  const [isPurchasing, setIsPurchasing] = useState(false);
  const [purchasedTicket, setPurchasedTicket] = useState<{ id: string } | null>(null);

  const isFree = event.price.toLowerCase() === "free";
  const unitPrice = isFree ? 0 : parseFloat(event.price.replace("$", ""));
  const totalPrice = unitPrice * quantity;

  const handleConfirmPurchase = async () => {
    setIsPurchasing(true);
    try {
      const response = await fetch("/api/payments/ticket", {
        method: "POST",
        headers: {
          "Content-Type": "application/json",
        },
        body: JSON.stringify({
          eventId: event.id.toString(),
          quantity: quantity,
          buyerWallet: "G...MOCK_WALLET_ADDRESS", // Placeholder
        }),
      });

      const data = await response.json();

      if (!response.ok) {
        throw new Error(data.error || "Failed to purchase ticket");
      }

      setPurchasedTicket({ id: data.ticketId });
      toast.success("Ticket purchased successfully!");
    } catch (error: any) {
      toast.error(error.message || "Something went wrong. Please try again.");
    } finally {
      setIsPurchasing(false);
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
          />

          {/* Modal Content */}
          <motion.div
            initial={{ scale: 0.9, opacity: 0, y: 20 }}
            animate={{ scale: 1, opacity: 1, y: 0 }}
            exit={{ scale: 0.9, opacity: 0, y: 20 }}
            className="relative w-full max-w-[500px] bg-[#FFFBE9] rounded-[32px] overflow-hidden border border-black/10 shadow-2xl"
          >
            {/* Close Button */}
            <button
              onClick={onClose}
              className="absolute top-6 right-6 w-10 h-10 rounded-full bg-white/50 hover:bg-white transition-colors flex items-center justify-center border border-black/5 z-10"
            >
              <X size={20} className="text-black" />
            </button>

            {!purchasedTicket ? (
              <div className="p-8 sm:p-10 flex flex-col gap-8">
                <div className="flex flex-col gap-2">
                  <div className="flex items-center gap-2 text-[#FDDA23] font-bold uppercase tracking-wider text-sm">
                    <Ticket size={16} />
                    <span>Confirm Ticket</span>
                  </div>
                  <h2 className="text-[28px] sm:text-[32px] font-bold text-black font-heading leading-tight">
                    {event.title}
                  </h2>
                  <p className="text-black/60 font-medium">
                    {event.date} • {event.location}
                  </p>
                </div>

                <div className="bg-white/50 rounded-2xl p-6 border border-black/5 flex flex-col gap-6">
                  <div className="flex justify-between items-center">
                    <span className="text-lg font-bold text-black">Quantity</span>
                    <div className="flex items-center gap-4">
                      <button
                        onClick={() => setQuantity(Math.max(1, quantity - 1))}
                        className="w-10 h-10 rounded-full bg-white border border-black/10 flex items-center justify-center hover:bg-[#FDDA23] transition-colors"
                      >
                        <Minus size={18} />
                      </button>
                      <span className="text-xl font-bold w-6 text-center">{quantity}</span>
                      <button
                        onClick={() => setQuantity(quantity + 1)}
                        className="w-10 h-10 rounded-full bg-white border border-black/10 flex items-center justify-center hover:bg-[#FDDA23] transition-colors"
                      >
                        <Plus size={18} />
                      </button>
                    </div>
                  </div>

                  <div className="h-[1px] bg-black/5 w-full" />

                  <div className="flex justify-between items-center">
                    <span className="text-lg font-bold text-black">Total Price</span>
                    <span className="text-2xl font-bold text-black font-heading">
                      {isFree ? "FREE" : `$${totalPrice.toFixed(2)}`}
                    </span>
                  </div>
                </div>

                <button
                  onClick={handleConfirmPurchase}
                  disabled={isPurchasing}
                  className="w-full bg-[#FDDA23] text-black font-bold text-xl h-16 rounded-full border-2 border-black shadow-[4px_4px_0px_0px_rgba(0,0,0,1)] active:translate-x-[2px] active:translate-y-[2px] active:shadow-none transition-all flex items-center justify-center gap-3 disabled:opacity-70 disabled:cursor-not-allowed group"
                >
                  {isPurchasing ? (
                    <div className="w-6 h-6 border-2 border-black/30 border-t-black rounded-full animate-spin" />
                  ) : (
                    <>
                      <span>Confirm Purchase</span>
                      <ArrowRight size={24} className="group-hover:translate-x-1 transition-transform" />
                    </>
                  )}
                </button>
              </div>
            ) : (
              <div className="p-8 sm:p-10 flex flex-col items-center text-center gap-8">
                <motion.div
                  initial={{ scale: 0.5, opacity: 0 }}
                  animate={{ scale: 1, opacity: 1 }}
                  className="w-20 h-20 rounded-full bg-green-100 flex items-center justify-center text-green-600"
                >
                  <CheckCircle2 size={48} />
                </motion.div>

                <div className="flex flex-col gap-2">
                  <h2 className="text-3xl font-bold text-black font-heading">Ticket Minted!</h2>
                  <p className="text-black/60 font-medium">Your ticket has been successfully registered on the Stellar network.</p>
                </div>

                <div className="bg-white p-6 rounded-3xl shadow-xl border border-black/5 flex flex-col items-center gap-4">
                  <QRCodeSVG
                    value={purchasedTicket.id}
                    size={200}
                    level="H"
                    includeMargin={true}
                  />
                  <div className="flex flex-col gap-1">
                    <span className="text-xs font-bold text-black/40 uppercase tracking-widest">Ticket ID</span>
                    <span className="font-mono text-sm font-bold text-black">{purchasedTicket.id}</span>
                  </div>
                </div>

                <button
                  onClick={onClose}
                  className="w-full bg-black text-white font-bold text-lg h-14 rounded-full hover:opacity-90 transition-opacity"
                >
                  Done
                </button>
              </div>
            )}

            {/* Background Watermark */}
            <div className="absolute -right-10 -bottom-10 opacity-[0.03] pointer-events-none -rotate-12 z-0">
              <Image
                src="/icons/stellar-logo.svg"
                width={300}
                height={300}
                alt="bg-watermark"
              />
            </div>
          </motion.div>
        </div>
      )}
    </AnimatePresence>
  );
}
