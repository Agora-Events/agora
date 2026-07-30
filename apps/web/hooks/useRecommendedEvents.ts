// apps/web/hooks/useRecommendedEvents.ts
import useSWR from "swr";

export interface RecommendedEvent {
  id: string;
  title: string;
  slug: string;
  description: string | null;
  start_time: string;   // ISO-8601
  end_time: string;
  location: string | null;
  banner_url: string | null;
  category_id: string;
  category_name: string;
  organizer_id: string;
  organizer_name: string;
  organizer_avatar: string | null;
  min_price: number | null;  // null → free
  tickets_remaining: number;
  relevance_score: number;
}

export interface RecommendationsResponse {
  events: RecommendedEvent[];
  personalised: boolean;
  based_on_categories: string[];
}

const fetcher = async (url: string): Promise<RecommendationsResponse> => {
  const res = await fetch(url, { credentials: "include" });
  if (!res.ok) {
    const err = new Error("Failed to load recommendations");
    throw err;
  }
  return res.json();
};

/**
 * Fetches personalised event recommendations for the current user.
 *
 * @param limit   Number of events to request (1–24, default 12)
 * @param enabled Set to false to skip the request (e.g. when user is logged out)
 */
export function useRecommendedEvents(limit = 12, enabled = true) {
  const url = enabled
    ? `/api/v1/recommendations/events?limit=${limit}`
    : null;

  const { data, error, isLoading, mutate } = useSWR<RecommendationsResponse>(
    url,
    fetcher,
    {
      revalidateOnFocus: false,
      dedupingInterval: 5 * 60 * 1000, // 5 min
    }
  );

  return {
    events: data?.events ?? [],
    personalised: data?.personalised ?? false,
    basedOnCategories: data?.based_on_categories ?? [],
    isLoading,
    isError: !!error,
    refresh: mutate,
  };
}