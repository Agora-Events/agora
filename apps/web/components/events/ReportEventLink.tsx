"use client";

import { useTranslations } from "next-intl";
import { SUPPORT_EMAIL } from "@/lib/constants";

interface ReportEventLinkProps {
  eventId: string | number;
  className?: string;
}

export function ReportEventLink({ eventId, className = "" }: ReportEventLinkProps) {
  const t = useTranslations("eventDetail");

  const eventUrl = typeof window !== "undefined"
    ? window.location.href
    : `https://agora.events/events/${eventId}`;

  const subject = encodeURIComponent(`Report event ${eventId}`);
  const body = encodeURIComponent(eventUrl);
  const mailtoHref = `mailto:${SUPPORT_EMAIL}?subject=${subject}&body=${body}`;

  return (
    <a
      href={mailtoHref}
      className={`text-xs text-gray-500 hover:text-gray-700 underline underline-offset-2 transition-colors focus-visible:outline-2 focus-visible:outline-offset-2 focus-visible:outline-accent ${className}`}
      aria-label={`${t("reportEvent")} (${eventId})`}
    >
      {t("reportEvent")}
    </a>
  );
}
