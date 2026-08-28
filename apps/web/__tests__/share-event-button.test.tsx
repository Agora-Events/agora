import { render, screen, fireEvent, waitFor } from "@testing-library/react";
import { expect, describe, it, vi, beforeEach, afterEach } from "vitest";
import { ShareEventButton } from "@/components/events/share-event-button";

vi.mock("sonner", () => ({
  toast: { success: vi.fn(), error: vi.fn() },
}));

import { toast } from "sonner";

const mockedToast = vi.mocked(toast);

describe("ShareEventButton component", () => {
  beforeEach(() => {
    vi.resetAllMocks();
    // Default: no native share, clipboard available
    Object.assign(navigator, {
      share: undefined,
      clipboard: { writeText: vi.fn().mockResolvedValue(undefined) },
    });
    Object.defineProperty(window, "location", {
      writable: true,
      value: { href: "https://agora.example/events/42" },
    });
  });

  afterEach(() => {
    vi.restoreAllMocks();
  });

  it("renders a share button with an accessible label", () => {
    render(<ShareEventButton title="Stellar Asado" />);
    expect(screen.getByRole("button", { name: "Share this event" })).toBeInTheDocument();
    expect(screen.getByText("Share")).toBeInTheDocument();
  });

  it("uses the Web Share API when available", async () => {
    const share = vi.fn().mockResolvedValue(undefined);
    Object.assign(navigator, { share });

    render(<ShareEventButton title="Stellar Asado" />);
    fireEvent.click(screen.getByRole("button", { name: "Share this event" }));

    await waitFor(() => {
      expect(share).toHaveBeenCalledWith({
        title: "Stellar Asado",
        text: "Check out this event: Stellar Asado",
        url: "https://agora.example/events/42",
      });
    });
    expect(mockedToast.success).not.toHaveBeenCalled();
  });

  it("falls back to copying the URL and shows a success toast when share is unavailable", async () => {
    render(<ShareEventButton title="Stellar Asado" />);
    fireEvent.click(screen.getByRole("button", { name: "Share this event" }));

    await waitFor(() => {
      expect(navigator.clipboard.writeText).toHaveBeenCalledWith("https://agora.example/events/42");
    });
    expect(mockedToast.success).toHaveBeenCalledWith("Link copied");
  });

  it("silently ignores a user-cancelled AbortError from the share sheet", async () => {
    const abortError = new DOMException("The user aborted a request.", "AbortError");
    Object.assign(navigator, {
      share: vi.fn().mockRejectedValue(abortError),
      clipboard: { writeText: vi.fn().mockResolvedValue(undefined) },
    });

    render(<ShareEventButton title="Stellar Asado" />);
    fireEvent.click(screen.getByRole("button", { name: "Share this event" }));

    // Let the promise reject and settle
    await waitFor(() => {
      expect(navigator.clipboard.writeText).not.toHaveBeenCalled();
    });
    expect(mockedToast.success).not.toHaveBeenCalled();
    expect(mockedToast.error).not.toHaveBeenCalled();
  });

  it("falls back to clipboard when the share sheet throws a non-Abort error", async () => {
    Object.assign(navigator, {
      share: vi.fn().mockRejectedValue(new Error("not allowed")),
      clipboard: { writeText: vi.fn().mockResolvedValue(undefined) },
    });

    render(<ShareEventButton title="Stellar Asado" />);
    fireEvent.click(screen.getByRole("button", { name: "Share this event" }));

    await waitFor(() => {
      expect(navigator.clipboard.writeText).toHaveBeenCalledWith("https://agora.example/events/42");
    });
    expect(mockedToast.success).toHaveBeenCalledWith("Link copied");
  });
});
