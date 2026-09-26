"use client";

import { useCallback } from "react";
import { useRouter } from "next/navigation";
import useSWR from "swr";
import type { AuthUser, SessionResponse } from "@/types/auth";

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
}

/**
 * Centralised authentication state.
 *
 * Wraps the custom-JWT session behind `GET /api/auth/session` and shares it
 * across the app through SWR's cache, so every consumer sees the same user
 * without issuing its own request.
 *
 * @example
 * const { user, walletAddress, isAuthenticated, isLoading, signOut } = useAuth();
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

  // A failed session lookup means we cannot prove the user is signed in.
  const user: AuthUser | null = error ? null : (data?.user ?? null);

  const signOut = useCallback(async () => {
    try {
      await fetch(SIGNOUT_ENDPOINT, {
        method: "POST",
        credentials: "same-origin",
      });
    } finally {
      // Drop the cached session even if the request failed, so the UI never
      // keeps showing a user we can no longer authenticate.
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
  };
}
