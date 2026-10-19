"use client";

import { toast } from "sonner";
import type { MouseEvent } from "react";

type WalletAddressProps = {
  address: string;
  className?: string;
};

function formatWalletAddress(address: string): string {
  if (address.length < 12) {
    return address;
  }

  return `${address.slice(0, 4)}…${address.slice(-4)}`;
}

export function WalletAddress({ address, className = "" }: WalletAddressProps) {
  const display = formatWalletAddress(address);

  const handleCopy = async (event: MouseEvent<HTMLButtonElement>) => {
    event.preventDefault();
    event.stopPropagation();

    try {
      await navigator.clipboard.writeText(address);
      toast.success("Copied");
    } catch {
      toast.error("Could not copy address");
    }
  };

  return (
    <span
      className={`inline-flex min-w-0 max-w-full items-center gap-1.5 ${className}`}
    >
      <span
        title={address}
        aria-label={address}
        className="font-mono text-sm tracking-wide whitespace-nowrap"
      >
        {display}
      </span>
      <button
        type="button"
        onClick={handleCopy}
        aria-label={`Copy wallet address ${address}`}
        className="inline-flex h-7 w-7 shrink-0 items-center justify-center rounded-full border border-current/30 transition-colors hover:bg-current/10 focus-visible:outline-2 focus-visible:outline-offset-2 focus-visible:outline-[#FDDA23]"
      >
        <svg
          viewBox="0 0 24 24"
          fill="none"
          stroke="currentColor"
          strokeWidth={1.8}
          className="h-3.5 w-3.5"
          aria-hidden="true"
        >
          <rect x="8" y="8" width="12" height="12" rx="2" />
          <path d="M16 8V6a2 2 0 0 0-2-2H6a2 2 0 0 0-2 2v8a2 2 0 0 0 2 2h2" />
        </svg>
      </button>
    </span>
  );
}
