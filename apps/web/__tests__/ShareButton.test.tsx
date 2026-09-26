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

const mockUseIsMobile = vi.fn(() => false);

vi.mock("@/hooks/useIsMobile", () => ({
  default: () => mockUseIsMobile(),
}));

describe("ShareButton", () => {
  const originalShare = navigator.share;
  const originalClipboard = navigator.clipboard;
  const pageUrl = "https://agora.events/events/42";

  beforeEach(() => {
    vi.clearAllMocks();
    mockUseIsMobile.mockReturnValue(false);
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
    Object.defineProperty(navigator, "clipboard", {
      configurable: true,
      value: originalClipboard,
    });
  });

  it("renders a share button", () => {
    render(<ShareButton title="Stellar Meetup" text="Join us" />);
    expect(screen.getByRole("button", { name: /share/i })).toBeInTheDocument();
  });

  it("calls navigator.share with title and text when available on mobile", async () => {
    const share = vi.fn().mockResolvedValue(undefined);
    mockUseIsMobile.mockReturnValue(true);
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
    mockUseIsMobile.mockReturnValue(true);
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
});
