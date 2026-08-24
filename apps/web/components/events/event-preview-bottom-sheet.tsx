"use client";

import { useCallback, useRef } from "react";
import { motion, AnimatePresence, type PanInfo } from "framer-motion";
import Image from "next/image";
import Link from "next/link";
import { Overlay } from "@/components/ui/overlay";
import { useFocusTrap } from "@/hooks/useFocusTrap";
import { X, ArrowRight } from "@/components/ui/icons";

// ─── Types ────────────────────────────────────────────────────────────────────

export interface EventPreview {
  id: number | string;
  title: string;
  date: string;
  venue: string;
  imageUrl?: string;
}

interface EventPreviewBottomSheetProps {
  /** Whether the bottom sheet is visible */
  isOpen: boolean;
  /** Callback when the sheet should close */
  onClose: () => void;
  /** Event data to display */
  event: EventPreview | null;
}

// ─── Animation constants ──────────────────────────────────────────────────────

const SHEET_HEIGHT = 340; // px — approximate height of the sheet content
const SWIPE_THRESHOLD = 80; // px — minimum drag distance to trigger close

const sheetVariants = {
  hidden: { y: SHEET_HEIGHT },
  visible: { y: 0 },
};

// ─── Component ────────────────────────────────────────────────────────────────

export function EventPreviewBottomSheet({
  isOpen,
  onClose,
  event,
}: EventPreviewBottomSheetProps) {
  const focusTrapRef = useFocusTrap<HTMLDivElement>(isOpen, onClose);
  const sheetRef = useRef<HTMLDivElement>(null);

  const handleDragEnd = useCallback(
    (_: unknown, info: PanInfo) => {
      if (info.offset.y > SWIPE_THRESHOLD) {
        onClose();
      }
    },
    [onClose]
  );

  // Don't render anything when closed and no event data
  if (!event) return null;

  return (
    <AnimatePresence>
      {isOpen && (
        <div className="fixed inset-0 z-50 flex items-end justify-center lg:hidden">
          {/* Backdrop */}
          <Overlay isOpen={isOpen} onClose={onClose} zIndex={50} />

          {/* Bottom Sheet */}
          <motion.div
            ref={focusTrapRef as React.Ref<HTMLDivElement>}
            role="dialog"
            aria-modal="true"
            aria-label={`Event preview: ${event.title}`}
            className="relative z-50 w-full max-w-lg rounded-t-2xl bg-white shadow-[-4px_-4px_0_rgba(0,0,0,1)] border border-black"
            variants={sheetVariants}
            initial="hidden"
            animate="visible"
            exit="hidden"
            transition={{ type: "spring", damping: 25, stiffness: 300 }}
            drag="y"
            dragConstraints={{ top: 0, bottom: SHEET_HEIGHT }}
            dragElastic={{ top: 0, bottom: 0.5 }}
            onDragEnd={handleDragEnd}
            style={{ touchAction: "pan-y" }}
          >
            {/* Drag Handle */}
            <div className="flex justify-center pt-3 pb-1">
              <div className="w-10 h-1 rounded-full bg-gray-300" />
            </div>

            {/* Close Button */}
            <button
              onClick={onClose}
              className="absolute top-4 right-4 p-1 rounded-full hover:bg-gray-100 transition-colors z-10"
              aria-label="Close preview"
            >
              <X size={20} />
            </button>

            {/* Content */}
            <div className="px-5 pb-6 pt-2" ref={sheetRef}>
              {/* Event Image */}
              {event.imageUrl && (
                <div className="relative w-full h-40 rounded-lg overflow-hidden mb-4">
                  <Image
                    src={event.imageUrl}
                    alt={event.title}
                    fill
                    className="object-cover"
                    sizes="(max-width: 512px) 100vw, 512px"
                  />
                </div>
              )}

              {/* Event Title */}
              <h3 className="font-semibold text-lg leading-tight mb-1">
                {event.title}
              </h3>

              {/* Event Date */}
              <div className="flex items-center gap-2 text-sm text-gray-600 mb-1">
                <svg
                  width="16"
                  height="16"
                  viewBox="0 0 24 24"
                  fill="none"
                  stroke="currentColor"
                  strokeWidth="2"
                  strokeLinecap="round"
                  strokeLinejoin="round"
                  aria-hidden="true"
                >
                  <rect x="3" y="4" width="18" height="18" rx="2" ry="2" />
                  <line x1="16" y1="2" x2="16" y2="6" />
                  <line x1="8" y1="2" x2="8" y2="6" />
                  <line x1="3" y1="10" x2="21" y2="10" />
                </svg>
                <span>{event.date}</span>
              </div>

              {/* Event Venue */}
              <div className="flex items-center gap-2 text-sm text-gray-600 mb-5">
                <svg
                  width="16"
                  height="16"
                  viewBox="0 0 24 24"
                  fill="none"
                  stroke="currentColor"
                  strokeWidth="2"
                  strokeLinecap="round"
                  strokeLinejoin="round"
                  aria-hidden="true"
                >
                  <path d="M21 10c0 7-9 13-9 13s-9-6-9-13a9 9 0 0 1 18 0z" />
                  <circle cx="12" cy="10" r="3" />
                </svg>
                <span>{event.venue}</span>
              </div>

              {/* CTA Button */}
              <Link
                href={`/events/${event.id}`}
                className="flex items-center justify-center gap-2 w-full py-3 px-6 rounded-full border border-black font-semibold bg-white shadow-[-4px_4px_0_rgba(0,0,0,1)] hover:-translate-x-[2px] hover:translate-y-[2px] hover:shadow-[-2px_2px_0_rgba(0,0,0,1)] active:-translate-x-[4px] active:translate-y-[4px] active:shadow-none transition-all"
                onClick={onClose}
              >
                View Event Details
                <ArrowRight size={18} />
              </Link>
            </div>
          </motion.div>
        </div>
      )}
    </AnimatePresence>
  );
}