import { afterEach, describe, expect, it, vi } from "vitest";
import { trackFunnelEvent } from "@/utils/funnel-analytics";

describe("trackFunnelEvent", () => {
  afterEach(() => {
    vi.restoreAllMocks();
    vi.unstubAllGlobals();
  });

  it("sends the analytics event with the expected payload", async () => {
    const fetchMock = vi.fn().mockResolvedValue({ ok: true });
    vi.stubGlobal("fetch", fetchMock);

    await trackFunnelEvent("page_view", 12345);

    expect(fetchMock).toHaveBeenCalledTimes(1);
    expect(fetchMock).toHaveBeenCalledWith(
      "/api/analytics",
      expect.objectContaining({
        method: "POST",
        headers: { "Content-Type": "application/json" },
        body: JSON.stringify({ type: "page_view", eventId: "12345" }),
        keepalive: true,
      }),
    );
  });

  it("handles empty and unusual values by coercing eventId to string", async () => {
    const fetchMock = vi.fn().mockResolvedValue({ ok: true });
    vi.stubGlobal("fetch", fetchMock);

    await trackFunnelEvent("checkout_started", "" as unknown as number);
    await trackFunnelEvent("checkout_started", Number.NaN);

    expect(fetchMock).toHaveBeenNthCalledWith(
      1,
      "/api/analytics",
      expect.objectContaining({
        body: JSON.stringify({ type: "checkout_started", eventId: "" }),
      }),
    );
    expect(fetchMock).toHaveBeenNthCalledWith(
      2,
      "/api/analytics",
      expect.objectContaining({
        body: JSON.stringify({ type: "checkout_started", eventId: "NaN" }),
      }),
    );
  });

  it("does not throw when the network request fails", async () => {
    vi.stubGlobal(
      "fetch",
      vi.fn().mockRejectedValue(new Error("network unavailable")),
    );

    await expect(trackFunnelEvent("page_view", "evt_42")).resolves.toBeUndefined();
  });
});
