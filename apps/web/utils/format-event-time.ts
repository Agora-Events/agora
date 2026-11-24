/**
 * Formats an ISO timestamp with the viewer's local timezone.
 * Returns a formatted string like "Sat, 1 Jun 2026, 7:00 PM GMT+1"
 *
 * @param isoString - ISO 8601 timestamp
 * @param locale    - BCP 47 locale tag (e.g. "en-US", "es", "fr"). Defaults to "en-US".
 */

export function formatEventTime(isoString: string, locale = "en-US"): string {
  const date = new Date(isoString);

  if (Number.isNaN(date.getTime())) {
    return "";
  }

  const formatter = new Intl.DateTimeFormat(locale, {
    weekday: "short",
    year: "numeric",
    month: "short",
    day: "numeric",
    hour: "numeric",
    minute: "2-digit",
    timeZoneName: "short",
  });

  return formatter.format(date);
}

/**
 * Gets the viewer's IANA timezone string (e.g., "Europe/London", "Africa/Lagos")
 */
export function getTimezone(): string {
  return Intl.DateTimeFormat().resolvedOptions().timeZone;
}
