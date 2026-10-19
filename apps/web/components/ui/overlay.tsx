"use client";

import { useCallback, useEffect, useRef } from "react";

/**
 * Overlay component that covers the viewport with a semi-transparent backdrop.
 * Renders behind modals/drawers and closes them on click.
 */
interface OverlayProps {
  /** Whether the overlay is visible */
  isOpen: boolean;
  /** Callback fired when the overlay is clicked or Escape is pressed */
  onClose: () => void;
  /** Optional z-index override (default: 40) */
  zIndex?: number;
}

export function Overlay({ isOpen, onClose, zIndex = 40 }: OverlayProps) {
  const overlayRef = useRef<HTMLDivElement>(null);

  const handleKeyDown = useCallback(
    (e: KeyboardEvent) => {
      if (e.key === "Escape" && isOpen) {
        onClose();
      }
    },
    [isOpen, onClose]
  );

  useEffect(() => {
    if (!isOpen) return;
    document.addEventListener("keydown", handleKeyDown);
    return () => document.removeEventListener("keydown", handleKeyDown);
  }, [isOpen, handleKeyDown]);

  if (!isOpen) return null;

  return (
    <div
      ref={overlayRef}
      onClick={onClose}
      role="presentation"
      aria-hidden="true"
      className="fixed inset-0 bg-black/40 z-40 lg:hidden"
      style={{ zIndex }}
    />
  );
}