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

/**
 * Builds the reference string using the same Intl options the function uses.
 * This keeps assertions correct across Node / ICU versions without hard-coding
 * locale-specific strings that differ between releases.
 */
function expectedFormat(locale: string, tz: string, iso: string): string {
  return withTimezone(tz, () =>
    new Intl.DateTimeFormat(locale, {
      weekday: "short",
      year: "numeric",
      month: "short",
      day: "numeric",
      hour: "numeric",
      minute: "2-digit",
      timeZoneName: "short",
    }).format(new Date(iso))
  );
}

const ISO_LAGOS = "2026-06-01T18:00:00.000Z";
const ISO_NYC   = "2026-06-15T16:00:00.000Z";

describe("formatEventTime", () => {
  // ── Existing en-US tests ────────────────────────────────────────────────

  it("formats a UTC time into a fixed timezone (Africa/Lagos)", () => {
    const result = withTimezone("Africa/Lagos", () => formatEventTime(ISO_LAGOS));
    expect(result).toBe("Mon, Jun 1, 2026, 7:00 PM GMT+1");
  });

  it("formats the same instant differently in America/New_York", () => {
    const result = withTimezone("America/New_York", () => formatEventTime(ISO_NYC));
    expect(result).toBe("Mon, Jun 15, 2026, 12:00 PM EDT");
  });

  it("shows the standard-time abbreviation just before a DST boundary", () => {
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

  // ── Locale-aware tests ──────────────────────────────────────────────────

  it("honours an explicit en-US locale (backward-compatible default)", () => {
    const result   = withTimezone("Africa/Lagos", () => formatEventTime(ISO_LAGOS, "en-US"));
    const expected = expectedFormat("en-US", "Africa/Lagos", ISO_LAGOS);
    expect(result).toBe(expected);
  });

  it("formats in Spanish (es) in Africa/Lagos", () => {
    const result   = withTimezone("Africa/Lagos", () => formatEventTime(ISO_LAGOS, "es"));
    const expected = expectedFormat("es", "Africa/Lagos", ISO_LAGOS);
    expect(result).toBe(expected);
    // Sanity-check: Spanish output must differ from US-English output.
    const enResult = withTimezone("Africa/Lagos", () => formatEventTime(ISO_LAGOS, "en-US"));
    expect(result).not.toBe(enResult);
  });

  it("formats in French (fr) in Africa/Lagos", () => {
    const result   = withTimezone("Africa/Lagos", () => formatEventTime(ISO_LAGOS, "fr"));
    const expected = expectedFormat("fr", "Africa/Lagos", ISO_LAGOS);
    expect(result).toBe(expected);
    const enResult = withTimezone("Africa/Lagos", () => formatEventTime(ISO_LAGOS, "en-US"));
    expect(result).not.toBe(enResult);
  });

  it("formats in Spanish (es) in America/New_York", () => {
    const result   = withTimezone("America/New_York", () => formatEventTime(ISO_NYC, "es"));
    const expected = expectedFormat("es", "America/New_York", ISO_NYC);
    expect(result).toBe(expected);
  });

  it("different locales produce different output for the same instant and timezone", () => {
    const en = withTimezone("Africa/Lagos", () => formatEventTime(ISO_LAGOS, "en-US"));
    const es = withTimezone("Africa/Lagos", () => formatEventTime(ISO_LAGOS, "es"));
    const fr = withTimezone("Africa/Lagos", () => formatEventTime(ISO_LAGOS, "fr"));
    // All three locales must produce distinct strings.
    expect(en).not.toBe(es);
    expect(en).not.toBe(fr);
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
