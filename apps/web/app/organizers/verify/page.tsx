"use client";

import { useState, useEffect, useCallback } from "react";
import { Navbar } from "@/components/layout/navbar";
import { Footer } from "@/components/layout/footer";
import { Button } from "@/components/ui/button";
import { useAuth } from "@/hooks/useAuth";
import Image from "next/image";
import Link from "next/link";
import { toast } from "sonner";

// ─── Constants ────────────────────────────────────────────────────────────────

/** Minimum collateral required to become a verified organizer (in XLM). */
const MIN_STAKE_XLM = 100;

/** Unstaking lockup period in seconds (7 days). */
const UNSTAKE_LOCKUP_SECONDS = 7 * 24 * 60 * 60;

// ─── Types ────────────────────────────────────────────────────────────────────

interface StakeState {
  /** Amount currently staked, in XLM. */
  stakedAmount: number;
  /** Whether the organizer holds "Verified" status on-chain. */
  isVerified: boolean;
  /** Unix timestamp (ms) when the unstake lockup expires, or null if not withdrawing. */
  unstakeLockupEndsAt: number | null;
  /** Whether the organizer has initiated a withdrawal. */
  isPendingWithdrawal: boolean;
}

// ─── Helpers ─────────────────────────────────────────────────────────────────

/**
 * Formats a seconds-remaining countdown into "Xd Xh Xm Xs".
 */
function formatCountdown(totalSeconds: number): string {
  if (totalSeconds <= 0) return "0s";
  const d = Math.floor(totalSeconds / 86400);
  const h = Math.floor((totalSeconds % 86400) / 3600);
  const m = Math.floor((totalSeconds % 3600) / 60);
  const s = totalSeconds % 60;
  const parts: string[] = [];
  if (d > 0) parts.push(`${d}d`);
  if (h > 0) parts.push(`${h}h`);
  if (m > 0) parts.push(`${m}m`);
  if (s > 0 || parts.length === 0) parts.push(`${s}s`);
  return parts.join(" ");
}

/**
 * Builds a mock XDR transaction payload string.
 * In production this would call the Stellar SDK to build a real
 * InvokeHostFunction operation against the staking contract.
 */
function buildStakeXdr(walletAddress: string, amountXlm: number): string {
  const encoded = btoa(
    JSON.stringify({
      op: "invoke_host_function",
      contract: "CSTAKE_CONTRACT_ADDRESS_PLACEHOLDER",
      function: "stake",
      args: [walletAddress, (amountXlm * 10_000_000).toString()],
      network: "testnet",
    }),
  );
  return `AAAAAQAAAA${encoded.slice(0, 40)}...`;
}

/**
 * Builds a mock XDR payload for initiating an unstake.
 */
function buildUnstakeXdr(walletAddress: string): string {
  const encoded = btoa(
    JSON.stringify({
      op: "invoke_host_function",
      contract: "CSTAKE_CONTRACT_ADDRESS_PLACEHOLDER",
      function: "begin_unstake",
      args: [walletAddress],
      network: "testnet",
    }),
  );
  return `AAAAAQAAAA${encoded.slice(0, 40)}...`;
}

// ─── Sub-components ───────────────────────────────────────────────────────────

function StatCard({
  label,
  value,
  accent = false,
}: {
  label: string;
  value: React.ReactNode;
  accent?: boolean;
}) {
  return (
    <div
      className={`rounded-2xl border-2 border-black p-6 flex flex-col gap-2 shadow-[-4px_4px_0_rgba(0,0,0,1)] ${
        accent ? "bg-accent" : "bg-white"
      }`}
    >
      <span className="text-sm font-semibold text-black/60 uppercase tracking-wider">
        {label}
      </span>
      <span className="text-3xl font-extrabold text-ink-deep">{value}</span>
    </div>
  );
}

function VerifiedBadge({ verified }: { verified: boolean }) {
  return (
    <span
      className={`inline-flex items-center gap-2 px-4 py-1.5 rounded-full border-2 border-black text-sm font-bold shadow-[-2px_2px_0_rgba(0,0,0,1)] ${
        verified ? "bg-success-light text-black" : "bg-surface text-black/60"
      }`}
    >
      {verified ? (
        <>
          <span aria-hidden>✓</span> Verified Organizer
        </>
      ) : (
        <>
          <span aria-hidden>○</span> Unverified
        </>
      )}
    </span>
  );
}

/** Full-page loading skeleton */
function LoadingSkeleton() {
  return (
    <div className="flex-1 max-w-4xl w-full mx-auto px-4 lg:px-0 py-12 flex flex-col gap-8 animate-pulse">
      <div className="h-10 w-64 bg-black/10 rounded-xl" />
      <div className="grid grid-cols-1 sm:grid-cols-3 gap-6">
        {[1, 2, 3].map((i) => (
          <div key={i} className="h-28 rounded-2xl bg-black/10" />
        ))}
      </div>
      <div className="h-64 rounded-2xl bg-black/10" />
    </div>
  );
}

/** XDR payload modal */
function XdrModal({
  xdr,
  title,
  onClose,
}: {
  xdr: string;
  title: string;
  onClose: () => void;
}) {
  const copy = () => {
    navigator.clipboard.writeText(xdr).then(() => toast.success("XDR copied to clipboard"));
  };

  return (
    <div
      className="fixed inset-0 z-50 flex items-center justify-center bg-black/50 p-4"
      role="dialog"
      aria-modal="true"
      aria-labelledby="xdr-modal-title"
    >
      <div className="bg-base w-full max-w-2xl rounded-3xl border-2 border-black shadow-[-8px_8px_0_rgba(0,0,0,1)] p-8 flex flex-col gap-6">
        <div className="flex items-center justify-between">
          <h2 id="xdr-modal-title" className="text-2xl font-extrabold italic">
            {title}
          </h2>
          <button
            onClick={onClose}
            aria-label="Close modal"
            className="w-10 h-10 rounded-full border-2 border-black flex items-center justify-center hover:bg-surface transition-colors"
          >
            ✕
          </button>
        </div>

        <p className="text-sm text-black/70">
          Sign this XDR payload with your Stellar wallet (e.g. Freighter) and
          submit it to the network.
        </p>

        <textarea
          readOnly
          value={xdr}
          rows={5}
          aria-label="XDR transaction payload"
          className="w-full font-mono text-xs bg-ink text-accent rounded-xl border-2 border-black p-4 resize-none outline-none focus-visible:ring-2 focus-visible:ring-accent"
        />

        <div className="flex flex-col sm:flex-row gap-4">
          <Button
            onClick={copy}
            backgroundColor="bg-accent"
            textColor="text-black"
            shadowColor="rgba(0,0,0,1)"
            className="flex-1 font-bold"
          >
            Copy XDR
          </Button>
          <a
            href="https://laboratory.stellar.org/#txsigner"
            target="_blank"
            rel="noreferrer"
            className="flex-1"
          >
            <Button
              backgroundColor="bg-white"
              textColor="text-black"
              shadowColor="rgba(0,0,0,1)"
              className="w-full font-bold"
            >
              Open in Stellar Lab ↗
            </Button>
          </a>
          <Button
            onClick={onClose}
            backgroundColor="bg-surface"
            textColor="text-black"
            shadowColor="rgba(0,0,0,0.3)"
            className="font-bold"
          >
            Cancel
          </Button>
        </div>
      </div>
    </div>
  );
}

// ─── Countdown timer hook ─────────────────────────────────────────────────────

function useCountdown(endsAt: number | null) {
  const [remaining, setRemaining] = useState<number>(0);

  useEffect(() => {
    if (!endsAt) return;
    const tick = () => {
      const diff = Math.max(0, Math.floor((endsAt - Date.now()) / 1000));
      setRemaining(diff);
    };
    tick();
    const id = setInterval(tick, 1000);
    return () => clearInterval(id);
  }, [endsAt]);

  return remaining;
}

// ─── Stake amount input ───────────────────────────────────────────────────────

function StakeInput({
  value,
  onChange,
  min,
  error,
}: {
  value: string;
  onChange: (v: string) => void;
  min: number;
  error?: string;
}) {
  return (
    <div className="flex flex-col gap-2">
      <label
        htmlFor="stake-amount"
        className="text-sm font-semibold text-black"
      >
        Amount to stake (XLM)
      </label>
      <div
        className={`flex items-center gap-3 rounded-2xl border-2 px-5 py-4 bg-white shadow-[-2px_2px_0_rgba(0,0,0,1)] transition-colors ${
          error ? "border-error" : "border-black"
        }`}
      >
        <span className="text-lg font-bold text-black/40">XLM</span>
        <input
          id="stake-amount"
          type="number"
          min={min}
          step="1"
          value={value}
          onChange={(e) => onChange(e.target.value)}
          placeholder={`Min. ${min}`}
          aria-describedby={error ? "stake-amount-error" : undefined}
          className="flex-1 text-2xl font-extrabold text-ink-deep bg-transparent outline-none placeholder:text-black/20"
        />
      </div>
      {error && (
        <p id="stake-amount-error" role="alert" className="text-sm text-error font-semibold">
          {error}
        </p>
      )}
      <p className="text-xs text-black/50">
        Minimum collateral required: <strong>{min} XLM</strong>
      </p>
    </div>
  );
}

// ─── Main page ────────────────────────────────────────────────────────────────

export default function OrganizerVerifyPage() {
  const { user, walletAddress, isLoading: authLoading } = useAuth();

  // Simulated on-chain state — in production, fetch from a Soroban RPC call.
  const [stakeState, setStakeState] = useState<StakeState>({
    stakedAmount: 0,
    isVerified: false,
    unstakeLockupEndsAt: null,
    isPendingWithdrawal: false,
  });

  const [stakeInput, setStakeInput] = useState("");
  const [stakeInputError, setStakeInputError] = useState("");
  const [isFetchingState, setIsFetchingState] = useState(true);
  const [xdrModal, setXdrModal] = useState<{
    xdr: string;
    title: string;
  } | null>(null);
  const [isStaking, setIsStaking] = useState(false);
  const [isWithdrawing, setIsWithdrawing] = useState(false);

  const countdown = useCountdown(stakeState.unstakeLockupEndsAt);
  const lockupExpired =
    stakeState.isPendingWithdrawal && countdown === 0;

  // Simulate loading on-chain state
  useEffect(() => {
    if (authLoading) return;
    const timer = setTimeout(() => {
      // Mock: organizer has already staked 150 XLM and is verified
      setStakeState({
        stakedAmount: 150,
        isVerified: true,
        unstakeLockupEndsAt: null,
        isPendingWithdrawal: false,
      });
      setIsFetchingState(false);
    }, 800);
    return () => clearTimeout(timer);
  }, [authLoading]);

  const handleStake = useCallback(async () => {
    const amount = parseFloat(stakeInput);
    if (!stakeInput || isNaN(amount) || amount < MIN_STAKE_XLM) {
      setStakeInputError(`Minimum stake is ${MIN_STAKE_XLM} XLM`);
      return;
    }
    setStakeInputError("");

    const address = walletAddress ?? "GAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAWHF";
    setIsStaking(true);

    // Build the XDR payload (real impl would await Stellar SDK)
    const xdr = buildStakeXdr(address, amount);

    // Simulate network delay
    await new Promise((r) => setTimeout(r, 600));
    setIsStaking(false);

    setXdrModal({ xdr, title: "Stake Collateral — Sign Transaction" });
  }, [stakeInput, walletAddress]);

  const handleWithdraw = useCallback(async () => {
    if (stakeState.isPendingWithdrawal && !lockupExpired) return;

    const address = walletAddress ?? "GAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAWHF";
    setIsWithdrawing(true);

    await new Promise((r) => setTimeout(r, 600));
    setIsWithdrawing(false);

    if (!stakeState.isPendingWithdrawal) {
      // Initiate the lockup countdown
      const xdr = buildUnstakeXdr(address);
      setXdrModal({ xdr, title: "Initiate Withdrawal — Sign Transaction" });

      setStakeState((prev) => ({
        ...prev,
        isPendingWithdrawal: true,
        unstakeLockupEndsAt: Date.now() + UNSTAKE_LOCKUP_SECONDS * 1000,
        isVerified: false,
      }));
    } else {
      // Lockup has expired — complete the withdrawal
      const xdr = buildUnstakeXdr(address);
      setXdrModal({ xdr, title: "Complete Withdrawal — Sign Transaction" });
      setStakeState((prev) => ({
        ...prev,
        stakedAmount: 0,
        isPendingWithdrawal: false,
        unstakeLockupEndsAt: null,
        isVerified: false,
      }));
      toast.success("Withdrawal complete — collateral returned to your wallet");
    }
  }, [stakeState.isPendingWithdrawal, lockupExpired, walletAddress]);

  // ── Unauthenticated gate ───────────────────────────────────────────────────
  if (!authLoading && !user) {
    return (
      <main className="flex flex-col min-h-screen bg-base">
        <Navbar />
        <div className="flex-1 flex flex-col items-center justify-center gap-8 px-4 py-20">
          <div className="bg-white rounded-3xl border-2 border-black shadow-[-8px_8px_0_rgba(0,0,0,1)] p-12 flex flex-col items-center gap-6 max-w-md w-full text-center">
            <div className="w-20 h-20 rounded-full bg-surface border-2 border-black flex items-center justify-center text-4xl">
              🔒
            </div>
            <h1 className="text-3xl font-extrabold italic">Sign in required</h1>
            <p className="text-black/60">
              You must be signed in to access the organizer verification dashboard.
            </p>
            <Link href="/auth" className="w-full">
              <Button
                backgroundColor="bg-accent"
                textColor="text-black"
                shadowColor="rgba(0,0,0,1)"
                className="w-full font-bold text-lg"
              >
                Sign In
              </Button>
            </Link>
          </div>
        </div>
        <Footer />
      </main>
    );
  }

  return (
    <main className="flex flex-col min-h-screen bg-base">
      <Navbar />

      {xdrModal && (
        <XdrModal
          xdr={xdrModal.xdr}
          title={xdrModal.title}
          onClose={() => setXdrModal(null)}
        />
      )}

      <div className="flex-1 max-w-4xl w-full mx-auto px-4 lg:px-0 py-12 flex flex-col gap-10">

        {/* ── Header ── */}
        <div className="flex flex-col gap-4">
          <nav aria-label="Breadcrumb">
            <ol className="flex items-center gap-2 text-sm text-black/50">
              <li>
                <Link href="/organizers" className="hover:text-black transition-colors font-medium">
                  Organizers
                </Link>
              </li>
              <li aria-hidden>/</li>
              <li className="font-semibold text-black" aria-current="page">
                Verification & Staking
              </li>
            </ol>
          </nav>

          <div className="flex flex-col sm:flex-row sm:items-center sm:justify-between gap-4">
            <div>
              <h1 className="text-4xl md:text-5xl font-extrabold italic text-ink-deep leading-tight">
                Organizer Verification
              </h1>
              <p className="mt-2 text-black/60 text-lg">
                Stake collateral to earn your Verified badge and unlock premium
                hosting features.
              </p>
            </div>
            <VerifiedBadge verified={stakeState.isVerified} />
          </div>
        </div>

        {authLoading || isFetchingState ? (
          <LoadingSkeleton />
        ) : (
          <>
            {/* ── Stats row ── */}
            <div className="grid grid-cols-1 sm:grid-cols-3 gap-6">
              <StatCard
                label="Your Stake"
                value={`${stakeState.stakedAmount} XLM`}
                accent={stakeState.stakedAmount >= MIN_STAKE_XLM}
              />
              <StatCard
                label="Min. Collateral"
                value={`${MIN_STAKE_XLM} XLM`}
              />
              <StatCard
                label="Status"
                value={stakeState.isVerified ? "✓ Verified" : "Unverified"}
                accent={stakeState.isVerified}
              />
            </div>

            {/* ── How it works ── */}
            <section
              aria-labelledby="how-it-works-heading"
              className="bg-ink text-white rounded-3xl border-2 border-black shadow-[-6px_6px_0_rgba(0,0,0,1)] p-8 flex flex-col gap-6"
            >
              <h2
                id="how-it-works-heading"
                className="text-2xl font-extrabold italic"
              >
                How it works
              </h2>
              <ol className="flex flex-col sm:flex-row gap-6 sm:gap-0">
                {[
                  {
                    step: "01",
                    title: "Stake XLM",
                    body: `Lock at least ${MIN_STAKE_XLM} XLM as collateral. Your stake signals commitment to your audience.`,
                  },
                  {
                    step: "02",
                    title: "Get Verified",
                    body: "Once confirmed on-chain, your profile displays a Verified badge unlocking promoted listings.",
                  },
                  {
                    step: "03",
                    title: "Withdraw Anytime",
                    body: `Initiate a withdrawal and your collateral is returned after a ${Math.floor(UNSTAKE_LOCKUP_SECONDS / 86400)}-day lockup.`,
                  },
                ].map((item, i, arr) => (
                  <li key={item.step} className="flex-1 flex gap-4 sm:flex-col">
                    <div className="flex items-start gap-4 sm:flex-col sm:gap-3">
                      <span className="text-4xl font-extrabold text-accent leading-none">
                        {item.step}
                      </span>
                      {i < arr.length - 1 && (
                        <div className="hidden sm:block h-px w-full bg-white/20 mt-5" />
                      )}
                    </div>
                    <div>
                      <h3 className="font-bold text-lg">{item.title}</h3>
                      <p className="text-white/70 text-sm mt-1">{item.body}</p>
                    </div>
                  </li>
                ))}
              </ol>
            </section>

            {/* ── Stake panel ── */}
            {!stakeState.isPendingWithdrawal && (
              <section
                aria-labelledby="stake-heading"
                className="bg-white rounded-3xl border-2 border-black shadow-[-6px_6px_0_rgba(0,0,0,1)] p-8 flex flex-col gap-6"
              >
                <h2
                  id="stake-heading"
                  className="text-2xl font-extrabold italic"
                >
                  Stake Collateral
                </h2>

                <StakeInput
                  value={stakeInput}
                  onChange={setStakeInput}
                  min={MIN_STAKE_XLM}
                  error={stakeInputError}
                />

                <div className="flex flex-col sm:flex-row gap-4 items-start sm:items-center justify-between pt-2">
                  <p className="text-sm text-black/50 max-w-xs">
                    The transaction will be signed by your connected Stellar
                    wallet. No keys leave your device.
                  </p>
                  <Button
                    onClick={handleStake}
                    isLoading={isStaking}
                    backgroundColor="bg-accent"
                    textColor="text-black"
                    shadowColor="rgba(0,0,0,1)"
                    className="font-bold text-lg px-8 whitespace-nowrap"
                    aria-label="Generate stake XDR transaction"
                  >
                    {isStaking ? "Preparing…" : "Stake Collateral"}
                    {!isStaking && (
                      <Image
                        src="/icons/arrow-up-right-01.svg"
                        width={20}
                        height={20}
                        alt=""
                        aria-hidden="true"
                      />
                    )}
                  </Button>
                </div>
              </section>
            )}

            {/* ── Withdraw panel ── */}
            {stakeState.stakedAmount > 0 && (
              <section
                aria-labelledby="withdraw-heading"
                className="bg-surface rounded-3xl border-2 border-black shadow-[-6px_6px_0_rgba(0,0,0,1)] p-8 flex flex-col gap-6"
              >
                <h2
                  id="withdraw-heading"
                  className="text-2xl font-extrabold italic"
                >
                  Withdraw Collateral
                </h2>

                {stakeState.isPendingWithdrawal ? (
                  <div className="flex flex-col gap-4">
                    <div
                      className={`rounded-2xl border-2 border-black p-6 flex flex-col gap-2 shadow-[-4px_4px_0_rgba(0,0,0,1)] ${
                        lockupExpired ? "bg-success-light" : "bg-white"
                      }`}
                      aria-live="polite"
                      aria-atomic="true"
                    >
                      {lockupExpired ? (
                        <p className="font-bold text-lg">
                          ✓ Lockup complete — you may now withdraw your{" "}
                          <strong>{stakeState.stakedAmount} XLM</strong>.
                        </p>
                      ) : (
                        <>
                          <span className="text-sm font-semibold text-black/60 uppercase tracking-wider">
                            Withdrawal unlocks in
                          </span>
                          <span
                            className="text-4xl font-extrabold tabular-nums"
                            aria-label={`${formatCountdown(countdown)} remaining`}
                          >
                            {formatCountdown(countdown)}
                          </span>
                          <p className="text-sm text-black/50 mt-1">
                            Your staked collateral of{" "}
                            <strong>{stakeState.stakedAmount} XLM</strong> will
                            be returned once the lockup expires.
                          </p>
                        </>
                      )}
                    </div>

                    {lockupExpired && (
                      <Button
                        onClick={handleWithdraw}
                        isLoading={isWithdrawing}
                        backgroundColor="bg-accent"
                        textColor="text-black"
                        shadowColor="rgba(0,0,0,1)"
                        className="font-bold text-lg self-start"
                        aria-label="Complete withdrawal of staked collateral"
                      >
                        {isWithdrawing ? "Preparing…" : "Complete Withdrawal"}
                      </Button>
                    )}
                  </div>
                ) : (
                  <div className="flex flex-col gap-4">
                    <p className="text-sm text-black/60">
                      Initiating a withdrawal will remove your Verified status
                      immediately. Your{" "}
                      <strong>{stakeState.stakedAmount} XLM</strong> will be
                      returned after the{" "}
                      <strong>
                        {Math.floor(UNSTAKE_LOCKUP_SECONDS / 86400)}-day
                      </strong>{" "}
                      lockup period.
                    </p>
                    <Button
                      onClick={handleWithdraw}
                      isLoading={isWithdrawing}
                      backgroundColor="bg-white"
                      textColor="text-black"
                      shadowColor="rgba(0,0,0,1)"
                      className="font-bold self-start"
                      aria-label="Initiate collateral withdrawal"
                    >
                      {isWithdrawing ? "Preparing…" : "Initiate Withdrawal"}
                    </Button>
                  </div>
                )}
              </section>
            )}

            {/* ── Info banner ── */}
            <aside
              aria-label="Security information"
              className="rounded-2xl border-2 border-black bg-accent-muted p-6 flex gap-4 items-start shadow-[-4px_4px_0_rgba(0,0,0,1)]"
            >
              <span className="text-2xl" aria-hidden>
                🔐
              </span>
              <div>
                <h3 className="font-bold text-ink-deep">
                  Non-custodial &amp; trustless
                </h3>
                <p className="text-sm text-black/70 mt-1">
                  Agora never holds your keys. All staking operations are
                  executed via signed XDR payloads submitted directly to the
                  Stellar network through your own wallet.
                </p>
              </div>
            </aside>
          </>
        )}
      </div>

      <Footer />
    </main>
  );
}
