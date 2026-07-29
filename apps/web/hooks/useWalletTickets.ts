"use client";

import useSWR from "swr";

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
}

export interface WalletTicketsData {
  upcoming: WalletTicket[];
  past: WalletTicket[];
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
 * past collections.
 *
 * @example
 * const { upcoming, past, isLoading, error } = useWalletTickets();
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
    isLoading,
    error: error as Error | undefined,
    mutate,
  };
}
