"use client";

import { useEffect } from "react";
import { ErrorBanner } from "@/components/ui/error-banner";

interface EventErrorProps {
  error: Error & { digest?: string };
  reset: () => void;
}

export default function EventError({ error, reset }: EventErrorProps) {
  useEffect(() => {
    // Log error for monitoring
    console.error("Unhandled error in /events/[id]:", error);
  }, [error]);

  return (
    <div className="w-full max-w-[1221px] mx-auto px-6 py-12 sm:py-20 flex flex-col items-center justify-center min-h-[400px]">
      <ErrorBanner
        message="Failed to load event details"
        description={
          error.message || "An unexpected error occurred. Please try again."
        }
        onRetry={reset}
      />
    </div>
  );
}
