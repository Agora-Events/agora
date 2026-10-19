"use client";

import { useEffect } from "react";
import { trackFunnelEvent } from "@/utils/funnel-analytics";

export function EventPageView({ eventId }: { eventId: string | number }) {
  useEffect(() => {
    void trackFunnelEvent("page_view", eventId);
  }, [eventId]);

  return null;
}
