/**
 * Returns a relative time string for a given date.
 * Uses Intl.RelativeTimeFormat to format the time difference.
 * Special-cases "today", "tomorrow", and "yesterday" via Intl (numeric: "auto")
 * so labels are automatically localised — no hard-coded English strings.
 * Picks the largest sensible unit (minutes → hours → days → weeks → months → years).
 *
 * @param date   - The date to format relative to now.
 * @param locale - BCP 47 locale tag (e.g. "en", "es", "fr"). Defaults to "en".
 */

export function getRelativeTime(date: Date, locale = "en"): string {
  if (Number.isNaN(date.getTime())) {
    return "";
  }

  const now = new Date();
  const diffMs = date.getTime() - now.getTime();
  const diffSeconds = diffMs / 1000;
  const diffMinutes = diffSeconds / 60;
  const diffHours = diffMinutes / 60;
  const diffDays = diffHours / 24;
  const diffWeeks = diffDays / 7;
  const diffMonths = diffDays / 30;
  const diffYears = diffMonths / 12;

  const rtf = new Intl.RelativeTimeFormat(locale, { numeric: "auto" });

  const calendarDate = date.toDateString();
  const isToday = calendarDate === now.toDateString();
  const tomorrow = new Date(now);
  tomorrow.setDate(tomorrow.getDate() + 1);
  const isTomorrow = calendarDate === tomorrow.toDateString();
  const yesterday = new Date(now);
  yesterday.setDate(yesterday.getDate() - 1);
  const isYesterday = calendarDate === yesterday.toDateString();

  // Use Intl.RelativeTimeFormat with numeric:"auto" to produce localised
  // "today" / "tomorrow" / "yesterday" equivalents (e.g. "hoy", "mañana",
  // "ayer" in Spanish) instead of hard-coded English literals.
  if (isToday) {
    return rtf.format(0, "day");
  }

  if (isTomorrow) {
    return rtf.format(1, "day");
  }

  if (isYesterday) {
    return rtf.format(-1, "day");
  }

  // Pick the largest sensible unit
  if (Math.abs(diffYears) >= 1) {
    return rtf.format(Math.round(diffYears), "year");
  }

  if (Math.abs(diffMonths) >= 1) {
    return rtf.format(Math.round(diffMonths), "month");
  }

  if (Math.abs(diffWeeks) >= 1) {
    return rtf.format(Math.round(diffWeeks), "week");
  }

  if (Math.abs(diffDays) >= 1) {
    return rtf.format(Math.round(diffDays), "day");
  }

  if (Math.abs(diffHours) >= 1) {
    return rtf.format(Math.round(diffHours), "hour");
  }

  if (Math.abs(diffMinutes) >= 1) {
    return rtf.format(Math.round(diffMinutes), "minute");
  }

  return rtf.format(Math.round(diffSeconds), "second");
}
