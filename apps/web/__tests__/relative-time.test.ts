import { describe, it, expect, beforeEach, afterEach, vi } from "vitest";
import { getRelativeTime } from "@/utils/relative-time";

// Fixed "now" used across every test so results don't depend on when the
// suite happens to run. Pinned away from any timezone/DST edge and pinned
// to UTC so the calendar-day comparisons inside getRelativeTime are
// deterministic regardless of the host machine's local timezone.
const NOW = new Date("2026-06-15T12:00:00.000Z");

const MINUTE_MS = 60 * 1000;
const HOUR_MS = 60 * MINUTE_MS;
const DAY_MS = 24 * HOUR_MS;

function fromNow(offsetMs: number): Date {
  return new Date(NOW.getTime() + offsetMs);
}

/**
 * Builds the reference string directly from Intl.RelativeTimeFormat so that
 * assertions stay correct across Node/ICU versions without hard-coding locale-
 * specific strings that may differ between releases.
 */
function rtf(locale: string, value: number, unit: Intl.RelativeTimeFormatUnit): string {
  return new Intl.RelativeTimeFormat(locale, { numeric: "auto" }).format(value, unit);
}

describe("getRelativeTime", () => {
  let originalTZ: string | undefined;

  beforeEach(() => {
    originalTZ = process.env.TZ;
    process.env.TZ = "UTC";
    vi.useFakeTimers();
    vi.setSystemTime(NOW);
  });

  afterEach(() => {
    vi.useRealTimers();
    process.env.TZ = originalTZ;
  });

  // ── English (default) ───────────────────────────────────────────────────

  it('returns the Intl "today" label for a time later the same day (en)', () => {
    expect(getRelativeTime(fromNow(2 * HOUR_MS))).toBe(rtf("en", 0, "day"));
  });

  it('returns the Intl "today" label for a time earlier the same day (en)', () => {
    expect(getRelativeTime(fromNow(-2 * HOUR_MS))).toBe(rtf("en", 0, "day"));
  });

  it('returns the Intl "today" label for exactly now (en)', () => {
    expect(getRelativeTime(NOW)).toBe(rtf("en", 0, "day"));
  });

  it('returns the Intl "tomorrow" label for the next calendar day (en)', () => {
    expect(getRelativeTime(fromNow(DAY_MS))).toBe(rtf("en", 1, "day"));
  });

  it('returns the Intl "yesterday" label for the previous calendar day (en)', () => {
    expect(getRelativeTime(fromNow(-13 * HOUR_MS))).toBe(rtf("en", -1, "day"));
    expect(getRelativeTime(fromNow(-35 * HOUR_MS))).toBe(rtf("en", -1, "day"));
  });

  it("formats a few days into the future (en)", () => {
    expect(getRelativeTime(fromNow(3 * DAY_MS))).toBe("in 3 days");
  });

  it("formats a couple of weeks into the future (en)", () => {
    expect(getRelativeTime(fromNow(14 * DAY_MS))).toBe("in 2 weeks");
  });

  it("formats a couple of months into the future (en)", () => {
    expect(getRelativeTime(fromNow(60 * DAY_MS))).toBe("in 2 months");
  });

  it("formats a few days in the past (en)", () => {
    expect(getRelativeTime(fromNow(-3 * DAY_MS))).toBe("3 days ago");
  });

  it("formats a couple of months in the past (en)", () => {
    expect(getRelativeTime(fromNow(-60 * DAY_MS))).toBe("2 months ago");
  });

  it("uses years for dates at least twelve months away (en)", () => {
    expect(getRelativeTime(fromNow(359 * DAY_MS))).toBe("in 12 months");
    expect(getRelativeTime(fromNow(360 * DAY_MS))).toBe("next year");
    expect(getRelativeTime(fromNow(-360 * DAY_MS))).toBe("last year");
    expect(getRelativeTime(fromNow(720 * DAY_MS))).toBe("in 2 years");
  });

  it("does not throw for an invalid date and falls back to an empty string", () => {
    const invalidDate = new Date("not-a-date");
    expect(() => getRelativeTime(invalidDate)).not.toThrow();
    expect(getRelativeTime(invalidDate)).toBe("");
  });

  // ── Spanish (es) ────────────────────────────────────────────────────────

  it('returns the Intl "today" label in Spanish for a time the same day', () => {
    const result = getRelativeTime(fromNow(2 * HOUR_MS), "es");
    expect(result).toBe(rtf("es", 0, "day"));
  });

  it('returns the Intl "tomorrow" label in Spanish', () => {
    const result = getRelativeTime(fromNow(DAY_MS), "es");
    expect(result).toBe(rtf("es", 1, "day"));
  });

  it('returns the Intl "yesterday" label in Spanish', () => {
    const result = getRelativeTime(fromNow(-13 * HOUR_MS), "es");
    expect(result).toBe(rtf("es", -1, "day"));
  });

  it("formats days into the future in Spanish", () => {
    const result = getRelativeTime(fromNow(3 * DAY_MS), "es");
    expect(result).toBe(rtf("es", 3, "day"));
  });

  it("formats days in the past in Spanish", () => {
    const result = getRelativeTime(fromNow(-3 * DAY_MS), "es");
    expect(result).toBe(rtf("es", -3, "day"));
  });

  it("formats weeks in Spanish", () => {
    const result = getRelativeTime(fromNow(14 * DAY_MS), "es");
    expect(result).toBe(rtf("es", 2, "week"));
  });

  it("formats months in Spanish", () => {
    const result = getRelativeTime(fromNow(60 * DAY_MS), "es");
    expect(result).toBe(rtf("es", 2, "month"));
  });

  // ── Locale contrast: Spanish differs from English ───────────────────────

  it('"today" label is different in Spanish vs English', () => {
    const en = getRelativeTime(fromNow(2 * HOUR_MS), "en");
    const es = getRelativeTime(fromNow(2 * HOUR_MS), "es");
    expect(en).not.toBe(es);
  });

  it('"tomorrow" label is different in Spanish vs English', () => {
    const en = getRelativeTime(fromNow(DAY_MS), "en");
    const es = getRelativeTime(fromNow(DAY_MS), "es");
    expect(en).not.toBe(es);
  });

  it('"yesterday" label is different in Spanish vs English', () => {
    const en = getRelativeTime(fromNow(-13 * HOUR_MS), "en");
    const es = getRelativeTime(fromNow(-13 * HOUR_MS), "es");
    expect(en).not.toBe(es);
  });

  it("relative days differ between Spanish and English", () => {
    const en = getRelativeTime(fromNow(3 * DAY_MS), "en");
    const es = getRelativeTime(fromNow(3 * DAY_MS), "es");
    expect(en).not.toBe(es);
  });

  // ── Invalid date still returns empty regardless of locale ────────────────

  it("returns empty string for an invalid date regardless of locale", () => {
    const invalid = new Date("not-a-date");
    expect(getRelativeTime(invalid, "es")).toBe("");
    expect(getRelativeTime(invalid, "fr")).toBe("");
  });
});
