import { describe, it, expect, beforeEach, afterEach, vi } from "vitest";
import {
  ATTRIBUTION_MAX_AGE_MS,
  captureAttribution,
  getCheckoutAttribution,
} from "@/utils/attribution";

const STORAGE_KEY = "agora_checkout_attribution";

describe("attribution", () => {
  beforeEach(() => {
    localStorage.clear();
  });

  afterEach(() => {
    vi.restoreAllMocks();
  });

  it("stores utm_source, utm_medium, and utm_campaign from search params", () => {
    const params = new URLSearchParams(
      "utm_source=twitter&utm_medium=social&utm_campaign=launch"
    );

    const result = captureAttribution(params);

    expect(result).toEqual({
      utmSource: "twitter",
      utmMedium: "social",
      utmCampaign: "launch",
    });
    expect(JSON.parse(localStorage.getItem(STORAGE_KEY) ?? "null")).toEqual({
      ...result,
      capturedAt: expect.any(Number),
    });
  });

  it("returns null and stores nothing when there are no UTM params", () => {
    localStorage.setItem(
      STORAGE_KEY,
      JSON.stringify({ utmSource: "existing", utmMedium: "email", utmCampaign: "old" })
    );

    const result = captureAttribution(new URLSearchParams("ref=homepage"));

    expect(result).toBeNull();
    expect(JSON.parse(localStorage.getItem(STORAGE_KEY) ?? "null")).toEqual({
      utmSource: "existing",
      utmMedium: "email",
      utmCampaign: "old",
    });
  });

  it("getCheckoutAttribution returns what was captured", () => {
    captureAttribution(
      new URLSearchParams("utm_source=newsletter&utm_campaign=spring")
    );

    expect(getCheckoutAttribution()).toEqual({
      utmSource: "newsletter",
      utmMedium: undefined,
      utmCampaign: "spring",
    });
  });

  it("trims values and ignores blank UTM params", () => {
    const result = captureAttribution(
      new URLSearchParams("utm_source=%20twitter%20&utm_medium=&utm_campaign=%20")
    );

    expect(result).toEqual({
      utmSource: "twitter",
      utmMedium: undefined,
      utmCampaign: undefined,
    });
  });

  it("does not crash when localStorage throws (private mode)", () => {
    vi.spyOn(Storage.prototype, "setItem").mockImplementation(() => {
      throw new Error("QuotaExceededError");
    });

    expect(() =>
      captureAttribution(new URLSearchParams("utm_source=private"))
    ).not.toThrow();

    expect(
      captureAttribution(new URLSearchParams("utm_source=private"))
    ).toEqual({
      utmSource: "private",
      utmMedium: undefined,
      utmCampaign: undefined,
    });
  });

  it("getCheckoutAttribution returns undefined when storage throws or is empty", () => {
    expect(getCheckoutAttribution()).toBeUndefined();

    vi.spyOn(Storage.prototype, "getItem").mockImplementation(() => {
      throw new Error("SecurityError");
    });

    expect(getCheckoutAttribution()).toBeUndefined();
  });

  it("getCheckoutAttribution ignores corrupt stored values", () => {
    localStorage.setItem(STORAGE_KEY, "not-json");
    expect(getCheckoutAttribution()).toBeUndefined();

    localStorage.setItem(STORAGE_KEY, JSON.stringify({ utmSource: 123 }));
    expect(getCheckoutAttribution()).toBeUndefined();
  });
  describe("30-day expiry", () => {
    const DAY_MS = 24 * 60 * 60 * 1000;
    const NOW = new Date("2026-11-24T12:00:00Z");

    beforeEach(() => {
      vi.useFakeTimers();
      vi.setSystemTime(NOW);
    });

    afterEach(() => {
      vi.useRealTimers();
    });

    it("window is 30 days", () => {
      expect(ATTRIBUTION_MAX_AGE_MS).toBe(30 * DAY_MS);
    });

    it("stores capturedAt as the capture time", () => {
      captureAttribution(new URLSearchParams("utm_source=twitter"));
      expect(JSON.parse(localStorage.getItem(STORAGE_KEY) ?? "null")).toEqual({
        utmSource: "twitter",
        capturedAt: NOW.getTime(),
      });
    });

    it("returns a fresh entry, including one exactly 30 days old", () => {
      captureAttribution(new URLSearchParams("utm_source=twitter&utm_campaign=launch"));

      vi.advanceTimersByTime(29 * DAY_MS);
      expect(getCheckoutAttribution()).toEqual({
        utmSource: "twitter",
        utmMedium: undefined,
        utmCampaign: "launch",
      });

      vi.advanceTimersByTime(DAY_MS);
      expect(getCheckoutAttribution()?.utmCampaign).toBe("launch");
      expect(localStorage.getItem(STORAGE_KEY)).not.toBeNull();
    });

    it("ignores and removes an entry older than 30 days", () => {
      captureAttribution(new URLSearchParams("utm_source=twitter&utm_campaign=launch"));

      vi.advanceTimersByTime(30 * DAY_MS + 1);

      expect(getCheckoutAttribution()).toBeUndefined();
      expect(localStorage.getItem(STORAGE_KEY)).toBeNull();
    });

    it("ignores and removes a legacy entry without capturedAt", () => {
      localStorage.setItem(
        STORAGE_KEY,
        JSON.stringify({ utmSource: "existing", utmMedium: "email", utmCampaign: "old" })
      );

      expect(getCheckoutAttribution()).toBeUndefined();
      expect(localStorage.getItem(STORAGE_KEY)).toBeNull();
    });

    it("ignores entries whose capturedAt is not a usable timestamp", () => {
      for (const capturedAt of ["2026-11-01", null, NOW.getTime() + DAY_MS]) {
        localStorage.setItem(STORAGE_KEY, JSON.stringify({ utmSource: "x", capturedAt }));
        expect(getCheckoutAttribution()).toBeUndefined();
        expect(localStorage.getItem(STORAGE_KEY)).toBeNull();
      }
    });

    it("a new capture restarts the window", () => {
      captureAttribution(new URLSearchParams("utm_campaign=first"));
      vi.advanceTimersByTime(20 * DAY_MS);
      captureAttribution(new URLSearchParams("utm_campaign=second"));
      vi.advanceTimersByTime(20 * DAY_MS);

      expect(getCheckoutAttribution()?.utmCampaign).toBe("second");
    });
  });
});
