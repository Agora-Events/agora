"use client";

import { useState, useEffect, useCallback } from "react";
import { useRouter } from "next/navigation";
import Link from "next/link";
import { Button } from "@/components/ui/button";
import { useAuth } from "@/hooks/useAuth";
import { toast } from "sonner";

// ─── Types ────────────────────────────────────────────────────────────────────

interface ProStatus {
  active: boolean;
  billingCycleEndsAt: string | null;
  priceUsdc: string;
}

interface SeriesPass {
  id: string;
  eventName: string;
  validFrom: string;
  validUntil: string;
  totalUses: number;
  usedUses: number;
}

interface SubscriptionData {
  pro: ProStatus;
  seriesPasses: SeriesPass[];
}

// ─── Constants ────────────────────────────────────────────────────────────────

const PRO_BENEFITS = [
  { emoji: "🚀", text: "Promoted event listings in Discover" },
  { emoji: "📊", text: "Advanced attendee analytics dashboard" },
  { emoji: "🎟️", text: "Unlimited series pass creation" },
  { emoji: "✓",  text: "Verified Organizer badge on your profile" },
  { emoji: "💬", text: "Priority support channel access" },
];

// ─── Helpers ─────────────────────────────────────────────────────────────────

function fmt(iso: string): string {
  return new Date(iso).toLocaleDateString("en-US", {
    year: "numeric",
    month: "short",
    day: "numeric",
  });
}

function daysLeft(iso: string): number {
  return Math.max(0, Math.ceil((new Date(iso).getTime() - Date.now()) / 86_400_000));
}

function usesLeft(pass: SeriesPass): number {
  return pass.totalUses - pass.usedUses;
}

// ─── Sub-components ───────────────────────────────────────────────────────────

function SettingsTabs({ active }: { active: string }) {
  const tabs = [
    { id: "profile",       label: "Profile",       href: "/settings" },
    { id: "notifications", label: "Notifications", href: "/settings" },
    { id: "payment",       label: "Payment",       href: "/settings" },
    { id: "subscriptions", label: "Subscriptions", href: "/settings/subscriptions" },
  ];
  return (
    <div className="flex border-b border-border-warm">
      {tabs.map((t) => (
        <Link
          key={t.id}
          href={t.href}
          className={`flex-1 px-4 py-4 text-sm font-semibold text-center transition-colors ${
            active === t.id
              ? "text-ink-soft border-b-2 border-ink"
              : "text-muted-text hover:text-ink-soft"
          }`}
          aria-current={active === t.id ? "page" : undefined}
        >
          {t.label}
        </Link>
      ))}
    </div>
  );
}

function ProBenefitItem({ emoji, text }: { emoji: string; text: string }) {
  return (
    <li className="flex items-center gap-3">
      <span
        className="w-8 h-8 rounded-full bg-accent border-2 border-black flex items-center justify-center text-sm shrink-0"
        aria-hidden="true"
      >
        {emoji}
      </span>
      <span className="text-sm font-medium text-ink-soft">{text}</span>
    </li>
  );
}

function SkeletonCard({ rows = 4 }: { rows?: number }) {
  return (
    <div className="animate-pulse rounded-2xl border border-border-warm bg-white p-6 flex flex-col gap-4">
      <div className="h-5 w-40 bg-black/10 rounded-lg" />
      {Array.from({ length: rows }).map((_, i) => (
        <div key={i} className="h-4 bg-black/10 rounded-lg" style={{ width: `${70 + (i % 3) * 10}%` }} />
      ))}
    </div>
  );
}

function PassCard({ pass }: { pass: SeriesPass }) {
  const remaining = usesLeft(pass);
  const pct = Math.round((remaining / pass.totalUses) * 100);
  const expired = new Date(pass.validUntil) < new Date();

  return (
    <article
      aria-label={`Series pass for ${pass.eventName}`}
      className="bg-white rounded-2xl border-2 border-black shadow-[-4px_4px_0_rgba(0,0,0,1)] p-6 flex flex-col gap-4"
    >
      <div className="flex items-start justify-between gap-3">
        <h3 className="font-extrabold text-ink-deep text-lg leading-tight">
          {pass.eventName}
        </h3>
        <span
          className={`shrink-0 text-xs font-bold px-3 py-1 rounded-full border-2 border-black ${
            expired ? "bg-surface text-black/50" : "bg-success-light text-black"
          }`}
        >
          {expired ? "Expired" : "Active"}
        </span>
      </div>

      {/* Validity */}
      <div className="text-sm text-muted-text">
        <span className="font-semibold text-ink-soft">{fmt(pass.validFrom)}</span>
        {" → "}
        <span className="font-semibold text-ink-soft">{fmt(pass.validUntil)}</span>
        {!expired && (
          <span className="ml-2 text-xs">({daysLeft(pass.validUntil)} days left)</span>
        )}
      </div>

      {/* Uses bar */}
      <div className="flex flex-col gap-1.5">
        <div className="flex justify-between text-xs font-semibold text-muted-text">
          <span>Uses remaining</span>
          <span>
            {remaining} / {pass.totalUses}
          </span>
        </div>
        <div
          className="h-3 w-full rounded-full border-2 border-black bg-surface overflow-hidden"
          role="progressbar"
          aria-valuenow={remaining}
          aria-valuemin={0}
          aria-valuemax={pass.totalUses}
          aria-label={`${remaining} of ${pass.totalUses} uses remaining`}
        >
          <div
            className="h-full bg-accent transition-all duration-500"
            style={{ width: `${pct}%` }}
          />
        </div>
      </div>
    </article>
  );
}

// ─── Main page ────────────────────────────────────────────────────────────────

export default function SubscriptionsPage() {
  const router = useRouter();
  const { user, isAuthenticated, isLoading: authLoading } = useAuth();

  const [data, setData] = useState<SubscriptionData | null>(null);
  const [fetchError, setFetchError] = useState<string | null>(null);
  const [isLoading, setIsLoading] = useState(true);
  const [proActionLoading, setProActionLoading] = useState(false);

  // Auth guard
  useEffect(() => {
    if (!authLoading && !isAuthenticated) {
      router.replace("/auth");
    }
  }, [authLoading, isAuthenticated, router]);

  // Fetch subscription state
  useEffect(() => {
    if (!isAuthenticated) return;
    const controller = new AbortController();

    (async () => {
      try {
        const res = await fetch("/api/subscriptions", {
          credentials: "same-origin",
          signal: controller.signal,
        });
        if (!res.ok) {
          const body = await res.json().catch(() => ({}));
          throw new Error(body.error ?? `Request failed (${res.status})`);
        }
        setData(await res.json());
      } catch (err) {
        if (err instanceof Error && err.name === "AbortError") return;
        setFetchError(err instanceof Error ? err.message : "Failed to load subscriptions");
      } finally {
        if (!controller.signal.aborted) setIsLoading(false);
      }
    })();

    return () => controller.abort();
  }, [isAuthenticated]);

  const callSubscriptionApi = useCallback(
    async (action: string, passId?: string): Promise<boolean> => {
      try {
        const res = await fetch("/api/subscriptions", {
          method: "POST",
          credentials: "same-origin",
          headers: { "Content-Type": "application/json" },
          body: JSON.stringify({ action, ...(passId ? { passId } : {}) }),
        });
        const body = await res.json().catch(() => ({}));
        if (!res.ok) throw new Error(body.error ?? `Request failed (${res.status})`);
        return true;
      } catch (err) {
        toast.error(err instanceof Error ? err.message : "Something went wrong");
        return false;
      }
    },
    [],
  );

  const handleProToggle = useCallback(async () => {
    if (!data) return;
    const isActive = data.pro.active;
    setProActionLoading(true);

    const ok = await callSubscriptionApi(isActive ? "cancel_pro" : "subscribe_pro");
    if (ok) {
      setData((prev) =>
        prev
          ? {
              ...prev,
              pro: {
                ...prev.pro,
                active: !isActive,
                billingCycleEndsAt: !isActive
                  ? new Date(Date.now() + 30 * 24 * 60 * 60 * 1000).toISOString()
                  : null,
              },
            }
          : prev,
      );
      toast.success(isActive ? "Pro subscription cancelled." : "Welcome to Agora Pro! 🎉");
    }

    setProActionLoading(false);
  }, [data, callSubscriptionApi]);

  // ── Render ────────────────────────────────────────────────────────────────

  if (authLoading) return null;

  return (
    <main className="flex flex-col min-h-screen bg-base">
      <div className="w-full max-w-3xl mx-auto px-4 py-10">

        {/* Page header */}
        <div className="mb-8 flex flex-wrap items-start justify-between gap-4">
          <div>
            <h1 className="text-3xl font-bold text-ink-soft">Settings</h1>
            <p className="text-muted-text mt-1">Manage your account preferences</p>
            {user && (
              <p className="text-muted-text mt-1 text-sm break-all">
                Signed in as{" "}
                <span className="font-medium text-ink-soft">
                  {user.email ?? user.walletAddress ?? user.id}
                </span>
              </p>
            )}
          </div>
        </div>

        <div className="bg-white rounded-2xl border border-border-warm shadow-[-4px_4px_0_rgba(0,0,0,1)] overflow-hidden">
          <SettingsTabs active="subscriptions" />

          <div className="p-6 md:p-8 flex flex-col gap-8">

            {/* Error state */}
            {fetchError && (
              <div
                role="alert"
                className="rounded-xl border border-error bg-red-50 px-4 py-3 text-sm text-error"
              >
                {fetchError}
              </div>
            )}

            {/* ── Agora Pro card ───────────────────────────────────────── */}
            <section aria-labelledby="pro-heading">
              <h2
                id="pro-heading"
                className="text-lg font-bold text-ink-soft mb-4"
              >
                Agora Pro Organiser
              </h2>

              {isLoading ? (
                <SkeletonCard rows={5} />
              ) : (
                <div
                  className={`rounded-2xl border-2 border-black shadow-[-4px_4px_0_rgba(0,0,0,1)] overflow-hidden`}
                >
                  {/* Header band */}
                  <div
                    className={`flex items-center justify-between px-6 py-4 ${
                      data?.pro.active ? "bg-accent" : "bg-ink"
                    }`}
                  >
                    <div className="flex items-center gap-3">
                      <span
                        className="text-2xl"
                        aria-hidden="true"
                      >
                        {data?.pro.active ? "⭐" : "🔓"}
                      </span>
                      <div>
                        <p
                          className={`font-extrabold text-lg leading-tight ${
                            data?.pro.active ? "text-black" : "text-white"
                          }`}
                        >
                          Agora Pro
                        </p>
                        <p
                          className={`text-xs font-semibold ${
                            data?.pro.active ? "text-black/70" : "text-white/60"
                          }`}
                        >
                          {data?.pro.active
                            ? data.pro.billingCycleEndsAt
                              ? `Renews ${fmt(data.pro.billingCycleEndsAt)}`
                              : "Active"
                            : `$${data?.pro.priceUsdc ?? "9.99"} USDC / month`}
                        </p>
                      </div>
                    </div>

                    {/* Active badge */}
                    {data?.pro.active && (
                      <span className="text-xs font-bold px-3 py-1 rounded-full border-2 border-black bg-white">
                        Active
                      </span>
                    )}
                  </div>

                  {/* Benefits list */}
                  <div className="bg-white px-6 py-5">
                    <ul className="flex flex-col gap-3" aria-label="Pro benefits">
                      {PRO_BENEFITS.map((b) => (
                        <ProBenefitItem key={b.text} {...b} />
                      ))}
                    </ul>

                    {/* Billing info */}
                    {data?.pro.active && data.pro.billingCycleEndsAt && (
                      <p className="mt-4 text-xs text-muted-text">
                        Your subscription renews on{" "}
                        <strong>{fmt(data.pro.billingCycleEndsAt)}</strong>.
                        Cancelling now stops renewal; access continues until
                        that date.
                      </p>
                    )}

                    {/* CTA */}
                    <div className="mt-6 flex flex-col sm:flex-row gap-3">
                      <Button
                        onClick={handleProToggle}
                        isLoading={proActionLoading}
                        backgroundColor={data?.pro.active ? "bg-white" : "bg-accent"}
                        textColor="text-black"
                        shadowColor="rgba(0,0,0,1)"
                        className="font-bold flex-1"
                        aria-label={
                          data?.pro.active
                            ? "Cancel Agora Pro subscription"
                            : "Subscribe to Agora Pro"
                        }
                        disabled={proActionLoading}
                      >
                        {proActionLoading
                          ? "Processing…"
                          : data?.pro.active
                          ? "Cancel Subscription"
                          : `Subscribe — $${data?.pro.priceUsdc ?? "9.99"} USDC/mo`}
                      </Button>
                      {!data?.pro.active && (
                        <Link
                          href="/pricing"
                          className="text-sm font-semibold text-muted-text self-center hover:text-ink-soft transition-colors"
                        >
                          Compare plans →
                        </Link>
                      )}
                    </div>
                  </div>
                </div>
              )}
            </section>

            {/* ── Series Passes ─────────────────────────────────────────── */}
            <section aria-labelledby="passes-heading">
              <div className="flex items-center justify-between mb-4">
                <h2
                  id="passes-heading"
                  className="text-lg font-bold text-ink-soft"
                >
                  My Series Passes
                </h2>
                <span className="text-sm text-muted-text">
                  {data?.seriesPasses.length ?? 0} active
                </span>
              </div>

              {isLoading ? (
                <div className="flex flex-col gap-4">
                  <SkeletonCard rows={3} />
                  <SkeletonCard rows={3} />
                </div>
              ) : data?.seriesPasses.length === 0 ? (
                <div className="rounded-2xl border-2 border-dashed border-black/20 p-10 flex flex-col items-center text-center gap-4">
                  <span className="text-4xl" aria-hidden="true">🎟️</span>
                  <div>
                    <h3 className="font-bold text-ink-soft">No series passes yet</h3>
                    <p className="text-sm text-muted-text mt-1 max-w-xs">
                      Series passes give you repeated access to recurring events.
                      Browse events to find season passes.
                    </p>
                  </div>
                  <Link href="/discover">
                    <Button
                      backgroundColor="bg-accent"
                      textColor="text-black"
                      shadowColor="rgba(0,0,0,1)"
                      className="font-bold"
                    >
                      Browse Events
                    </Button>
                  </Link>
                </div>
              ) : (
                <div className="flex flex-col gap-4">
                  {data.seriesPasses.map((pass) => (
                    <PassCard key={pass.id} pass={pass} />
                  ))}
                </div>
              )}
            </section>

            {/* ── Info footer ───────────────────────────────────────────── */}
            <aside
              aria-label="Billing information"
              className="rounded-xl border border-border-warm bg-surface p-5 flex gap-3 items-start text-sm text-muted-text"
            >
              <span className="text-lg" aria-hidden="true">ℹ️</span>
              <p>
                All payments are processed in{" "}
                <strong className="text-ink-soft">USDC on the Stellar network</strong>.
                Transactions are non-custodial — Agora never holds your private keys.
                Need help?{" "}
                <Link
                  href="/help"
                  className="underline font-semibold text-ink-soft hover:text-ink transition-colors"
                >
                  Visit our Help Centre
                </Link>
                .
              </p>
            </aside>
          </div>
        </div>
      </div>
    </main>
  );
}
