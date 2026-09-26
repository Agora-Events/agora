export interface CalendarEventInput {
  id?: string | number;
  title: string;
  description?: string;
  location?: string;
  date?: string;
  startsAt?: string | Date;
  endsAt?: string | Date;
}

/**
 * Escapes characters in text strings per RFC 5545 spec for iCalendar.
 * Backslashes (\), semicolons (;), commas (,), and newlines (\n) must be escaped.
 */
export function escapeIcsText(text: string): string {
  if (!text) return "";
  return text
    .replace(/\\/g, "\\\\")
    .replace(/;/g, "\\;")
    .replace(/,/g, "\\,")
    .replace(/\r?\n/g, "\\n");
}

/**
 * Formats a Date object into UTC iCalendar timestamp format (YYYYMMDDTHHMMSSZ).
 */
export function formatToUtcString(date: Date): string {
  const pad = (n: number) => n.toString().padStart(2, "0");
  const year = date.getUTCFullYear();
  const month = pad(date.getUTCMonth() + 1);
  const day = pad(date.getUTCDate());
  const hours = pad(date.getUTCHours());
  const minutes = pad(date.getUTCMinutes());
  const seconds = pad(date.getUTCSeconds());

  return `${year}${month}${day}T${hours}${minutes}${seconds}Z`;
}

/**
 * Parses an event date string or Date object into a valid Date.
 * Falls back to current date if parsing fails.
 */
export function parseEventDate(dateInput?: string | Date): Date {
  if (!dateInput) return new Date();
  if (dateInput instanceof Date) return isNaN(dateInput.getTime()) ? new Date() : dateInput;

  const parsed = new Date(dateInput);
  if (!isNaN(parsed.getTime())) {
    return parsed;
  }
  return new Date();
}

/**
 * Builds a spec-compliant VCALENDAR / VEVENT .ics string from an event object.
 */
export function buildIcsFile(event: CalendarEventInput): string {
  const startDate = parseEventDate(event.startsAt || event.date);
  const endDate = event.endsAt
    ? parseEventDate(event.endsAt)
    : new Date(startDate.getTime() + 2 * 60 * 60 * 1000);

  const now = new Date();
  const dtStamp = formatToUtcString(now);
  const dtStart = formatToUtcString(startDate);
  const dtEnd = formatToUtcString(endDate);
  const uid = `${event.id ?? "agora-event"}-${startDate.getTime()}@agora.events`;

  const summary = escapeIcsText(event.title || "Agora Event");
  const description = escapeIcsText(event.description || "");
  const location = escapeIcsText(event.location || "");

  const lines = [
    "BEGIN:VCALENDAR",
    "VERSION:2.0",
    "PRODID:-//Agora//Event Calendar//EN",
    "CALSCALE:GREGORIAN",
    "METHOD:PUBLISH",
    "BEGIN:VEVENT",
    `UID:${uid}`,
    `DTSTAMP:${dtStamp}`,
    `DTSTART:${dtStart}`,
    `DTEND:${dtEnd}`,
    `SUMMARY:${summary}`,
    `DESCRIPTION:${description}`,
    `LOCATION:${location}`,
    "END:VEVENT",
    "END:VCALENDAR",
  ];

  return lines.join("\r\n") + "\r\n";
}

/**
 * Generates a prefilled Google Calendar creation URL from an event object.
 */
export function buildGoogleCalendarUrl(event: CalendarEventInput): string {
  const startDate = parseEventDate(event.startsAt || event.date);
  const endDate = event.endsAt
    ? parseEventDate(event.endsAt)
    : new Date(startDate.getTime() + 2 * 60 * 60 * 1000);

  const dtStart = formatToUtcString(startDate);
  const dtEnd = formatToUtcString(endDate);

  const params = new URLSearchParams({
    action: "TEMPLATE",
    text: event.title || "Agora Event",
    dates: `${dtStart}/${dtEnd}`,
  });

  if (event.description) {
    params.set("details", event.description);
  }
  if (event.location) {
    params.set("location", event.location);
  }

  return `https://calendar.google.com/calendar/render?${params.toString()}`;
}
