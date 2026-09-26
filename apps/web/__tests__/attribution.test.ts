import { describe, it, expect, beforeEach, afterEach, vi } from "vitest";
import { captureAttribution, getCheckoutAttribution } from "@/utils/attribution";

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
    expect(JSON.parse(localStorage.getItem(STORAGE_KEY) ?? "null")).toEqual(result);
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
});
