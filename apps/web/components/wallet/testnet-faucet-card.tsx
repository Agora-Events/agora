"use client";

/**
 * TestnetFaucetCard — Issue #1492
 *
 * Lets developers and beta testers fund a Stellar Testnet keypair with XLM
 * via the Friendbot faucet and automatically establishes a USDC trustline,
 * all without leaving the Agora interface.
 *
 * Acceptance criteria:
 * - Clicking "Fund Wallet" calls https://friendbot.stellar.org?addr={publicKey}.
 * - USDC trustline is established automatically in the same flow.
 * - Component is invisible in production (Mainnet) builds — callers guard
 *   with `NEXT_PUBLIC_STELLAR_NETWORK === "TESTNET"`, but the component also
 *   short-circuits itself as a safety net.
 * - Shows a loading state and success toast with the transaction hash.
 */

import { useState } from "react";
import { toast } from "sonner";
import {
  isConnected,
  requestAccess,
  signTransaction,
} from "@stellar/freighter-api";
import {
  Asset,
  Keypair,
  Networks,
  Operation,
  TransactionBuilder,
} from "@stellar/stellar-sdk";

// ---------------------------------------------------------------------------
// Constants
// ---------------------------------------------------------------------------

const FRIENDBOT_URL = "https://friendbot.stellar.org";
const HORIZON_URL =
  process.env.NEXT_PUBLIC_STELLAR_HORIZON_URL ??
  "https://horizon-testnet.stellar.org";

/**
 * Testnet USDC issuing account (Circle's testnet USDC).
 * Override via NEXT_PUBLIC_USDC_ISSUER if your deployment uses a different one.
 */
const USDC_ISSUER =
  process.env.NEXT_PUBLIC_USDC_ISSUER ??
  "GBBD47IF6LWK7P7MDEVSCWR7DPUWV3NY3DTQEVFL4NAT4AQH3ZLLFLA5";

const USDC_ASSET = new Asset("USDC", USDC_ISSUER);

// ---------------------------------------------------------------------------
// Helpers
// ---------------------------------------------------------------------------

async function fundWithFriendbot(publicKey: string): Promise<string> {
  const url = `${FRIENDBOT_URL}?addr=${encodeURIComponent(publicKey)}`;
  const res = await fetch(url);
  if (!res.ok) {
    const text = await res.text().catch(() => res.statusText);
    throw new Error(`Friendbot failed (${res.status}): ${text}`);
  }
  const data = await res.json();
  // Horizon wraps the hash inside `hash` at the top level.
  return (data?.hash as string) ?? "unknown";
}

async function buildChangeTrustXdr(publicKey: string): Promise<string> {
  const accountRes = await fetch(
    `${HORIZON_URL}/accounts/${encodeURIComponent(publicKey)}`,
  );
  if (!accountRes.ok) {
    throw new Error("Could not load account from Horizon.");
  }
  const account = await accountRes.json();

  // Minimal stub compatible with TransactionBuilder — we only need id/sequence.
  const accountStub = {
    accountId: () => publicKey,
    sequenceNumber: () => account.sequence as string,
    incrementSequenceNumber() {
      // TransactionBuilder calls this; we provide a no-op since Horizon
      // returns the current sequence and we build only one tx.
    },
  };

  const tx = new TransactionBuilder(accountStub as any, {
    fee: "100",
    networkPassphrase: Networks.TESTNET,
  })
    .addOperation(Operation.changeTrust({ asset: USDC_ASSET }))
    .setTimeout(30)
    .build();

  return tx.toXDR();
}

// ---------------------------------------------------------------------------
// Component
// ---------------------------------------------------------------------------

/**
 * Faucet card for the `/wallet` page.
 *
 * Only renders when `NEXT_PUBLIC_STELLAR_NETWORK === "TESTNET"`.
 * Pass `publicKey` from `useAuth().walletAddress`.
 */
export function TestnetFaucetCard({ publicKey }: { publicKey: string }) {
  // Safety net — never render in production even if mounted incorrectly.
  if (process.env.NEXT_PUBLIC_STELLAR_NETWORK !== "TESTNET") return null;

  return <FaucetCardInner publicKey={publicKey} />;
}

function FaucetCardInner({ publicKey }: { publicKey: string }) {
  const [step, setStep] = useState<
    "idle" | "funding" | "trustline" | "success" | "error"
  >("idle");
  const [xlmHash, setXlmHash] = useState<string | null>(null);
  const [trustHash, setTrustHash] = useState<string | null>(null);
  const [errorMsg, setErrorMsg] = useState<string | null>(null);

  const isLoading = step === "funding" || step === "trustline";

  const handleFund = async () => {
    setStep("funding");
    setErrorMsg(null);
    setXlmHash(null);
    setTrustHash(null);

    try {
      // ── Step 1: Fund via Friendbot ──────────────────────────────────────
      const hash = await fundWithFriendbot(publicKey);
      setXlmHash(hash);
      toast.success(`Funded with 10,000 XLM! Tx: ${hash.slice(0, 12)}…`);

      // ── Step 2: Establish USDC trustline via Freighter ──────────────────
      setStep("trustline");

      let walletConnected = false;
      try {
        const connResult = await isConnected();
        walletConnected =
          typeof connResult === "boolean"
            ? connResult
            : Boolean(
                (connResult as { isConnected?: boolean })?.isConnected,
              );
      } catch {
        walletConnected = false;
      }

      if (walletConnected) {
        // Ensure we have permission to read the address.
        const accessResult = await requestAccess();
        const connectedKey =
          typeof accessResult === "string"
            ? accessResult
            : (accessResult as { address?: string })?.address ?? publicKey;

        const xdr = await buildChangeTrustXdr(connectedKey);

        const signed = await signTransaction(xdr, {
          networkPassphrase: Networks.TESTNET,
        });

        // Submit the signed XDR to Horizon.
        const submitRes = await fetch(`${HORIZON_URL}/transactions`, {
          method: "POST",
          headers: { "Content-Type": "application/x-www-form-urlencoded" },
          body: new URLSearchParams({ tx: signed as string }),
        });

        if (!submitRes.ok) {
          const errBody = await submitRes.json().catch(() => ({}));
          // "op_already_exists" means the trustline is already established — not a real error.
          const extras = errBody?.extras?.result_codes?.operations ?? [];
          if (!extras.includes("op_already_exists")) {
            throw new Error(
              errBody?.title ?? `Trustline submission failed (${submitRes.status})`,
            );
          }
        }

        const submitData = await submitRes.json().catch(() => ({}));
        const tHash = (submitData?.hash as string) ?? null;
        setTrustHash(tHash);
        toast.success("USDC trustline established!");
      } else {
        // Freighter not available — skip trustline silently; user can retry.
        toast.info(
          "Freighter not detected — skipped USDC trustline. Install Freighter to complete setup.",
        );
      }

      setStep("success");
    } catch (err) {
      const msg = err instanceof Error ? err.message : "Unknown error.";
      setErrorMsg(msg);
      setStep("error");
      toast.error(msg);
    }
  };

  return (
    <section
      id="faucet"
      aria-labelledby="faucet-heading"
      className="bg-white rounded-2xl border border-amber-200 shadow-sm p-6 space-y-4"
    >
      {/* Header */}
      <div className="flex items-start gap-3">
        <span className="inline-flex items-center justify-center w-10 h-10 rounded-full bg-amber-100 flex-shrink-0">
          {/* Faucet icon */}
          <svg
            xmlns="http://www.w3.org/2000/svg"
            viewBox="0 0 24 24"
            fill="none"
            stroke="currentColor"
            strokeWidth={1.8}
            strokeLinecap="round"
            strokeLinejoin="round"
            className="w-5 h-5 text-amber-700"
            aria-hidden="true"
          >
            <path d="M5 8h14M5 8a2 2 0 1 0-4 0 2 2 0 0 0 4 0Zm14 0a2 2 0 1 0 4 0 2 2 0 0 0-4 0ZM5 8v10a2 2 0 0 0 2 2h10a2 2 0 0 0 2-2V8M10 12v4m4-4v4" />
          </svg>
        </span>
        <div>
          <h2
            id="faucet-heading"
            className="text-base font-semibold text-ink-soft"
          >
            Fund Wallet with Testnet XLM
          </h2>
          <p className="text-sm text-muted-text mt-0.5">
            Receive 10,000 test XLM from the Stellar Friendbot and automatically
            set up a USDC trustline — no external tabs needed.
          </p>
        </div>
      </div>

      {/* Status badges */}
      {(step === "success" || step === "funding" || step === "trustline") && (
        <ul className="space-y-2 text-sm">
          <li className="flex items-center gap-2">
            <StepIndicator done={!!xlmHash} active={step === "funding"} />
            <span className="text-ink-soft">
              {xlmHash
                ? `XLM funded — tx ${xlmHash.slice(0, 16)}…`
                : "Requesting XLM from Friendbot…"}
            </span>
          </li>
          <li className="flex items-center gap-2">
            <StepIndicator
              done={!!trustHash}
              active={step === "trustline"}
            />
            <span className="text-ink-soft">
              {trustHash
                ? `USDC trustline set — tx ${trustHash.slice(0, 16)}…`
                : "Establishing USDC trustline…"}
            </span>
          </li>
        </ul>
      )}

      {/* Error */}
      {step === "error" && errorMsg && (
        <p
          role="alert"
          className="text-sm text-red-600 bg-red-50 border border-red-200 rounded-lg px-4 py-3"
        >
          {errorMsg}
        </p>
      )}

      {/* CTA */}
      <button
        type="button"
        onClick={handleFund}
        disabled={isLoading || step === "success"}
        aria-busy={isLoading}
        className="inline-flex items-center gap-2 bg-amber-400 hover:bg-amber-500 disabled:opacity-60 disabled:cursor-not-allowed text-amber-950 text-sm font-semibold px-5 py-2.5 rounded-full transition-colors focus:outline-none focus-visible:ring-2 focus-visible:ring-amber-600"
      >
        {isLoading ? (
          <>
            <Spinner />
            {step === "funding" ? "Funding XLM…" : "Setting USDC trustline…"}
          </>
        ) : step === "success" ? (
          "✓ Wallet funded"
        ) : (
          "Fund Wallet with Testnet XLM"
        )}
      </button>
    </section>
  );
}

// ---------------------------------------------------------------------------
// Sub-components
// ---------------------------------------------------------------------------

function StepIndicator({
  done,
  active,
}: {
  done: boolean;
  active: boolean;
}) {
  if (done)
    return (
      <span className="w-4 h-4 text-green-500" aria-label="Done">
        ✓
      </span>
    );
  if (active) return <Spinner className="w-4 h-4 text-amber-600" />;
  return (
    <span className="w-4 h-4 rounded-full border-2 border-border-warm inline-block" />
  );
}

function Spinner({ className = "w-4 h-4" }: { className?: string }) {
  return (
    <svg
      className={`${className} animate-spin`}
      xmlns="http://www.w3.org/2000/svg"
      fill="none"
      viewBox="0 0 24 24"
      aria-hidden="true"
    >
      <circle
        className="opacity-25"
        cx="12"
        cy="12"
        r="10"
        stroke="currentColor"
        strokeWidth="4"
      />
      <path
        className="opacity-75"
        fill="currentColor"
        d="M4 12a8 8 0 018-8v4a4 4 0 00-4 4H4z"
      />
    </svg>
  );
}
