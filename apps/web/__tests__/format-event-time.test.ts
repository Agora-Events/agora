import { describe, it, expect, afterEach } from "vitest";
import { formatEventTime, getTimezone } from "@/utils/format-event-time";

/**
 * formatEventTime() and getTimezone() both read the Node/ICU timezone at
 * call time via Intl, so tests drive them by setting process.env.TZ before
 * each assertion (Node respects a TZ change for the next Intl/Date call)
 * and restoring the original value afterwards.
 */
function withTimezone<T>(tz: string, fn: () => T): T {
  const original = process.env.TZ;
  process.env.TZ = tz;
  try {
    return fn();
  } finally {
    process.env.TZ = original;
  }
}

describe("formatEventTime", () => {
  it("formats a UTC time into a fixed timezone (Africa/Lagos)", () => {
    const result = withTimezone("Africa/Lagos", () => formatEventTime("2026-06-01T18:00:00.000Z"));
    expect(result).toBe("Mon, Jun 1, 2026, 7:00 PM GMT+1");
  });

  it("formats the same instant differently in America/New_York", () => {
    const result = withTimezone("America/New_York", () =>
      formatEventTime("2026-06-15T16:00:00.000Z")
    );
    expect(result).toBe("Mon, Jun 15, 2026, 12:00 PM EDT");
  });

  it("shows the standard-time abbreviation just before a DST boundary", () => {
    // US clocks spring forward at 2:00am EST on 8 March 2026 (07:00 UTC).
    const result = withTimezone("America/New_York", () =>
      formatEventTime("2026-03-08T06:59:00.000Z")
    );
    expect(result).toBe("Sun, Mar 8, 2026, 1:59 AM EST");
  });

  it("shows the daylight-saving abbreviation just after the DST boundary", () => {
    const result = withTimezone("America/New_York", () =>
      formatEventTime("2026-03-08T07:01:00.000Z")
    );
    expect(result).toBe("Sun, Mar 8, 2026, 3:01 AM EDT");
  });

  it("falls back to an empty string instead of throwing for an invalid timestamp", () => {
    expect(() => formatEventTime("not-a-date")).not.toThrow();
    expect(formatEventTime("not-a-date")).toBe("");
  });

  it("falls back to an empty string for a missing/empty timestamp", () => {
    expect(formatEventTime("")).toBe("");
  });
});

describe("getTimezone", () => {
  afterEach(() => {
    delete process.env.TZ;
  });

  it("returns a valid IANA timezone name", () => {
    process.env.TZ = "Europe/London";
    expect(getTimezone()).toBe("Europe/London");
  });

  it("reflects a different configured timezone", () => {
    process.env.TZ = "Africa/Lagos";
    expect(getTimezone()).toBe("Africa/Lagos");
  });

  it("returns a non-empty string usable as a timezone label", () => {
    process.env.TZ = "America/New_York";
    const tz = getTimezone();
    expect(typeof tz).toBe("string");
    expect(tz.length).toBeGreaterThan(0);
  });
});
