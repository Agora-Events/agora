"use client";

import useSWR from "swr";
import type { PoapCollectible } from "@/components/wallet/PoapCard";

// ---------------------------------------------------------------------------
// Types
// ---------------------------------------------------------------------------

export interface WalletTicket {
  id: string;
  status: string;
  ticket_tier_id: string | null;
  ticket_tier_name: string | null;
  ticket_price: string | null;
  event_id: string | null;
  event_title: string | null;
  event_location: string | null;
  event_start_time: string | null;
  event_image_url: string | null;
  created_at: string;
  poap_minted?: boolean;
  poap_id?: string;
}

export interface WalletTicketsData {
  upcoming: WalletTicket[];
  past: WalletTicket[];
  poaps?: PoapCollectible[];
}

interface ApiResponse {
  data: WalletTicketsData;
  message: string;
}

// ---------------------------------------------------------------------------
// Fetcher
// ---------------------------------------------------------------------------

async function fetchWalletTickets(url: string): Promise<WalletTicketsData> {
  const res = await fetch(url, { credentials: "same-origin" });
  if (res.status === 401) {
    throw new Error("Unauthorized");
  }
  if (!res.ok) {
    throw new Error(`Failed to load wallet tickets (${res.status})`);
  }
  const json: ApiResponse = await res.json();
  return json.data;
}

// ---------------------------------------------------------------------------
// Hook
// ---------------------------------------------------------------------------

/**
 * Fetches the authenticated user's wallet tickets, split into upcoming and
 * past collections alongside earned POAP collectibles.
 *
 * @example
 * const { upcoming, past, poaps, isLoading, error } = useWalletTickets();
 */
export function useWalletTickets() {
  const { data, error, isLoading, mutate } = useSWR<WalletTicketsData>(
    "/api/v1/profile/tickets",
    fetchWalletTickets,
    {
      revalidateOnFocus: false,
      shouldRetryOnError: false,
    },
  );

  return {
    upcoming: data?.upcoming ?? [],
    past: data?.past ?? [],
    poaps: data?.poaps ?? [],
    isLoading,
    error: error as Error | undefined,
    mutate,
  };
}
