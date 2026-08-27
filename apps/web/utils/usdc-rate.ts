/**
 * USDC Exchange Rate Fetcher
 *
 * Fetches the current USDC price in USD and EUR from the CoinGecko public API.
 * Falls back to a fixed 1:1 USD peg if the fetch fails, since USDC is a
 * USD-backed stablecoin and its market price deviates by only a few basis points.
 *
 * Usage:
 *   const rate = await fetchUsdcRate("usd");
 *   const converted = 25 * rate; // e.g. 25.01
 */

export type FiatCurrency = "usd" | "eur";

export interface UsdcRateResult {
  /** The exchange rate: 1 USDC in the requested fiat currency */
  rate: number;
  /** Whether this value is a fallback due to a failed fetch */
  isFallback: boolean;
  /** Currency code returned */
  currency: FiatCurrency;
}

/**
 * USDC is pegged to USD, so 1 USDC ≈ 1 USD at all times.
 * For EUR we use a reasonable fallback that keeps the UI functional.
 */
const FALLBACK_RATES: Record<FiatCurrency, number> = {
  usd: 1.0,
  eur: 0.92,
};

const COINGECKO_API_URL =
  "https://api.coingecko.com/api/v3/simple/price?ids=usd-coin&vs_currencies=usd%2Ceur";

/** Cache the last successful fetch to avoid hammering the API on re-renders. */
let _cache: { data: Record<FiatCurrency, number>; fetchedAt: number } | null =
  null;
const CACHE_TTL_MS = 5 * 60 * 1000; // 5 minutes

/**
 * Fetch the current USDC exchange rate for the given fiat currency.
 *
 * @param currency - Target fiat currency: "usd" (default) or "eur".
 * @param signal   - Optional AbortSignal to cancel the request.
 * @returns        - An object containing the rate, currency, and whether it is a fallback.
 */
export async function fetchUsdcRate(
  currency: FiatCurrency = "usd",
  signal?: AbortSignal,
): Promise<UsdcRateResult> {
  // Return cached value if it is still fresh
  if (_cache && Date.now() - _cache.fetchedAt < CACHE_TTL_MS) {
    return {
      rate: _cache.data[currency],
      isFallback: false,
      currency,
    };
  }

  try {
    const response = await fetch(COINGECKO_API_URL, {
      signal,
      // Instruct browsers/CDNs to revalidate after 5 minutes
      next: { revalidate: 300 },
    } as RequestInit);

    if (!response.ok) {
      throw new Error(
        `CoinGecko responded with status ${response.status}: ${response.statusText}`,
      );
    }

    const json = (await response.json()) as {
      "usd-coin"?: { usd?: number; eur?: number };
    };

    const usdRate = json["usd-coin"]?.usd;
    const eurRate = json["usd-coin"]?.eur;

    if (typeof usdRate !== "number" || typeof eurRate !== "number") {
      throw new Error("Unexpected CoinGecko response shape");
    }

    _cache = {
      data: { usd: usdRate, eur: eurRate },
      fetchedAt: Date.now(),
    };

    return {
      rate: _cache.data[currency],
      isFallback: false,
      currency,
    };
  } catch (err) {
    // Do not re-throw when the request was intentionally cancelled
    if (err instanceof DOMException && err.name === "AbortError") {
      throw err;
    }

    console.warn(
      "[usdc-rate] Failed to fetch USDC rate from CoinGecko, using fallback:",
      err,
    );

    return {
      rate: FALLBACK_RATES[currency],
      isFallback: true,
      currency,
    };
  }
}

/**
 * Convert a USDC amount to fiat using the given rate.
 * Uses fixed-point arithmetic to avoid floating-point rounding artefacts.
 *
 * @param amountUsdc - Amount in USDC.
 * @param rate       - Exchange rate (1 USDC → fiat).
 * @param decimals   - Number of decimal places (default: 2).
 * @returns          - Converted fiat value as a number rounded to `decimals` places.
 */
export function convertUsdcToFiat(
  amountUsdc: number,
  rate: number,
  decimals = 2,
): number {
  const factor = Math.pow(10, decimals);
  return Math.round(amountUsdc * rate * factor) / factor;
}
