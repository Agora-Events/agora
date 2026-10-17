"use client";

/**
 * NetworkSwitcher — Issue #1567
 *
 * Owns the active Soroban RPC network for the app and exposes it through a
 * small React context so any descendant can read the active RPC URL or switch
 * networks. `NetworkSwitcher` renders the toggle control itself.
 */

import { createContext, useCallback, useContext, useMemo, useState } from "react";

export type RpcNetwork = "testnet" | "mainnet";

export const RPC_URLS: Record<RpcNetwork, string> = {
  testnet: "https://soroban-testnet.stellar.org",
  mainnet: "https://soroban-rpc.mainnet.stellar.org",
};

export interface NetworkSwitcherContextValue {
  /** Currently selected network. */
  activeNetwork: RpcNetwork;
  /** RPC endpoint for `activeNetwork`. */
  activeRpcUrl: string;
  /** Explicitly select a network. */
  switchNetwork: (network: RpcNetwork) => void;
  /** Flip between testnet and mainnet. */
  toggleNetwork: () => void;
}

const NetworkSwitcherContext = createContext<NetworkSwitcherContextValue | null>(null);

export function NetworkSwitcherProvider({
  children,
  defaultNetwork = "testnet",
}: {
  children: React.ReactNode;
  defaultNetwork?: RpcNetwork;
}) {
  const [activeNetwork, setActiveNetwork] = useState<RpcNetwork>(defaultNetwork);

  const switchNetwork = useCallback((network: RpcNetwork) => {
    setActiveNetwork(network);
  }, []);

  const toggleNetwork = useCallback(() => {
    setActiveNetwork((prev) => (prev === "testnet" ? "mainnet" : "testnet"));
  }, []);

  const value = useMemo<NetworkSwitcherContextValue>(
    () => ({
      activeNetwork,
      activeRpcUrl: RPC_URLS[activeNetwork],
      switchNetwork,
      toggleNetwork,
    }),
    [activeNetwork, switchNetwork, toggleNetwork],
  );

  return (
    <NetworkSwitcherContext.Provider value={value}>
      {children}
    </NetworkSwitcherContext.Provider>
  );
}

export function useNetworkSwitcher(): NetworkSwitcherContextValue {
  const context = useContext(NetworkSwitcherContext);
  if (!context) {
    throw new Error("useNetworkSwitcher must be used within a NetworkSwitcherProvider");
  }
  return context;
}

export function NetworkSwitcher() {
  const { activeNetwork, switchNetwork } = useNetworkSwitcher();

  return (
    <div role="group" aria-label="RPC network">
      {(["testnet", "mainnet"] as const).map((network) => (
        <button
          key={network}
          type="button"
          aria-pressed={activeNetwork === network}
          onClick={() => switchNetwork(network)}
          className="px-2 py-1 text-xs capitalize"
        >
          {network}
        </button>
      ))}
    </div>
  );
}

export default NetworkSwitcher;
