import { useEffect, useRef, RefObject } from "react";

const FOCUSABLE =
  'a[href],area[href],input:not([disabled]),select:not([disabled]),textarea:not([disabled]),button:not([disabled]),iframe,object,embed,[contenteditable],[tabindex]:not([tabindex="-1"])';

export function useFocusTrap<T extends HTMLElement>(
  active: boolean,
  onClose?: () => void
): RefObject<T | null> {
  const containerRef = useRef<T | null>(null);
  const triggerRef = useRef<Element | null>(null);

  useEffect(() => {
    if (!active) {
      triggerRef.current = null;
      return;
    }

    // Store the element that triggered the modal so focus can be restored on close
    triggerRef.current = document.activeElement;

    // Use double-frame requestAnimationFrame to ensure the DOM is fully painted
    // and focusable before attempting to move focus (avoids 50ms hardcoded delay)
    let frameId: number;
    const focusFirst = () => {
      const el = containerRef.current?.querySelector<HTMLElement>(FOCUSABLE);
      el?.focus();
    };

    frameId = requestAnimationFrame(() => {
      frameId = requestAnimationFrame(focusFirst);
    });

    const handleKeyDown = (e: KeyboardEvent) => {
      // Escape key closes the active modal
      if (e.key === "Escape") {
        e.preventDefault();
        onClose?.();
        return;
      }

      if (e.key !== "Tab" || !containerRef.current) return;

      const focusable = Array.from(
        containerRef.current.querySelectorAll<HTMLElement>(FOCUSABLE)
      );
      if (focusable.length === 0) return;

      const first = focusable[0];
      const last = focusable[focusable.length - 1];

      if (e.shiftKey) {
        if (document.activeElement === first) {
          e.preventDefault();
          last.focus();
        }
      } else {
        if (document.activeElement === last) {
          e.preventDefault();
          first.focus();
        }
      }
    };

    document.addEventListener("keydown", handleKeyDown);
    return () => {
      cancelAnimationFrame(frameId);
      document.removeEventListener("keydown", handleKeyDown);
      // Restore focus to the original trigger element when the modal unmounts
      (triggerRef.current as HTMLElement | null)?.focus();
    };
  }, [active, onClose]);

  return containerRef;
}
