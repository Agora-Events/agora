"use client";

/**
 * NetworkSwitcher — Issue #1541
 *
 * Header dropdown that lets developers/QA flip Agora's active Stellar
 * network between Testnet and Mainnet. Delegates to `useNetwork()`
 * (lib/stellar/network-context.tsx), which handles persisting the choice,
 * updating the active RPC/Horizon endpoints, and clearing cached
 * blockchain data.
 *
 * A "TESTNET" badge is always shown on the trigger while testnet is
 * active, so users don't mistake it for mainnet with real funds.
 */

import { useEffect, useRef, useState } from "react";
import { ChevronDown } from "@/components/ui/icons";
import { useNetwork, type StellarNetworkId } from "@/lib/stellar/network-context";

const NETWORK_LABELS: Record<StellarNetworkId, string> = {
  TESTNET: "Testnet",
  MAINNET: "Mainnet",
};

const NETWORK_OPTIONS: StellarNetworkId[] = ["TESTNET", "MAINNET"];

export function NetworkSwitcher() {
  const { network, isTestnet, setNetwork } = useNetwork();
  const [isOpen, setIsOpen] = useState(false);
  const containerRef = useRef<HTMLDivElement>(null);

  // Close on outside click / Escape — mirrors the pattern already used by
  // the main mobile nav drawer in components/layout/navbar.tsx.
  useEffect(() => {
    if (!isOpen) return;

    const handleClick = (event: MouseEvent) => {
      if (
        containerRef.current &&
        !containerRef.current.contains(event.target as Node)
      ) {
        setIsOpen(false);
      }
    };
    const handleKeyDown = (event: KeyboardEvent) => {
      if (event.key === "Escape") setIsOpen(false);
    };

    document.addEventListener("mousedown", handleClick);
    document.addEventListener("keydown", handleKeyDown);
    return () => {
      document.removeEventListener("mousedown", handleClick);
      document.removeEventListener("keydown", handleKeyDown);
    };
  }, [isOpen]);

  const handleSelect = (next: StellarNetworkId) => {
    setNetwork(next);
    setIsOpen(false);
  };

  return (
    <div className="relative" ref={containerRef}>
      <button
        type="button"
        onClick={() => setIsOpen((prev) => !prev)}
        aria-haspopup="listbox"
        aria-expanded={isOpen}
        aria-label={`Active network: ${NETWORK_LABELS[network]}. Click to switch.`}
        className="flex items-center gap-1.5 h-[53px] px-3 rounded-full border border-black bg-white font-semibold text-sm text-black hover:-translate-x-[2px] hover:translate-y-[2px] hover:shadow-[-2px_2px_0px_0px_rgba(0,0,0,1)] transition-all"
      >
        <span
          className={`w-2 h-2 rounded-full ${isTestnet ? "bg-amber-500" : "bg-green-500"}`}
          aria-hidden="true"
        />
        <span className="hidden sm:inline">{NETWORK_LABELS[network]}</span>
        {isTestnet && (
          <span className="bg-amber-500 text-white text-[10px] font-bold px-1.5 py-0.5 rounded-full uppercase tracking-wide">
            Testnet
          </span>
        )}
        <ChevronDown
          size={16}
          className={`transition-transform ${isOpen ? "rotate-180" : ""}`}
        />
      </button>

      {isOpen && (
        <div
          role="listbox"
          aria-label="Select Stellar network"
          className="absolute right-0 mt-2 w-44 bg-white border border-black rounded-xl shadow-[-4px_4px_0px_0px_rgba(0,0,0,1)] overflow-hidden z-50"
        >
          {NETWORK_OPTIONS.map((option) => (
            <button
              key={option}
              type="button"
              role="option"
              aria-selected={network === option}
              onClick={() => handleSelect(option)}
              className={`w-full flex items-center justify-between gap-2 px-4 py-3 text-sm font-medium text-left text-black hover:bg-gray-50 transition-colors ${
                network === option ? "bg-gray-100" : ""
              }`}
            >
              {NETWORK_LABELS[option]}
              {option === "TESTNET" && (
                <span className="text-[10px] font-bold text-amber-600 uppercase">
                  Testnet
                </span>
              )}
            </button>
          ))}
        </div>
      )}
    </div>
  );
}
