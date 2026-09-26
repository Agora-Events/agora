"use client";

import { useEffect, useState } from "react";

type Listener = (message: string) => void;

const listeners = new Set<Listener>();

/**
 * Announces `message` to screen readers via the global live region mounted
 * once in `app/layout.tsx`. Safe to call from any client component.
 */
export function announce(message: string): void {
  listeners.forEach((listener) => listener(message));
}

/**
 * Visually hidden, polite live region. Screen readers read updates to its
 * text content without moving focus. Mount once near the root of the app;
 * call `announce()` from anywhere to have a message read aloud.
 */
export function LiveAnnouncer() {
  const [message, setMessage] = useState("");

  useEffect(() => {
    let clearTimer: ReturnType<typeof setTimeout>;

    const listener: Listener = (next) => {
      // Clear first so repeating the same message back-to-back still
      // triggers a fresh announcement (identical text is otherwise a no-op).
      setMessage("");
      clearTimer = setTimeout(() => setMessage(next), 50);
    };

    listeners.add(listener);
    return () => {
      clearTimeout(clearTimer);
      listeners.delete(listener);
    };
  }, []);

  return (
    <div role="status" aria-live="polite" aria-atomic="true" className="sr-only">
      {message}
    </div>
  );
}
