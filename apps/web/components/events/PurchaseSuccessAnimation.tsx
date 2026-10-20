"use client";

import { motion, useReducedMotion } from "framer-motion";

// ─── Confetti layout ──────────────────────────────────────────────────────────
// Fixed burst positions keep server and client renders identical.

const CONFETTI_COLORS = ["#FDDA23", "#22C55E", "#3B82F6", "#F43F5E", "#A855F7"];
const CONFETTI_PIECES = Array.from({ length: 14 }, (_, i) => {
  const angle = (i / 14) * Math.PI * 2;
  const distance = 56 + (i % 3) * 14;
  return {
    x: Math.cos(angle) * distance,
    y: Math.sin(angle) * distance,
    rotate: (i % 2 === 0 ? 1 : -1) * (90 + i * 15),
    color: CONFETTI_COLORS[i % CONFETTI_COLORS.length],
  };
});

interface PurchaseSuccessAnimationProps {
  /** True only once the purchase transaction has been confirmed. */
  active: boolean;
}

/**
 * Checkmark + confetti burst shown when a ticket purchase confirms (Issue #1543).
 * Users who prefer reduced motion get a static checkmark and no confetti.
 */
export function PurchaseSuccessAnimation({ active }: PurchaseSuccessAnimationProps) {
  const shouldReduceMotion = useReducedMotion();

  if (!active) return null;

  return (
    <div className="relative w-20 h-20" data-testid="purchase-success-animation">
      {!shouldReduceMotion && (
        <div
          className="absolute inset-0 pointer-events-none motion-reduce:hidden"
          aria-hidden="true"
          data-testid="purchase-success-confetti"
        >
          {CONFETTI_PIECES.map((piece, i) => (
            <motion.span
              key={i}
              className="absolute left-1/2 top-1/2 -ml-1 -mt-1 w-2 h-2 rounded-sm"
              style={{ backgroundColor: piece.color }}
              initial={{ x: 0, y: 0, opacity: 1, scale: 1, rotate: 0 }}
              animate={{ x: piece.x, y: piece.y, opacity: 0, scale: 0.6, rotate: piece.rotate }}
              transition={{ duration: 0.9, ease: "easeOut", delay: 0.15 }}
            />
          ))}
        </div>
      )}

      <motion.div
        initial={shouldReduceMotion ? false : { scale: 0.5, opacity: 0 }}
        animate={{ scale: 1, opacity: 1 }}
        transition={{ type: "spring", stiffness: 260, damping: 18 }}
        className="relative w-20 h-20 rounded-full bg-green-100 flex items-center justify-center text-green-600"
      >
        <svg
          xmlns="http://www.w3.org/2000/svg"
          width={44}
          height={44}
          viewBox="0 0 24 24"
          fill="none"
          stroke="currentColor"
          strokeWidth={2.5}
          strokeLinecap="round"
          strokeLinejoin="round"
          aria-hidden="true"
        >
          <motion.path
            d="M5 12.5l4.5 4.5L19 7.5"
            initial={shouldReduceMotion ? false : { pathLength: 0 }}
            animate={{ pathLength: 1 }}
            transition={{ duration: 0.45, ease: "easeOut", delay: 0.1 }}
          />
        </svg>
      </motion.div>
    </div>
  );
}
