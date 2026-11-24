"use client";

/**
 * NetworkProvider / useNetwork — Issue #1541
 *
 * App-wide Testnet/Mainnet selector surfaced via a dropdown in the header
 * (see `@/components/layout/network-switcher`). This is distinct from
 * `useStellarNetwork` (hooks/useStellarNetwork.ts), which reports whichever
 * network the user's connected Freighter wallet happens to be on — this
 * context instead controls which network *Agora itself* targets for RPC
 * calls, independent of the wallet.
 *
 * Switching network here:
 * - Persists the choice to localStorage so it survives reloads.
 * - Exposes the matching RPC/Horizon endpoints + passphrase via `endpoints`.
 * - Resets the shared Soroban RPC server singleton (see
 *   `lib/stellar/client.ts`) so the next call builds a fresh client instead
 *   of reusing one pinned to the previous network.
 * - Clears any locally cached blockchain reads (ticket availability, etc.)
 *   so stale data from the old network isn't shown.
 * - Dispatches a `agora:network-changed` CustomEvent so any mounted
 *   component/hook can react (e.g. re-poll ticket availability).
 */

import {
  createContext,
  useCallback,
  useContext,
  useEffect,
  useMemo,
  useState,
  type ReactNode,
} from "react";
import { Networks } from "@stellar/stellar-sdk";
import { resetSorobanRpcServer } from "./client";

export type StellarNetworkId = "TESTNET" | "MAINNET";

export interface NetworkEndpoints {
  rpcUrl: string;
  horizonUrl: string;
  networkPassphrase: string;
}

/**
 * Known endpoints per network. These mirror the same fallback URLs already
 * used in `lib/stellar/config.ts` / `lib/stellar/client.ts` so switching
 * here stays consistent with the rest of the app's defaults.
 */
export const NETWORK_ENDPOINTS: Record<StellarNetworkId, NetworkEndpoints> = {
  TESTNET: {
    rpcUrl: "https://soroban-testnet.stellar.org",
    horizonUrl: "https://horizon-testnet.stellar.org",
    networkPassphrase: Networks.TESTNET,
  },
  MAINNET: {
    rpcUrl: "https://soroban-rpc.mainnet.stellar.org",
    horizonUrl: "https://horizon.stellar.org",
    networkPassphrase: Networks.PUBLIC,
  },
};

const STORAGE_KEY = "agora:active-network";

/** Fired on `window` whenever the active network changes. */
export const NETWORK_CHANGE_EVENT = "agora:network-changed";

/** Prefixes of localStorage/sessionStorage keys that cache on-chain reads. */
const CHAIN_CACHE_KEY_PREFIXES = [
  "agora:chain-cache:",
  "agora:ticket-availability:",
];

function resolveDefaultNetwork(): StellarNetworkId {
  const envNetwork = (process.env.NEXT_PUBLIC_STELLAR_NETWORK || "TESTNET").toUpperCase();
  return envNetwork === "MAINNET" ? "MAINNET" : "TESTNET";
}

function clearCachedBlockchainData() {
  if (typeof window === "undefined") return;

  for (const storage of [window.localStorage, window.sessionStorage]) {
    try {
      const staleKeys: string[] = [];
      for (let i = 0; i < storage.length; i++) {
        const key = storage.key(i);
        if (key && CHAIN_CACHE_KEY_PREFIXES.some((prefix) => key.startsWith(prefix))) {
          staleKeys.push(key);
        }
      }
      staleKeys.forEach((key) => storage.removeItem(key));
    } catch {
      // Storage may be unavailable (private browsing, etc.) — safe to skip.
    }
  }
}

interface NetworkContextValue {
  /** The currently active network. */
  network: StellarNetworkId;
  /** RPC/Horizon endpoints + passphrase for the active network. */
  endpoints: NetworkEndpoints;
  /** True when the active network is Testnet. */
  isTestnet: boolean;
  /** Switch the active network, updating endpoints and clearing caches. */
  setNetwork: (next: StellarNetworkId) => void;
}

const NetworkContext = createContext<NetworkContextValue | null>(null);

export function NetworkProvider({ children }: { children: ReactNode }) {
  const [network, setNetworkState] = useState<StellarNetworkId>(resolveDefaultNetwork);

  // Hydrate any previously persisted choice after mount (avoids SSR/client
  // markup mismatches, since localStorage isn't available on the server).
  useEffect(() => {
    try {
      const stored = window.localStorage.getItem(STORAGE_KEY);
      if (stored === "TESTNET" || stored === "MAINNET") {
        setNetworkState(stored);
      }
    } catch {
      // ignore — fall back to the env-derived default.
    }
  }, []);

  const setNetwork = useCallback((next: StellarNetworkId) => {
    setNetworkState((current) => {
      if (current === next) return current;

      try {
        window.localStorage.setItem(STORAGE_KEY, next);
      } catch {
        // ignore — selection just won't persist across reloads.
      }

      // Rebuild the shared Soroban RPC client against the new network on
      // next use, and drop any blockchain reads cached from the old one.
      resetSorobanRpcServer();
      clearCachedBlockchainData();

      if (typeof window !== "undefined") {
        window.dispatchEvent(
          new CustomEvent(NETWORK_CHANGE_EVENT, { detail: { network: next } }),
        );
      }

      return next;
    });
  }, []);

  const value = useMemo<NetworkContextValue>(
    () => ({
      network,
      endpoints: NETWORK_ENDPOINTS[network],
      isTestnet: network === "TESTNET",
      setNetwork,
    }),
    [network, setNetwork],
  );

  return <NetworkContext.Provider value={value}>{children}</NetworkContext.Provider>;
}

export function useNetwork(): NetworkContextValue {
  const ctx = useContext(NetworkContext);
  if (!ctx) {
    throw new Error("useNetwork must be used within a NetworkProvider");
  }
  return ctx;
}
