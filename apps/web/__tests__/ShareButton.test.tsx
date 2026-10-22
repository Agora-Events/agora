import { render, screen, waitFor, fireEvent } from "@testing-library/react";
import { describe, it, expect, vi, beforeEach, afterEach } from "vitest";
import { toast } from "sonner";
import ShareButton from "@/components/events/ShareButton";

vi.mock("sonner", () => ({
  toast: { success: vi.fn(), error: vi.fn() },
}));

vi.mock("next-intl", () => ({
  useTranslations: () => (key: string) => (key === "share" ? "Share" : key),
}));

describe("ShareButton", () => {
  const originalShare = navigator.share;
  const originalCanShare = navigator.canShare;
  const originalClipboard = navigator.clipboard;
  const pageUrl = "https://agora.events/events/42";

  beforeEach(() => {
    vi.clearAllMocks();
    Object.defineProperty(window, "location", {
      configurable: true,
      value: { href: pageUrl },
    });
  });

  afterEach(() => {
    Object.defineProperty(navigator, "share", {
      configurable: true,
      value: originalShare,
    });
    Object.defineProperty(navigator, "canShare", {
      configurable: true,
      value: originalCanShare,
    });
    Object.defineProperty(navigator, "clipboard", {
      configurable: true,
      value: originalClipboard,
    });
  });

  it("renders a share button", () => {
    render(<ShareButton title="Stellar Meetup" text="Join us" />);
    expect(screen.getByRole("button", { name: /share/i })).toBeInTheDocument();
  });

  it("calls navigator.share with title, text, and url when available", async () => {
    const share = vi.fn().mockResolvedValue(undefined);
    Object.defineProperty(navigator, "share", {
      configurable: true,
      value: share,
    });

    render(<ShareButton title="Stellar Meetup" text="Join us this Friday" />);
    fireEvent.click(screen.getByRole("button", { name: /share/i }));

    await waitFor(() => {
      expect(share).toHaveBeenCalledWith({
        title: "Stellar Meetup",
        text: "Join us this Friday",
        url: pageUrl,
      });
    });
    expect(toast.success).not.toHaveBeenCalled();
    expect(toast.error).not.toHaveBeenCalled();
  });

  it("falls back to clipboard and shows copied message when share is unavailable", async () => {
    const writeText = vi.fn().mockResolvedValue(undefined);
    Object.defineProperty(navigator, "share", {
      configurable: true,
      value: undefined,
    });
    Object.defineProperty(navigator, "clipboard", {
      configurable: true,
      value: { writeText },
    });

    render(<ShareButton title="Stellar Meetup" text="Join us" />);
    fireEvent.click(screen.getByRole("button", { name: /share/i }));

    await waitFor(() => {
      expect(writeText).toHaveBeenCalledWith(pageUrl);
      expect(toast.success).toHaveBeenCalledWith("Link copied to clipboard");
    });
  });

  it("does not show an error when the user cancels navigator.share", async () => {
    const abortError = new DOMException("Share cancelled", "AbortError");
    const share = vi.fn().mockRejectedValue(abortError);
    Object.defineProperty(navigator, "share", {
      configurable: true,
      value: share,
    });

    render(<ShareButton title="Stellar Meetup" text="Join us" />);
    fireEvent.click(screen.getByRole("button", { name: /share/i }));

    await waitFor(() => {
      expect(share).toHaveBeenCalled();
    });
    expect(toast.error).not.toHaveBeenCalled();
    expect(toast.success).not.toHaveBeenCalled();
  });

  it("shows an error toast if share throws an unexpected error", async () => {
    const generalError = new Error("Failed to share");
    const share = vi.fn().mockRejectedValue(generalError);
    Object.defineProperty(navigator, "share", {
      configurable: true,
      value: share,
    });

    render(<ShareButton title="Stellar Meetup" text="Join us" />);
    fireEvent.click(screen.getByRole("button", { name: /share/i }));

    await waitFor(() => {
      expect(toast.error).toHaveBeenCalledWith("Unable to share link");
    });
  });
});
