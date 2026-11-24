import { afterEach, describe, expect, it, vi } from "vitest";
import { fetchPopularEvents } from "@/utils/api";

describe("fetchPopularEvents", () => {
  afterEach(() => {
    vi.unstubAllGlobals();
  });

  it("rethrows the original AbortError when the fallback request is cancelled", async () => {
    const controller = new AbortController();
    const abortError = new DOMException("Request cancelled", "AbortError");
    const fetchMock = vi
      .fn<typeof fetch>()
      .mockResolvedValueOnce(Response.json({}))
      .mockImplementationOnce(async (_input, options) => {
        controller.abort(abortError);
        options?.signal?.throwIfAborted();
        throw new Error("Expected the fallback request to be cancelled");
      });
    vi.stubGlobal("fetch", fetchMock);

    await expect(fetchPopularEvents(2, controller.signal)).rejects.toBe(abortError);
    expect(fetchMock).toHaveBeenNthCalledWith(1, "/api/events/discover?page=2", {
      signal: controller.signal,
    });
    expect(fetchMock).toHaveBeenNthCalledWith(2, "/api/events/discover", {
      signal: controller.signal,
    });
  });

  it("keeps the normalization error for other fallback failures", async () => {
    const fetchMock = vi
      .fn<typeof fetch>()
      .mockResolvedValueOnce(Response.json({}))
      .mockRejectedValueOnce(new Error("Network failure"));
    vi.stubGlobal("fetch", fetchMock);

    await expect(fetchPopularEvents()).rejects.toThrow(
      "Unable to normalize popular events payload"
    );
  });
});
