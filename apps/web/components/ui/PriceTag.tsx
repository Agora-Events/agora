"use client";

import { useEffect, useRef, useState } from "react";
import {
  convertUsdcToFiat,
  fetchUsdcRate,
  type FiatCurrency,
} from "@/utils/usdc-rate";

// ─── Types ────────────────────────────────────────────────────────────────────

export interface PriceTagProps {
  /** Amount expressed in USDC (e.g. 25 for 25 USDC). */
  amountUsdc: number;
  /**
   * Fiat currency to display alongside the USDC price.
   * Defaults to "usd".
   */
  currency?: FiatCurrency;
  /**
   * Additional Tailwind classes to apply to the root wrapper element.
   * Useful for spacing in parent layouts.
   */
  className?: string;
}

// ─── Currency symbols ─────────────────────────────────────────────────────────

const CURRENCY_SYMBOL: Record<FiatCurrency, string> = {
  usd: "$",
  eur: "€",
};

const CURRENCY_LABEL: Record<FiatCurrency, string> = {
  usd: "USD",
  eur: "EUR",
};

// ─── Component ────────────────────────────────────────────────────────────────

/**
 * PriceTag
 *
 * Displays a USDC amount and its approximate fiat value beneath it.
 *
 * ```tsx
 * <PriceTag amountUsdc={25} currency="usd" />
 * // → "25.00 USDC"
 * //   "≈ $25.01 USD"
 * ```
 *
 * The fiat conversion is fetched from CoinGecko and cached for 5 minutes.
 * If the fetch fails the component falls back to a sensible default rate
 * and signals this to the user with a "(est.)" annotation.
 */
export function PriceTag({
  amountUsdc,
  currency = "usd",
  className = "",
}: PriceTagProps) {
  const [fiatAmount, setFiatAmount] = useState<number | null>(null);
  const [isFallback, setIsFallback] = useState(false);
  const [isLoading, setIsLoading] = useState(true);
  const abortRef = useRef<AbortController | null>(null);

  useEffect(() => {
    // Cancel any in-flight request when props change or component unmounts
    abortRef.current?.abort();
    const controller = new AbortController();
    abortRef.current = controller;

    setIsLoading(true);

    fetchUsdcRate(currency, controller.signal)
      .then((result) => {
        setFiatAmount(convertUsdcToFiat(amountUsdc, result.rate));
        setIsFallback(result.isFallback);
      })
      .catch((err: unknown) => {
        // AbortError fires when the component unmounts – ignore silently
        if (err instanceof DOMException && err.name === "AbortError") return;
        // Any other unexpected error: keep the UI in a clean fallback state
        console.error("[PriceTag] Unexpected error fetching USDC rate:", err);
        setFiatAmount(convertUsdcToFiat(amountUsdc, currency === "eur" ? 0.92 : 1.0));
        setIsFallback(true);
      })
      .finally(() => {
        setIsLoading(false);
      });

    return () => {
      controller.abort();
    };
  }, [amountUsdc, currency]);

  // ── Derived display values ──────────────────────────────────────────────────

  // USDC uses 7 decimal places on-chain, but we display 2 for readability
  const usdcDisplay = amountUsdc.toFixed(2);

  const symbol = CURRENCY_SYMBOL[currency];
  const label = CURRENCY_LABEL[currency];

  const fiatDisplay =
    fiatAmount !== null ? fiatAmount.toFixed(2) : null;

  // ── Render ──────────────────────────────────────────────────────────────────

  return (
    <div
      className={`inline-flex flex-col gap-0.5 ${className}`}
      // Accessible label so screen readers announce the full price upfront
      aria-label={`${usdcDisplay} USDC${fiatDisplay !== null ? `, approximately ${symbol}${fiatDisplay} ${label}` : ""}`}
    >
      {/* Primary: USDC amount */}
      <span className="font-semibold text-[20px] leading-tight text-black">
        {usdcDisplay}{" "}
        <span className="text-[14px] font-medium text-black/60">USDC</span>
      </span>

      {/* Secondary: fiat conversion */}
      <span
        className="text-[13px] leading-snug text-black/50 font-normal"
        aria-hidden="true"
      >
        {isLoading ? (
          // Skeleton shimmer while the rate is loading
          <span
            className="inline-block h-3.5 w-24 rounded bg-black/10 animate-pulse"
            role="status"
            aria-label="Loading conversion rate"
          />
        ) : fiatDisplay !== null ? (
          <>
            ≈&nbsp;{symbol}
            {fiatDisplay}&nbsp;{label}
            {isFallback && (
              <span
                className="ml-1 text-black/35"
                title="Live rate unavailable – using estimated value"
              >
                (est.)
              </span>
            )}
          </>
        ) : (
          // Should not normally be reached, but keeps JSX exhaustive
          <span className="text-black/35">Conversion unavailable</span>
        )}
      </span>
    </div>
  );
}
