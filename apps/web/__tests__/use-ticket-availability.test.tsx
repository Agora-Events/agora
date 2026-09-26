import { describe, it, expect, vi, beforeEach, afterEach } from "vitest";
import { renderHook, act, waitFor } from "@testing-library/react";
import { SWRConfig } from "swr";
import type { ReactNode } from "react";
import { useTicketAvailability } from "@/hooks/useTicketAvailability";

const mockData = {
  totalTickets: 100,
  mintedTickets: 40,
  availableTickets: 60,
  isSoldOut: false,
  percentageSold: 40,
};

/**
 * Fresh SWR cache per test so polling/dedup state from one test can't leak
 * into the next (mirrors the pattern used in useEventDetails.test.tsx).
 */
function createWrapper() {
  return ({ children }: { children: ReactNode }) => (
    <SWRConfig value={{ provider: () => new Map(), dedupingInterval: 0 }}>
      {children}
    </SWRConfig>
  );
}

function setVisibility(state: "visible" | "hidden") {
  Object.defineProperty(document, "visibilityState", {
    configurable: true,
    get: () => state,
  });
  document.dispatchEvent(new Event("visibilitychange"));
}

describe("useTicketAvailability", () => {
  let consoleErrorSpy: ReturnType<typeof vi.spyOn>;

  beforeEach(() => {
    vi.useFakeTimers();
    setVisibility("visible");
    consoleErrorSpy = vi.spyOn(console, "error").mockImplementation(() => {});
  });

  afterEach(() => {
    // Fail loudly if React ever warned about a state update on an
    // unmounted component -- that's exactly the bug this hook must avoid.
    const unmountWarning = consoleErrorSpy.mock.calls.some((args) =>
      args.some(
        (arg) =>
          typeof arg === "string" &&
          arg.includes("state update on an unmounted component"),
      ),
    );
    expect(unmountWarning).toBe(false);

    consoleErrorSpy.mockRestore();
    vi.useRealTimers();
    vi.unstubAllGlobals();
  });

  it("fetches immediately on mount", async () => {
    const fetchMock = vi.fn().mockResolvedValue({
      ok: true,
      json: async () => mockData,
    });
    vi.stubGlobal("fetch", fetchMock);

    const { result } = renderHook(
      () => useTicketAvailability("evt_1", { pollInterval: 5000 }),
      { wrapper: createWrapper() },
    );

    await act(async () => {
      await vi.advanceTimersByTimeAsync(0);
    });

    expect(fetchMock).toHaveBeenCalledTimes(1);
    expect(fetchMock).toHaveBeenCalledWith(
      "/api/events/evt_1/availability",
      expect.objectContaining({ credentials: "include" }),
    );
    await waitFor(() => expect(result.current.data).toEqual(mockData));
    expect(result.current.isLoading).toBe(false);
    expect(result.current.error).toBeFalsy();
  });

  it("re-fetches after the polling interval elapses", async () => {
    const fetchMock = vi.fn().mockResolvedValue({
      ok: true,
      json: async () => mockData,
    });
    vi.stubGlobal("fetch", fetchMock);

    renderHook(() => useTicketAvailability("evt_2", { pollInterval: 5000 }), {
      wrapper: createWrapper(),
    });

    await act(async () => {
      await vi.advanceTimersByTimeAsync(0);
    });
    expect(fetchMock).toHaveBeenCalledTimes(1);

    await act(async () => {
      await vi.advanceTimersByTimeAsync(5000);
    });
    expect(fetchMock).toHaveBeenCalledTimes(2);

    await act(async () => {
      await vi.advanceTimersByTimeAsync(5000);
    });
    expect(fetchMock).toHaveBeenCalledTimes(3);
  });

  it("stops polling while hidden and resumes (with an immediate refetch) when visible again", async () => {
    const fetchMock = vi.fn().mockResolvedValue({
      ok: true,
      json: async () => mockData,
    });
    vi.stubGlobal("fetch", fetchMock);

    renderHook(() => useTicketAvailability("evt_3", { pollInterval: 5000 }), {
      wrapper: createWrapper(),
    });

    await act(async () => {
      await vi.advanceTimersByTimeAsync(0);
    });
    expect(fetchMock).toHaveBeenCalledTimes(1);

    act(() => {
      setVisibility("hidden");
    });

    // Plenty of time for several poll intervals to have elapsed, but the
    // tab is hidden so no additional fetches should occur.
    await act(async () => {
      await vi.advanceTimersByTimeAsync(20000);
    });
    expect(fetchMock).toHaveBeenCalledTimes(1);

    act(() => {
      setVisibility("visible");
    });

    // Becoming visible again triggers an immediate re-fetch to catch missed
    // updates.
    await act(async () => {
      await vi.advanceTimersByTimeAsync(0);
    });
    expect(fetchMock).toHaveBeenCalledTimes(2);

    // Polling resumes on the configured interval afterwards.
    await act(async () => {
      await vi.advanceTimersByTimeAsync(5000);
    });
    expect(fetchMock).toHaveBeenCalledTimes(3);
  });

  it("stops all timers and fetches after unmount", async () => {
    const fetchMock = vi.fn().mockResolvedValue({
      ok: true,
      json: async () => mockData,
    });
    vi.stubGlobal("fetch", fetchMock);

    const { unmount } = renderHook(
      () => useTicketAvailability("evt_4", { pollInterval: 5000 }),
      { wrapper: createWrapper() },
    );

    await act(async () => {
      await vi.advanceTimersByTimeAsync(0);
    });
    const callsBeforeUnmount = fetchMock.mock.calls.length;
    expect(callsBeforeUnmount).toBeGreaterThan(0);

    unmount();

    await act(async () => {
      await vi.advanceTimersByTimeAsync(30000);
    });

    expect(fetchMock).toHaveBeenCalledTimes(callsBeforeUnmount);
  });

  it("surfaces an error state without crashing when the fetch fails", async () => {
    const fetchMock = vi.fn().mockResolvedValue({
      ok: false,
      json: async () => ({}),
    });
    vi.stubGlobal("fetch", fetchMock);

    const { result } = renderHook(
      () => useTicketAvailability("evt_5", { pollInterval: 5000 }),
      { wrapper: createWrapper() },
    );

    await act(async () => {
      await vi.advanceTimersByTimeAsync(0);
    });

    await waitFor(() => expect(result.current.error).toBeTruthy());
    expect(result.current.data).toBeUndefined();
    expect(result.current.isLoading).toBe(false);
  });
});
