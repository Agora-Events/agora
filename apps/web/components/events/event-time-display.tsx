"use client";

import { useIsMounted } from "@/hooks/useIsMounted";
import { formatEventTime, getTimezone } from "@/utils/format-event-time";

interface EventTimeDisplayProps {
  startsAt: string;
}

export function EventTimeDisplay({ startsAt }: EventTimeDisplayProps) {
  const isMounted = useIsMounted();

  if (!isMounted) {
    // Return placeholder during SSR to avoid hydration mismatch
    return <span className="text-[18px] sm:text-[19px] font-medium text-black">{startsAt}</span>;
  }

  const formattedTime = formatEventTime(startsAt);
  const timezone = getTimezone();

  return (
    <div className="flex flex-col">
      <span className="text-[18px] sm:text-[19px] font-medium text-black" title={timezone}>
        {formattedTime}
      </span>
      <span className="text-[12px] sm:text-[13px] text-gray-500 mt-1">
        Times shown in your local timezone
      </span>
    </div>
  );
}
