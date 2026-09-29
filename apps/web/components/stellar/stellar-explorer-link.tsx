"use client";

import React, { useCallback, useState } from "react";
import { toast } from "sonner";

// ─── Types ────────────────────────────────────────────────────────────────────

export type StellarNetwork = "testnet" | "mainnet";

interface StellarExplorerLinkProps {
  /** The transaction hash (64-char hex string). */
  txHash: string;
  /** The Stellar network. Defaults to NEXT_PUBLIC_STELLAR_NETWORK env var. */
  network?: StellarNetwork;
  /**
   * Visual variant.
   * - `"link"` (default): renders a styled anchor with the hash truncated.
   * - `"button"`: renders a small pill button labelled "View on Explorer".
   */
  variant?: "link" | "button";
  /** Additional CSS classes applied to the root element. */
  className?: string;
}

// ─── Helpers ─────────────────────────────────────────────────────────────────

/**
 * Resolves the Stellar Expert explorer URL for a transaction hash.
 *
 * @param txHash   - 64-character hex transaction hash.
 * @param network  - "testnet" or "mainnet".
 * @returns Full URL, e.g. `https://stellar.expert/explorer/testnet/tx/<hash>`.
 */
export function getStellarExplorerUrl(
  txHash: string,
  network: StellarNetwork,
): string {
  const net = network === "mainnet" ? "public" : "testnet";
  return `https://stellar.expert/explorer/${net}/tx/${txHash}`;
}

/**
 * Reads the active network from `NEXT_PUBLIC_STELLAR_NETWORK`.
 * Defaults to "testnet" so contributors never accidentally link to mainnet.
 */
function resolveNetwork(override?: StellarNetwork): StellarNetwork {
  if (override) return override;
  const env = process.env.NEXT_PUBLIC_STELLAR_NETWORK ?? "TESTNET";
  return env.toUpperCase() === "MAINNET" ? "mainnet" : "testnet";
}

/** Truncates a long hash to a readable format, e.g. `a1b2c3…f9e8d7`. */
function truncateHash(hash: string): string {
  if (hash.length <= 14) return hash;
  return `${hash.slice(0, 6)}…${hash.slice(-6)}`;
}

// ─── Copy-to-clipboard icon ───────────────────────────────────────────────────

function CopyIcon({ copied }: { copied: boolean }) {
  return copied ? (
    // Checkmark
    <svg
      xmlns="http://www.w3.org/2000/svg"
      width={14}
      height={14}
      viewBox="0 0 24 24"
      fill="none"
      stroke="currentColor"
      strokeWidth={2.5}
      strokeLinecap="round"
      strokeLinejoin="round"
      aria-hidden="true"
    >
      <polyline points="20 6 9 17 4 12" />
    </svg>
  ) : (
    // Clipboard
    <svg
      xmlns="http://www.w3.org/2000/svg"
      width={14}
      height={14}
      viewBox="0 0 24 24"
      fill="none"
      stroke="currentColor"
      strokeWidth={2}
      strokeLinecap="round"
      strokeLinejoin="round"
      aria-hidden="true"
    >
      <rect x="9" y="2" width="6" height="4" rx="1" />
      <path d="M8 4H6a2 2 0 0 0-2 2v14a2 2 0 0 0 2 2h12a2 2 0 0 0 2-2V6a2 2 0 0 0-2-2h-2" />
    </svg>
  );
}

// ─── External link icon ───────────────────────────────────────────────────────

function ExternalLinkIcon() {
  return (
    <svg
      xmlns="http://www.w3.org/2000/svg"
      width={12}
      height={12}
      viewBox="0 0 24 24"
      fill="none"
      stroke="currentColor"
      strokeWidth={2}
      strokeLinecap="round"
      strokeLinejoin="round"
      aria-hidden="true"
      className="inline-block"
    >
      <path d="M18 13v6a2 2 0 0 1-2 2H5a2 2 0 0 1-2-2V8a2 2 0 0 1 2-2h6" />
      <polyline points="15 3 21 3 21 9" />
      <line x1="10" y1="14" x2="21" y2="3" />
    </svg>
  );
}

// ─── Component ────────────────────────────────────────────────────────────────

/**
 * Renders a Stellar Expert explorer link for a transaction hash, with an
 * optional copy-to-clipboard button.
 *
 * - Generates the correct explorer URL for both Testnet and Mainnet.
 * - Opens in a new tab with `rel="noopener noreferrer"`.
 * - Copy button shows a "Copied!" toast on success.
 * - Fully keyboard-accessible.
 *
 * @example
 * ```tsx
 * <StellarExplorerLink txHash={data.txHash} />
 * <StellarExplorerLink txHash={data.txHash} variant="button" network="mainnet" />
 * ```
 */
export function StellarExplorerLink({
  txHash,
  network,
  variant = "link",
  className = "",
}: StellarExplorerLinkProps) {
  const [copied, setCopied] = useState(false);
  const resolvedNetwork = resolveNetwork(network);
  const url = getStellarExplorerUrl(txHash, resolvedNetwork);

  const handleCopy = useCallback(async () => {
    try {
      await navigator.clipboard.writeText(txHash);
      setCopied(true);
      toast.success("Transaction hash copied!", { duration: 2000 });
      setTimeout(() => setCopied(false), 2000);
    } catch {
      toast.error("Failed to copy to clipboard.");
    }
  }, [txHash]);

  const networkLabel =
    resolvedNetwork === "testnet" ? "Testnet" : "Mainnet";

  if (variant === "button") {
    return (
      <div className={`inline-flex items-center gap-2 ${className}`}>
        <a
          href={url}
          target="_blank"
          rel="noopener noreferrer"
          className="inline-flex items-center gap-1.5 px-4 py-2 rounded-full bg-black/5 hover:bg-black/10 transition-colors text-sm font-semibold text-black/80 border border-black/10 focus:outline-none focus-visible:ring-2 focus-visible:ring-accent"
          aria-label={`View transaction on Stellar Expert (${networkLabel}) — opens in new tab`}
        >
          View on Explorer
          <ExternalLinkIcon />
        </a>

        <button
          type="button"
          onClick={handleCopy}
          className="w-8 h-8 rounded-full flex items-center justify-center bg-black/5 hover:bg-black/10 transition-colors border border-black/10 focus:outline-none focus-visible:ring-2 focus-visible:ring-accent"
          aria-label={copied ? "Copied!" : "Copy transaction hash"}
          title={copied ? "Copied!" : "Copy transaction hash"}
        >
          <CopyIcon copied={copied} />
        </button>
      </div>
    );
  }

  // Default "link" variant
  return (
    <span className={`inline-flex items-center gap-2 ${className}`}>
      <a
        href={url}
        target="_blank"
        rel="noopener noreferrer"
        className="inline-flex items-center gap-1 font-mono text-sm text-accent hover:underline focus:outline-none focus-visible:ring-2 focus-visible:ring-accent rounded"
        aria-label={`Transaction ${txHash} on Stellar Expert (${networkLabel}) — opens in new tab`}
      >
        {truncateHash(txHash)}
        <ExternalLinkIcon />
      </a>

      <button
        type="button"
        onClick={handleCopy}
        className="inline-flex items-center justify-center w-6 h-6 rounded text-black/40 hover:text-black/80 hover:bg-black/5 transition-colors focus:outline-none focus-visible:ring-2 focus-visible:ring-accent"
        aria-label={copied ? "Copied!" : "Copy transaction hash"}
        title={copied ? "Copied!" : "Copy transaction hash"}
      >
        <CopyIcon copied={copied} />
      </button>
    </span>
  );
}
