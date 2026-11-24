/**
 * Returns a relative time string for a given date.
 * Uses Intl.RelativeTimeFormat to format the time difference.
 * Special-cases "Yesterday", "Today", and "Tomorrow".
 * Picks the largest sensible unit (minutes → hours → days → weeks → months → years).
 */

export function getRelativeTime(date: Date): string {
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

  const rtf = new Intl.RelativeTimeFormat("en", { numeric: "auto" });

  const calendarDate = date.toDateString();
  const isToday = calendarDate === now.toDateString();
  const tomorrow = new Date(now);
  tomorrow.setDate(tomorrow.getDate() + 1);
  const isTomorrow = calendarDate === tomorrow.toDateString();

  if (isToday) {
    return "Today";
  }

  if (isTomorrow) {
    return "Tomorrow";
  }

  const yesterday = new Date(now);
  yesterday.setDate(yesterday.getDate() - 1);
  if (calendarDate === yesterday.toDateString()) {
    return "Yesterday";
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
