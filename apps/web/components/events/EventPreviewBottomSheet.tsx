"use client";

import React from "react";
import Image from "next/image";
import Link from "next/link";
import { motion, AnimatePresence, PanInfo } from "framer-motion";
import { Button } from "@/components/ui/button";

export interface PreviewEvent {
  id: string | number;
  title: string;
  date: string;
  location: string;
  price?: string;
  imageUrl?: string;
  category?: string;
}

interface EventPreviewBottomSheetProps {
  isOpen: boolean;
  onClose: () => void;
  event: PreviewEvent | null;
}

export function EventPreviewBottomSheet({
  isOpen,
  onClose,
  event,
}: EventPreviewBottomSheetProps) {
  if (!isOpen || !event) return null;

  const handleDragEnd = (_: unknown, info: PanInfo) => {
    if (info.offset.y > 100 || info.velocity.y > 500) {
      onClose();
    }
  };

  return (
    <AnimatePresence>
      <div className="fixed inset-0 z-50 flex items-end justify-center pointer-events-auto">
        {/* Backdrop Overlay */}
        <motion.div
          initial={{ opacity: 0 }}
          animate={{ opacity: 1 }}
          exit={{ opacity: 0 }}
          onClick={onClose}
          className="fixed inset-0 bg-black/40 backdrop-blur-xs transition-opacity"
          aria-hidden="true"
        />

        {/* Sliding Bottom Sheet */}
        <motion.div
          role="dialog"
          aria-modal="true"
          aria-labelledby="bottom-sheet-title"
          initial={{ y: "100%" }}
          animate={{ y: 0 }}
          exit={{ y: "100%" }}
          transition={{ type: "spring", stiffness: 300, damping: 30 }}
          drag="y"
          dragConstraints={{ top: 0, bottom: 0 }}
          dragElastic={{ top: 0, bottom: 0.8 }}
          onDragEnd={handleDragEnd}
          className="relative w-full max-w-lg rounded-t-3xl bg-white p-6 shadow-2xl border-t border-border-warm z-10 space-y-4 touch-pan-y"
        >
          {/* Drag handle bar */}
          <div className="flex justify-center pb-2">
            <div className="h-1.5 w-12 rounded-full bg-border-warm/80" />
          </div>

          {/* Header Row with Close Button */}
          <div className="flex items-start justify-between gap-3">
            <div className="flex-1 min-w-0">
              {event.category && (
                <span className="inline-block text-[10px] font-bold uppercase tracking-wider text-accent bg-accent/10 px-2 py-0.5 rounded-full mb-1">
                  {event.category}
                </span>
              )}
              <h2 id="bottom-sheet-title" className="text-lg font-bold text-ink-soft line-clamp-2">
                {event.title}
              </h2>
            </div>
            <button
              onClick={onClose}
              className="rounded-full p-1.5 text-muted-text hover:bg-surface transition-colors"
              aria-label="Close bottom sheet"
            >
              ✕
            </button>
          </div>

          {/* Event Content Details Card */}
          <div className="flex items-center gap-4 rounded-2xl bg-surface p-3 border border-border-warm/60">
            {/* Thumbnail Image */}
            <div className="relative h-20 w-20 flex-shrink-0 overflow-hidden rounded-xl bg-surface-alt">
              {event.imageUrl ? (
                <Image
                  src={event.imageUrl}
                  alt={event.title}
                  fill
                  className="object-cover"
                />
              ) : (
                <div className="flex h-full w-full items-center justify-center text-muted-text">
                  🎫
                </div>
              )}
            </div>

            {/* Details */}
            <div className="flex-1 min-w-0 space-y-1 text-xs text-muted-text">
              <div className="flex items-center gap-1.5">
                <span>📅</span>
                <span className="font-semibold text-ink-soft truncate">{event.date}</span>
              </div>
              <div className="flex items-center gap-1.5">
                <span>📍</span>
                <span className="truncate">{event.location}</span>
              </div>
              {event.price && (
                <div className="flex items-center gap-1.5 pt-0.5">
                  <span className="font-bold text-accent text-sm">{event.price}</span>
                </div>
              )}
            </div>
          </div>

          {/* Action CTAs */}
          <div className="flex items-center gap-3 pt-2">
            <Button
              variant="outline"
              onClick={onClose}
              className="flex-1 rounded-xl text-xs font-semibold"
            >
              Dismiss
            </Button>
            <Link href={`/events/${event.id}`} className="flex-1">
              <Button className="w-full rounded-xl bg-accent text-ink-soft hover:bg-accent-hover font-semibold text-xs">
                View Event Details →
              </Button>
            </Link>
          </div>
        </motion.div>
      </div>
    </AnimatePresence>
  );
}

export default EventPreviewBottomSheet;
