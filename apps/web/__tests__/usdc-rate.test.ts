import { afterEach, beforeEach, describe, expect, it, vi } from "vitest";
import { convertUsdcToFiat } from "@/utils/usdc-rate";

// ─── Helpers ──────────────────────────────────────────────────────────────────

const MOCK_API_RESPONSE = {
  "usd-coin": { usd: 1.001, eur: 0.924 },
};

function mockFetch(
  response: unknown,
  status = 200,
  ok = true,
): ReturnType<typeof vi.fn> {
  return vi.fn(async () => ({
    ok,
    status,
    statusText: ok ? "OK" : "Internal Server Error",
    json: async () => response,
  }));
}

// ─── Tests ────────────────────────────────────────────────────────────────────

describe("convertUsdcToFiat", () => {
  it("converts USDC to USD using provided rate", () => {
    expect(convertUsdcToFiat(25, 1.0)).toBe(25.0);
    expect(convertUsdcToFiat(25, 1.001)).toBe(25.03);
  });

  it("converts USDC to EUR using provided rate", () => {
    expect(convertUsdcToFiat(100, 0.924)).toBe(92.4);
  });

  it("rounds correctly to 2 decimal places", () => {
    // 10 * 0.333 = 3.33 (not 3.330000000000001)
    expect(convertUsdcToFiat(10, 0.333)).toBe(3.33);
  });

  it("supports custom decimal precision", () => {
    expect(convertUsdcToFiat(10, 1.0, 4)).toBe(10.0);
    expect(convertUsdcToFiat(10, 1.001, 3)).toBe(10.01);
  });

  it("returns 0 for 0 USDC input", () => {
    expect(convertUsdcToFiat(0, 1.001)).toBe(0);
  });
});

describe("fetchUsdcRate", () => {
  beforeEach(() => {
    vi.resetModules();
    // Reset the module-level cache between tests by re-importing fresh
    vi.stubGlobal("fetch", mockFetch(MOCK_API_RESPONSE));
  });

  afterEach(() => {
    vi.restoreAllMocks();
    vi.unstubAllGlobals();
  });

  it("returns USD rate from CoinGecko API", async () => {
    vi.stubGlobal("fetch", mockFetch(MOCK_API_RESPONSE));

    // Use dynamic import to bypass the module-level cache
    const { fetchUsdcRate: freshFetch } = await import("@/utils/usdc-rate");
    const result = await freshFetch("usd");

    expect(result.currency).toBe("usd");
    expect(result.isFallback).toBe(false);
    expect(typeof result.rate).toBe("number");
  });

  it("returns EUR rate from CoinGecko API", async () => {
    vi.stubGlobal("fetch", mockFetch(MOCK_API_RESPONSE));

    const { fetchUsdcRate: freshFetch } = await import("@/utils/usdc-rate");
    const result = await freshFetch("eur");

    expect(result.currency).toBe("eur");
    expect(result.isFallback).toBe(false);
    expect(typeof result.rate).toBe("number");
  });

  it("falls back to USD rate of 1.0 when fetch fails", async () => {
    vi.stubGlobal(
      "fetch",
      vi.fn(async () => {
        throw new Error("Network Error");
      }),
    );

    const { fetchUsdcRate: freshFetch } = await import("@/utils/usdc-rate");
    const result = await freshFetch("usd");

    expect(result.isFallback).toBe(true);
    expect(result.rate).toBe(1.0);
    expect(result.currency).toBe("usd");
  });

  it("falls back to EUR rate of 0.92 when fetch fails", async () => {
    vi.stubGlobal(
      "fetch",
      vi.fn(async () => {
        throw new Error("Network Error");
      }),
    );

    const { fetchUsdcRate: freshFetch } = await import("@/utils/usdc-rate");
    const result = await freshFetch("eur");

    expect(result.isFallback).toBe(true);
    expect(result.rate).toBe(0.92);
    expect(result.currency).toBe("eur");
  });

  it("falls back when API returns non-ok response", async () => {
    vi.stubGlobal("fetch", mockFetch({}, 503, false));

    const { fetchUsdcRate: freshFetch } = await import("@/utils/usdc-rate");
    const result = await freshFetch("usd");

    expect(result.isFallback).toBe(true);
  });

  it("falls back when API returns unexpected shape", async () => {
    vi.stubGlobal("fetch", mockFetch({ unexpected: "shape" }));

    const { fetchUsdcRate: freshFetch } = await import("@/utils/usdc-rate");
    const result = await freshFetch("usd");

    expect(result.isFallback).toBe(true);
  });

  it("re-throws AbortError so callers can handle cancellation", async () => {
    const abortError = new DOMException("Aborted", "AbortError");
    vi.stubGlobal(
      "fetch",
      vi.fn(async () => {
        throw abortError;
      }),
    );

    const { fetchUsdcRate: freshFetch } = await import("@/utils/usdc-rate");
    await expect(freshFetch("usd")).rejects.toThrow("Aborted");
  });
});
