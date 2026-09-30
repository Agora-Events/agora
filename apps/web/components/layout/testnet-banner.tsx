"use client";

/**
 * TestnetBanner — Issue #1491
 *
 * Displays a persistent warning strip informing users that Agora is running
 * against Stellar Testnet.  Renders only when
 * `NEXT_PUBLIC_STELLAR_NETWORK === "TESTNET"` (checked at runtime so the
 * component can be mounted unconditionally and tree-shaken in production).
 *
 * Acceptance criteria:
 * - Banner appears when NEXT_PUBLIC_STELLAR_NETWORK is "TESTNET".
 * - Dismissing the banner persists for the session (sessionStorage).
 * - Banner does not reappear on navigation within the same session.
 */

import { useEffect, useState } from "react";
import Link from "next/link";

const SESSION_KEY = "agora:testnet-banner-dismissed";

/**
 * Full-width testnet warning banner.
 *
 * Mount once inside the root layout (or any persistent shell) — it handles
 * its own visibility logic internally.
 */
export function TestnetBanner() {
  const [visible, setVisible] = useState(false);

  // Only activate on testnet builds; read dismissal state from sessionStorage.
  useEffect(() => {
    if (process.env.NEXT_PUBLIC_STELLAR_NETWORK !== "TESTNET") return;
    const dismissed = sessionStorage.getItem(SESSION_KEY) === "true";
    if (!dismissed) setVisible(true);
  }, []);

  const dismiss = () => {
    sessionStorage.setItem(SESSION_KEY, "true");
    setVisible(false);
  };

  if (!visible) return null;

  return (
    <div
      role="alert"
      aria-live="polite"
      className="w-full bg-amber-400 text-amber-950 text-sm font-medium px-4 py-2 flex items-center justify-between gap-4 z-50"
    >
      <div className="flex items-center gap-2 flex-wrap">
        {/* Testnet badge */}
        <span className="inline-flex items-center gap-1 bg-amber-600 text-white text-xs font-bold px-2 py-0.5 rounded-full uppercase tracking-wide">
          <svg
            xmlns="http://www.w3.org/2000/svg"
            viewBox="0 0 24 24"
            fill="currentColor"
            className="w-3 h-3"
            aria-hidden="true"
          >
            <path
              fillRule="evenodd"
              d="M9.401 3.003c1.155-2 4.043-2 5.197 0l7.355 12.748c1.154 2-.29 4.5-2.599 4.5H4.645c-2.309 0-3.752-2.5-2.598-4.5L9.4 3.003ZM12 8.25a.75.75 0 0 1 .75.75v3.75a.75.75 0 0 1-1.5 0V9a.75.75 0 0 1 .75-.75Zm0 8.25a.75.75 0 1 0 0-1.5.75.75 0 0 0 0 1.5Z"
              clipRule="evenodd"
            />
          </svg>
          Testnet (Soroban)
        </span>

        <span>
          You are using Agora on <strong>Stellar Testnet</strong>. Assets and
          transactions here have no real-world value.
        </span>

        <span className="flex items-center gap-3">
          <Link
            href="/wallet#faucet"
            className="underline underline-offset-2 hover:text-amber-800 transition-colors"
          >
            Get testnet funds →
          </Link>
          <a
            href="https://developers.stellar.org/docs/learn/fundamentals/networks"
            target="_blank"
            rel="noopener noreferrer"
            className="underline underline-offset-2 hover:text-amber-800 transition-colors"
          >
            Docs
          </a>
        </span>
      </div>

      <button
        type="button"
        onClick={dismiss}
        aria-label="Dismiss testnet banner"
        className="flex-shrink-0 rounded p-1 hover:bg-amber-500 transition-colors focus:outline-none focus-visible:ring-2 focus-visible:ring-amber-700"
      >
        <svg
          xmlns="http://www.w3.org/2000/svg"
          viewBox="0 0 20 20"
          fill="currentColor"
          className="w-4 h-4"
          aria-hidden="true"
        >
          <path d="M6.28 5.22a.75.75 0 0 0-1.06 1.06L8.94 10l-3.72 3.72a.75.75 0 1 0 1.06 1.06L10 11.06l3.72 3.72a.75.75 0 1 0 1.06-1.06L11.06 10l3.72-3.72a.75.75 0 0 0-1.06-1.06L10 8.94 6.28 5.22Z" />
        </svg>
      </button>
    </div>
  );
}
