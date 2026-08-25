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

      // Restore focus to the trigger element only if it is still in the DOM.
      // If it was removed while the modal was open (e.g. the item was deleted),
      // fall back to the nearest landmark we can reliably find: the page's main
      // heading, the <main> element, or <body> as a last resort — this keeps
      // focus at a sensible location rather than silently dropping it on <body>.
      const trigger = triggerRef.current as HTMLElement | null;
      if (trigger && document.body.contains(trigger)) {
        trigger.focus();
      } else {
        const fallback =
          (document.querySelector<HTMLElement>("h1")) ??
          (document.querySelector<HTMLElement>("main")) ??
          document.body;
        // <body> is not focusable by default; give it a transient tabIndex so
        // .focus() succeeds, then remove it so normal tab order is unaffected.
        if (fallback === document.body && document.body.tabIndex < 0) {
          document.body.tabIndex = -1;
          document.body.focus();
          document.body.removeAttribute("tabindex");
        } else {
          fallback.focus();
        }
      }
    };
  }, [active, onClose]);

  return containerRef;
}
