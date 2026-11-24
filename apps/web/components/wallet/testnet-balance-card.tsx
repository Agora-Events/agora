"use client";

import { useState, useEffect, useCallback } from "react";

// Component only renders when NEXT_PUBLIC_STELLAR_NETWORK === 'TESTNET' (Issue #1503).

const HORIZON_TESTNET = "https://horizon-testnet.stellar.org";
const TESTNET_USDC_ISSUER = "GBBD47IF6LWK7P7MDEVSCWR7DPUWV3NY3DTQEVFL4NAT4AQH3ZLLFLA5";
/** Stellar base reserve per subentry (0.5 XLM). */
const BASE_RESERVE_XLM = 0.5;
/** Minimum account reserve — 2 base entries (account itself). */
const MIN_ACCOUNT_ENTRIES = 2;

// ─── Types ────────────────────────────────────────────────────────────────────

interface HorizonBalance {
  asset_type: "native" | "credit_alphanum4" | "credit_alphanum12";
  asset_code?: string;
  asset_issuer?: string;
  balance: string;
}

interface HorizonAccount {
  balances: HorizonBalance[];
  subentry_count: number;
}

interface BalanceData {
  totalXlm: number;
  availableXlm: number;
  reservedXlm: number;
  usdc: number;
}

// ─── Helpers ──────────────────────────────────────────────────────────────────

function formatUsdc(amount: number): string {
  return amount.toFixed(7);
}

function formatXlm(amount: number): string {
  return amount.toLocaleString("en-US", { minimumFractionDigits: 2, maximumFractionDigits: 7 });
}

async function fetchTestnetBalances(publicKey: string): Promise<BalanceData> {
  const res = await fetch(`${HORIZON_TESTNET}/accounts/${encodeURIComponent(publicKey)}`);
  if (!res.ok) {
    throw new Error(`Horizon returned ${res.status} for account ${publicKey}`);
  }
  const account: HorizonAccount = await res.json();

  const nativeBal = account.balances.find((b) => b.asset_type === "native");
  const totalXlm = nativeBal ? parseFloat(nativeBal.balance) : 0;

  // Reserve = (2 + subentry_count) * 0.5 XLM
  const reservedXlm = (MIN_ACCOUNT_ENTRIES + (account.subentry_count ?? 0)) * BASE_RESERVE_XLM;
  const availableXlm = Math.max(0, totalXlm - reservedXlm);

  const usdcBal = account.balances.find(
    (b) =>
      b.asset_type !== "native" &&
      b.asset_code === "USDC" &&
      b.asset_issuer === TESTNET_USDC_ISSUER
  );
  const usdc = usdcBal ? parseFloat(usdcBal.balance) : 0;

  return { totalXlm, availableXlm, reservedXlm, usdc };
}

// ─── Sub-components ───────────────────────────────────────────────────────────

function BalanceRow({ label, value, mono = false }: { label: string; value: string; mono?: boolean }) {
  return (
    <div className="flex items-center justify-between py-2 border-b border-border-warm/50 last:border-0">
      <span className="text-sm text-muted-text">{label}</span>
      <span className={`text-sm font-semibold text-ink-soft ${mono ? "font-mono" : ""}`}>{value}</span>
    </div>
  );
}

function SpinIcon({ spinning }: { spinning: boolean }) {
  return (
    <svg
      xmlns="http://www.w3.org/2000/svg"
      width={16}
      height={16}
      viewBox="0 0 24 24"
      fill="none"
      stroke="currentColor"
      strokeWidth={2.5}
      strokeLinecap="round"
      strokeLinejoin="round"
      aria-hidden="true"
      className={spinning ? "animate-spin" : ""}
    >
      <path d="M21 12a9 9 0 1 1-6.219-8.56" />
    </svg>
  );
}

// ─── Component ────────────────────────────────────────────────────────────────

export interface TestnetBalanceCardProps {
  /** G... Stellar public key to query. */
  publicKey: string;
}

/**
 * Displays Testnet XLM and USDC balances with reserve breakdown.
 * Only renders when NEXT_PUBLIC_STELLAR_NETWORK === 'TESTNET'.
 *
 * Closes #1503
 */
export function TestnetBalanceCard({ publicKey }: TestnetBalanceCardProps) {
  // Guard: only render on Testnet
  if (process.env.NEXT_PUBLIC_STELLAR_NETWORK !== "TESTNET") {
    return null;
  }

  return <TestnetBalanceCardInner publicKey={publicKey} />;
}

function TestnetBalanceCardInner({ publicKey }: { publicKey: string }) {
  const [balances, setBalances] = useState<BalanceData | null>(null);
  const [isLoading, setIsLoading] = useState(true);
  const [error, setError] = useState<string | null>(null);

  const refresh = useCallback(async () => {
    if (!publicKey) return;
    setIsLoading(true);
    setError(null);
    try {
      const data = await fetchTestnetBalances(publicKey);
      setBalances(data);
    } catch (err) {
      setError(err instanceof Error ? err.message : "Failed to fetch balances.");
    } finally {
      setIsLoading(false);
    }
  }, [publicKey]);

  useEffect(() => {
    refresh();
  }, [refresh]);

  return (
    <section
      aria-labelledby="testnet-balance-heading"
      className="rounded-2xl border border-border-warm bg-white shadow-sm overflow-hidden"
    >
      {/* Header */}
      <div className="flex items-center justify-between px-6 py-4 border-b border-border-warm">
        <div>
          <h2
            id="testnet-balance-heading"
            className="text-base font-bold text-ink-soft flex items-center gap-2"
          >
            <span className="inline-flex items-center px-2 py-0.5 rounded-full text-[10px] font-bold tracking-widest uppercase bg-amber-100 text-amber-800 border border-amber-200">
              Testnet
            </span>
            Balance Breakdown
          </h2>
          <p className="text-xs text-muted-text mt-0.5">Stellar Horizon Testnet</p>
        </div>
        <button
          type="button"
          onClick={refresh}
          disabled={isLoading}
          aria-label="Refresh balance"
          className="inline-flex items-center gap-1.5 text-xs font-semibold px-3 py-1.5 rounded-lg border border-border-warm bg-surface hover:bg-surface-alt transition-colors disabled:opacity-50"
        >
          <SpinIcon spinning={isLoading} />
          Refresh Balance
        </button>
      </div>

      {/* Body */}
      <div className="px-6 py-4">
        {error && (
          <p
            role="alert"
            className="text-xs text-error bg-error/10 border border-error/20 rounded-lg px-3 py-2 mb-4"
          >
            {error}
          </p>
        )}

        {isLoading && !balances ? (
          <div className="space-y-3 animate-pulse">
            {[1, 2, 3, 4].map((i) => (
              <div key={i} className="h-8 rounded-lg bg-surface" />
            ))}
          </div>
        ) : balances ? (
          <div>
            {/* XLM */}
            <p className="text-xs font-bold uppercase tracking-wider text-muted-text mb-2">XLM</p>
            <div className="mb-4">
              <BalanceRow
                label="Total Balance"
                value={`${formatXlm(balances.totalXlm)} XLM`}
                mono
              />
              <BalanceRow
                label={`Reserved (${MIN_ACCOUNT_ENTRIES} + subentries × 0.5)`}
                value={`−${formatXlm(balances.reservedXlm)} XLM`}
                mono
              />
              <BalanceRow
                label="Available"
                value={`${formatXlm(balances.availableXlm)} XLM`}
                mono
              />
            </div>

            {/* USDC */}
            <p className="text-xs font-bold uppercase tracking-wider text-muted-text mb-2">USDC</p>
            <BalanceRow
              label="Testnet USDC Balance"
              value={`${formatUsdc(balances.usdc)} USDC`}
              mono
            />

            <p className="mt-4 text-[11px] text-muted-text/70 leading-relaxed">
              Reserve calculation: <span className="font-mono">(2 + subentries) × 0.5 XLM</span> is
              locked by the Stellar protocol and cannot be spent.
            </p>
          </div>
        ) : null}
      </div>
    </section>
  );
}

export default TestnetBalanceCard;
