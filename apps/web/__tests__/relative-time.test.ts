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

  it('returns "Today" for a time later the same day', () => {
    expect(getRelativeTime(fromNow(2 * HOUR_MS))).toBe("Today");
  });

  it('returns "Today" for a time earlier the same day', () => {
    expect(getRelativeTime(fromNow(-2 * HOUR_MS))).toBe("Today");
  });

  it('returns "Today" for exactly now', () => {
    expect(getRelativeTime(NOW)).toBe("Today");
  });

  it('returns "Tomorrow" for a date on the next calendar day', () => {
    expect(getRelativeTime(fromNow(DAY_MS))).toBe("Tomorrow");
  });

  it("formats a few days into the future", () => {
    expect(getRelativeTime(fromNow(3 * DAY_MS))).toBe("in 3 days");
  });

  it("formats a couple of weeks into the future", () => {
    expect(getRelativeTime(fromNow(14 * DAY_MS))).toBe("in 2 weeks");
  });

  it("formats a couple of months into the future", () => {
    expect(getRelativeTime(fromNow(60 * DAY_MS))).toBe("in 2 months");
  });

  it("formats a few days in the past", () => {
    expect(getRelativeTime(fromNow(-3 * DAY_MS))).toBe("3 days ago");
  });

  it("formats a couple of months in the past", () => {
    expect(getRelativeTime(fromNow(-60 * DAY_MS))).toBe("2 months ago");
  });

  it("does not throw for an invalid date and falls back to an empty string", () => {
    const invalidDate = new Date("not-a-date");
    expect(() => getRelativeTime(invalidDate)).not.toThrow();
    expect(getRelativeTime(invalidDate)).toBe("");
  });
});
