"use client";

import { ErrorBanner } from "@/components/ui/error-banner";
import { AddToCalendar } from "@/components/events/add-to-calendar";
import { ReportEventLink } from "@/components/events/ReportEventLink";
import { CalendarEventInput } from "@/utils/calendar";

/**
 * Client wrapper for event detail page that manages error state,
 * provides calendar actions, and a retry mechanism for failed data fetches.
 */
interface EventDetailClientProps {
  children: React.ReactNode;
  /** A key that when changed (by retry) triggers re-render */
  onRetry?: () => void;
  error?: Error | null;
  event?: CalendarEventInput;
}

export function EventDetailClient({
  children,
  onRetry,
  error,
  event,
}: EventDetailClientProps) {
  if (error) {
    return (
      <div className="flex-1 w-full max-w-[1221px] mx-auto px-6 py-6 sm:py-12">
        <ErrorBanner
          message="Failed to load event data"
          description={error.message || "An unexpected error occurred. Please try again."}
          onRetry={onRetry || (() => {})}
        />
      </div>
    );
  }

  return (
    <>
      {event && (
        <div className="mb-4 flex justify-end">
          <AddToCalendar event={event} />
        </div>
      )}
      {children}
      {event && (
        <div className="mt-8 flex justify-start">
          <ReportEventLink eventId={event.id} />
        </div>
      )}
    </>
  );
}

export { AddToCalendar };