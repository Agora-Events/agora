"use client";

import { useEffect, useState } from "react";

/**
 * Debounces a rapidly changing value.
 *
 * Returns the previous value until `delay` milliseconds have elapsed without a
 * further change, which keeps search inputs from firing a request on every
 * keystroke.
 *
 * @typeParam T - Type of the debounced value.
 * @param value - The value to debounce.
 * @param delay - Quiet period in milliseconds before the value is published.
 * @returns The most recent value that stayed unchanged for `delay` ms.
 *
 * @example
 * const [search, setSearch] = useState("");
 * const debouncedSearch = useDebounce(search, 300);
 */
export function useDebounce<T>(value: T, delay: number): T {
  const [debouncedValue, setDebouncedValue] = useState<T>(value);

  useEffect(() => {
    const timeout = setTimeout(() => setDebouncedValue(value), delay);

    // Restarting the timer on every change is what collapses a burst of
    // keystrokes into a single update.
    return () => clearTimeout(timeout);
  }, [value, delay]);

  return debouncedValue;
}

export default useDebounce;
