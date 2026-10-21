"use client";

import { useCallback, useEffect } from "react";
import { useRouter } from "next/navigation";
import useSWR from "swr";
import { toast } from "sonner";
import type { AuthUser, SessionResponse } from "@/types/auth";
import { useStellarNetwork, TESTNET_PASSPHRASE } from "@/hooks/useStellarNetwork";

export type { AuthUser, SessionResponse };

const SESSION_ENDPOINT = "/api/auth/session";
const SIGNOUT_ENDPOINT = "/api/auth/signout";

const SIGNED_OUT: SessionResponse = { user: null };

async function fetchSession(url: string): Promise<SessionResponse> {
  const response = await fetch(url, { credentials: "same-origin" });

  // Treat an unauthorized response as "signed out" rather than a failure.
  if (response.status === 401) {
    return SIGNED_OUT;
  }

  if (!response.ok) {
    throw new Error(`Failed to load session (${response.status})`);
  }

  return (await response.json()) as SessionResponse;
}

/** Value returned by {@link useAuth}. */
export interface UseAuthResult {
  /** The signed-in user, or `null` when signed out or still loading. */
  user: AuthUser | null;
  /** Convenience accessor for `user.walletAddress`. */
  walletAddress: string | null;
  /** `true` once a valid session has been loaded. */
  isAuthenticated: boolean;
  /** `true` while the session request is in flight. */
  isLoading: boolean;
  /** Clears the session cookie and redirects to `/`. */
  signOut: () => Promise<void>;
  /**
   * Network validation from Issue #1490.
   * `true` when the connected Freighter wallet is on Stellar Testnet.
   * Always `true` in non-testnet (production) builds.
   */
  isCorrectNetwork: boolean;
  /** The wallet's currently active network passphrase, or `null`. */
  currentNetwork: string | null;
  /**
   * Prompts the user to switch Freighter to Testnet.
   * No-op in production builds.
   */
  requestNetworkSwitch: () => void;
}

/**
 * Centralised authentication state — Issue #1490 adds Stellar network
 * validation so callers know whether the connected wallet is on the right
 * network before submitting any Soroban transaction.
 *
 * Network checking only activates when `NEXT_PUBLIC_STELLAR_NETWORK` is set
 * to `"TESTNET"`, so production builds are unaffected.
 *
 * @example
 * const { user, isAuthenticated, isCorrectNetwork, requestNetworkSwitch } = useAuth();
 * if (!isCorrectNetwork) { requestNetworkSwitch(); return; }
 */
export function useAuth(): UseAuthResult {
  const router = useRouter();

  const { data, error, isLoading, mutate } = useSWR<SessionResponse>(
    SESSION_ENDPOINT,
    fetchSession,
    {
      revalidateOnFocus: false,
      shouldRetryOnError: false,
    },
  );

  const isTestnetBuild =
    process.env.NEXT_PUBLIC_STELLAR_NETWORK === "TESTNET";

  // Network state is only polled on testnet builds to avoid unnecessary
  // Freighter calls in production.
  const {
    isCorrectNetwork: _isCorrectNetwork,
    currentNetwork,
    isLoading: networkLoading,
    requestNetworkSwitch,
  } = useStellarNetwork();

  // Derive effective network correctness:
  // - In production builds, always treat as correct (no wallet gate).
  // - In testnet builds, reflect the live Freighter state.
  const isCorrectNetwork = !isTestnetBuild || _isCorrectNetwork;

  // A failed session lookup means we cannot prove the user is signed in.
  const user: AuthUser | null = error ? null : (data?.user ?? null);

  // Warn the authenticated user once when they are on the wrong network.
  useEffect(() => {
    if (!isTestnetBuild) return;
    if (!user || networkLoading) return;
    if (!_isCorrectNetwork) {
      toast.warning(
        `Your Freighter wallet is not on Stellar Testnet. ` +
          `Please switch to "${TESTNET_PASSPHRASE}" to transact.`,
        { id: "wrong-network", duration: 8_000 },
      );
    }
  }, [_isCorrectNetwork, user, networkLoading, isTestnetBuild]);

  const signOut = useCallback(async () => {
    try {
      await fetch(SIGNOUT_ENDPOINT, {
        method: "POST",
        credentials: "same-origin",
      });
    } finally {
      // Drop the cached session even if the request failed.
      await mutate(SIGNED_OUT, { revalidate: false });
      router.replace("/");
      router.refresh();
    }
  }, [mutate, router]);

  return {
    user,
    walletAddress: user?.walletAddress ?? null,
    isAuthenticated: user !== null,
    isLoading,
    signOut,
    isCorrectNetwork,
    currentNetwork,
    requestNetworkSwitch,
  };
}
