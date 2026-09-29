import { describe, it, expect, vi, afterEach } from "vitest";
import { renderHook, waitFor } from "@testing-library/react";
import { SWRConfig } from "swr";
import type { ReactNode } from "react";
import { useRecommendedEvents } from "@/hooks/useRecommendedEvents";
import type { RecommendedEvent, RecommendationsResponse } from "@/hooks/useRecommendedEvents";

/**
 * Fresh SWR cache per test so state from one test can't leak into the next
 * (mirrors the pattern used in use-ticket-availability.test.tsx).
 */
function createWrapper() {
  return ({ children }: { children: ReactNode }) => (
    <SWRConfig value={{ provider: () => new Map(), dedupingInterval: 0 }}>{children}</SWRConfig>
  );
}

function makeEvent(overrides: Partial<RecommendedEvent> = {}): RecommendedEvent {
  return {
    id: "evt_1",
    title: "Lagos Tech Meetup",
    slug: "lagos-tech-meetup",
    description: null,
    start_time: "2026-07-01T18:00:00.000Z",
    end_time: "2026-07-01T21:00:00.000Z",
    location: "Lagos, Nigeria",
    banner_url: null,
    category_id: "cat_1",
    category_name: "Tech",
    organizer_id: "org_1",
    organizer_name: "Agora",
    organizer_avatar: null,
    min_price: null,
    tickets_remaining: 50,
    relevance_score: 0.9,
    ...overrides,
  };
}

describe("useRecommendedEvents", () => {
  afterEach(() => {
    vi.unstubAllGlobals();
  });

  it("returns the recommended events from the mocked API", async () => {
    const response: RecommendationsResponse = {
      events: [makeEvent()],
      personalised: true,
      based_on_categories: ["Tech"],
    };
    const fetchMock = vi.fn().mockResolvedValue({
      ok: true,
      json: async () => response,
    });
    vi.stubGlobal("fetch", fetchMock);

    const { result } = renderHook(() => useRecommendedEvents(), {
      wrapper: createWrapper(),
    });

    await waitFor(() => expect(result.current.isLoading).toBe(false));

    expect(result.current.events).toEqual(response.events);
    expect(result.current.personalised).toBe(true);
    expect(result.current.basedOnCategories).toEqual(["Tech"]);
    expect(result.current.isError).toBe(false);
    expect(fetchMock).toHaveBeenCalledWith(
      "/api/v1/recommendations/events?limit=12",
      expect.objectContaining({ credentials: "include" })
    );
  });

  it("surfaces an error state without crashing when the request fails", async () => {
    const fetchMock = vi.fn().mockResolvedValue({
      ok: false,
      json: async () => ({}),
    });
    vi.stubGlobal("fetch", fetchMock);

    const { result } = renderHook(() => useRecommendedEvents(), {
      wrapper: createWrapper(),
    });

    await waitFor(() => expect(result.current.isError).toBe(true));

    expect(result.current.events).toEqual([]);
    expect(result.current.isLoading).toBe(false);
  });

  it("returns an empty events array when the API has no recommendations", async () => {
    const response: RecommendationsResponse = {
      events: [],
      personalised: false,
      based_on_categories: [],
    };
    const fetchMock = vi.fn().mockResolvedValue({
      ok: true,
      json: async () => response,
    });
    vi.stubGlobal("fetch", fetchMock);

    const { result } = renderHook(() => useRecommendedEvents(), {
      wrapper: createWrapper(),
    });

    await waitFor(() => expect(result.current.isLoading).toBe(false));

    expect(result.current.events).toEqual([]);
    expect(result.current.personalised).toBe(false);
  });

  it("refetches when the limit parameter changes", async () => {
    const response: RecommendationsResponse = {
      events: [makeEvent()],
      personalised: false,
      based_on_categories: [],
    };
    const fetchMock = vi.fn().mockResolvedValue({
      ok: true,
      json: async () => response,
    });
    vi.stubGlobal("fetch", fetchMock);

    const { result, rerender } = renderHook(
      ({ limit }: { limit: number }) => useRecommendedEvents(limit),
      { wrapper: createWrapper(), initialProps: { limit: 12 } }
    );

    await waitFor(() => expect(result.current.isLoading).toBe(false));
    expect(fetchMock).toHaveBeenCalledWith(
      "/api/v1/recommendations/events?limit=12",
      expect.anything()
    );

    rerender({ limit: 6 });

    await waitFor(() =>
      expect(fetchMock).toHaveBeenCalledWith(
        "/api/v1/recommendations/events?limit=6",
        expect.anything()
      )
    );
  });

  it("does not fetch while disabled, and fetches once enabled", async () => {
    const response: RecommendationsResponse = {
      events: [makeEvent()],
      personalised: false,
      based_on_categories: [],
    };
    const fetchMock = vi.fn().mockResolvedValue({
      ok: true,
      json: async () => response,
    });
    vi.stubGlobal("fetch", fetchMock);

    const { result, rerender } = renderHook(
      ({ enabled }: { enabled: boolean }) => useRecommendedEvents(12, enabled),
      { wrapper: createWrapper(), initialProps: { enabled: false } }
    );

    expect(result.current.isLoading).toBe(false);
    expect(fetchMock).not.toHaveBeenCalled();

    rerender({ enabled: true });

    await waitFor(() => expect(fetchMock).toHaveBeenCalledTimes(1));
    await waitFor(() => expect(result.current.events).toEqual(response.events));
  });
});
