/**
 * Returns a relative time string for a given date.
 * Uses Intl.RelativeTimeFormat to format the time difference.
 * Special-cases "Today" and "Tomorrow".
 * Picks the largest sensible unit (minutes → hours → days → weeks → months).
 */

export function getRelativeTime(date: Date): string {
  const now = new Date();
  const diffMs = date.getTime() - now.getTime();
  const diffSeconds = diffMs / 1000;
  const diffMinutes = diffSeconds / 60;
  const diffHours = diffMinutes / 60;
  const diffDays = diffHours / 24;
  const diffWeeks = diffDays / 7;
  const diffMonths = diffDays / 30;

  const rtf = new Intl.RelativeTimeFormat("en", { numeric: "auto" });

  // Check if it's today
  const isToday = date.toDateString() === now.toDateString();
  const tomorrow = new Date(now);
  tomorrow.setDate(tomorrow.getDate() + 1);
  const isTomorrow = date.toDateString() === tomorrow.toDateString();

  if (isToday) {
    return "Today";
  }

  if (isTomorrow) {
    return "Tomorrow";
  }

  // Pick the largest sensible unit
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
