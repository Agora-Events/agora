import { useEffect, useState } from "react";

/**
 * Returns true only after the component has mounted on the client.
 * Use this to guard any rendering that depends on client-only APIs
 * (e.g. timezone offset) to prevent server/client hydration mismatches.
 */
export function useIsMounted(): boolean {
  const [isMounted, setIsMounted] = useState(false);

  useEffect(() => {
    const timer = window.setTimeout(() => {
      setIsMounted(true);
    }, 0);

    return () => window.clearTimeout(timer);
  }, []);

  return isMounted;
}
