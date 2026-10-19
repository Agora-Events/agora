import { render, screen, act } from "@testing-library/react";
import { describe, it, expect, vi, beforeEach, afterEach } from "vitest";
import { EventCountdown } from "@/components/events/event-countdown";

describe("EventCountdown", () => {
  beforeEach(() => {
    vi.useFakeTimers();
  });

  afterEach(() => {
    vi.useRealTimers();
  });

  it("shows days, hours, and minutes when start is 2 days away", () => {
    const now = new Date("2026-03-10T12:00:00.000Z");
    vi.setSystemTime(now);
    const startsAt = new Date(now.getTime() + 2 * 24 * 60 * 60 * 1000 + 3 * 60 * 60 * 1000 + 15 * 60 * 1000).toISOString();

    render(<EventCountdown startsAt={startsAt} />);

    expect(screen.getByText("2d 3h 15m")).toBeInTheDocument();
    expect(screen.queryByText(/-/)).not.toBeInTheDocument();
  });

  it("updates the display after one minute", () => {
    const now = new Date("2026-03-10T12:00:00.000Z");
    vi.setSystemTime(now);
    const startsAt = new Date(now.getTime() + 90 * 60 * 1000).toISOString();

    render(<EventCountdown startsAt={startsAt} />);
    expect(screen.getByText("1h 30m")).toBeInTheDocument();

    act(() => {
      vi.advanceTimersByTime(60_000);
    });

    expect(screen.getByText("1h 29m")).toBeInTheDocument();
  });

  it("shows happening/ended states for past start times, not negative numbers", () => {
    const now = new Date("2026-03-10T12:00:00.000Z");
    vi.setSystemTime(now);

    const happeningStart = new Date(now.getTime() - 30 * 60 * 1000).toISOString();
    const { unmount } = render(<EventCountdown startsAt={happeningStart} />);
    expect(screen.getByText("Happening now")).toBeInTheDocument();
    expect(screen.queryByText(/-/)).not.toBeInTheDocument();
    unmount();

    const endedStart = new Date(now.getTime() - 3 * 60 * 60 * 1000).toISOString();
    render(<EventCountdown startsAt={endedStart} />);
    expect(screen.getByText("Event ended")).toBeInTheDocument();
    expect(screen.queryByText(/-/)).not.toBeInTheDocument();
  });

  it("clears the interval on unmount", () => {
    const clearIntervalSpy = vi.spyOn(globalThis, "clearInterval");
    const now = new Date("2026-03-10T12:00:00.000Z");
    vi.setSystemTime(now);
    const startsAt = new Date(now.getTime() + 60 * 60 * 1000).toISOString();

    const { unmount } = render(<EventCountdown startsAt={startsAt} />);
    unmount();

    expect(clearIntervalSpy).toHaveBeenCalled();
    clearIntervalSpy.mockRestore();
  });
});
