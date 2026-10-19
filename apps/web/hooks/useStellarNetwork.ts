"use client";

/**
 * useStellarNetwork — Issue #1490
 *
 * Queries the active Freighter wallet for its configured network details and
 * exposes reactive state so any component can gate actions on the correct
 * Stellar network.
 *
 * Acceptance criteria:
 * - isCorrectNetwork is true exclusively when connected to Stellar Testnet.
 * - Switching network in Freighter triggers a reactive state update without
 *   a page reload (polled every POLL_INTERVAL_MS).
 * - requestNetworkSwitch() surfaces a user-readable message when the wallet
 *   is on the wrong network.
 */

import { useCallback, useEffect, useRef, useState } from "react";
import { getNetworkDetails } from "@stellar/freighter-api";

// ---------------------------------------------------------------------------
// Constants
// ---------------------------------------------------------------------------

/** The canonical Stellar Testnet network passphrase. */
export const TESTNET_PASSPHRASE = "Test SDF Network ; September 2015";

/** How often (ms) to re-query Freighter for network changes. */
const POLL_INTERVAL_MS = 3_000;

// ---------------------------------------------------------------------------
// Types
// ---------------------------------------------------------------------------

export interface StellarNetworkState {
  /** The passphrase of the wallet's currently selected network, or null while loading. */
  currentNetwork: string | null;
  /** True when the wallet is on Stellar Testnet. */
  isCorrectNetwork: boolean;
  /** True while the first network query is in-flight. */
  isLoading: boolean;
  /** Non-null when the last query encountered an error (e.g. Freighter not installed). */
  error: string | null;
  /**
   * Prompts the user to switch to Testnet.  Since Freighter does not expose a
   * programmatic network-switch API, this surfaces a toast-friendly message
   * explaining the required action.
   */
  requestNetworkSwitch: () => void;
}

// ---------------------------------------------------------------------------
// Hook
// ---------------------------------------------------------------------------

/**
 * Reactive Stellar network state sourced from Freighter.
 *
 * @example
 * const { isCorrectNetwork, currentNetwork, requestNetworkSwitch } = useStellarNetwork();
 * if (!isCorrectNetwork) requestNetworkSwitch();
 */
export function useStellarNetwork(): StellarNetworkState {
  const [currentNetwork, setCurrentNetwork] = useState<string | null>(null);
  const [isLoading, setIsLoading] = useState(true);
  const [error, setError] = useState<string | null>(null);

  const pollRef = useRef<ReturnType<typeof setInterval> | null>(null);

  const fetchNetwork = useCallback(async () => {
    try {
      const details = await getNetworkDetails();
      // getNetworkDetails returns an error field when Freighter is not
      // installed or the extension denied the request.
      if (details.error) {
        setError(String(details.error));
        setCurrentNetwork(null);
      } else {
        setCurrentNetwork(details.networkPassphrase ?? null);
        setError(null);
      }
    } catch (err) {
      setError(
        err instanceof Error
          ? err.message
          : "Could not reach Freighter wallet.",
      );
      setCurrentNetwork(null);
    } finally {
      setIsLoading(false);
    }
  }, []);

  // Initial fetch + polling for reactive updates without a page reload.
  useEffect(() => {
    fetchNetwork();
    pollRef.current = setInterval(fetchNetwork, POLL_INTERVAL_MS);
    return () => {
      if (pollRef.current !== null) clearInterval(pollRef.current);
    };
  }, [fetchNetwork]);

  const isCorrectNetwork = currentNetwork === TESTNET_PASSPHRASE;

  const requestNetworkSwitch = useCallback(() => {
    // Freighter does not expose a programmatic API to switch networks; the
    // best we can do is show the user a clear instruction.
    if (typeof window !== "undefined") {
      // Dispatch a custom event so any mounted banner/toast can react.
      window.dispatchEvent(
        new CustomEvent("stellar:wrong-network", {
          detail: {
            currentNetwork,
            requiredNetwork: TESTNET_PASSPHRASE,
            message:
              "Please open Freighter and switch to the Stellar Testnet " +
              '("Test SDF Network ; September 2015") before continuing.',
          },
        }),
      );
    }
  }, [currentNetwork]);

  return {
    currentNetwork,
    isCorrectNetwork,
    isLoading,
    error,
    requestNetworkSwitch,
  };
}
