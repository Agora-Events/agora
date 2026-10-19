import { afterEach, describe, expect, it, vi } from "vitest";
import { act, renderHook } from "@testing-library/react";
import useIsMobile from "@/hooks/useIsMobile";

describe("useIsMobile", () => {
  afterEach(() => {
    vi.restoreAllMocks();
  });

  it("returns true when window width is below the breakpoint", () => {
    Object.defineProperty(window, "innerWidth", {
      writable: true,
      configurable: true,
      value: 480,
    });

    const { result } = renderHook(() => useIsMobile(768));

    expect(result.current).toBe(true);
  });

  it("returns false when window width is above the breakpoint", () => {
    Object.defineProperty(window, "innerWidth", {
      writable: true,
      configurable: true,
      value: 1200,
    });

    const { result } = renderHook(() => useIsMobile(768));

    expect(result.current).toBe(false);
  });

  it("updates value after a resize event", () => {
    Object.defineProperty(window, "innerWidth", {
      writable: true,
      configurable: true,
      value: 1200,
    });

    const { result } = renderHook(() => useIsMobile(768));
    expect(result.current).toBe(false);

    act(() => {
      window.innerWidth = 500;
      window.dispatchEvent(new Event("resize"));
    });

    expect(result.current).toBe(true);
  });

  it("removes resize listener on unmount", () => {
    const removeSpy = vi.spyOn(window, "removeEventListener");

    const { unmount } = renderHook(() => useIsMobile(768));
    unmount();

    expect(removeSpy).toHaveBeenCalledWith("resize", expect.any(Function));
  });
});
