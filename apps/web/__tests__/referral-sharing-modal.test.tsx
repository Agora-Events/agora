import { render, screen, fireEvent, waitFor } from "@testing-library/react";
import { describe, it, expect, vi, beforeEach } from "vitest";
import { ReferralSharingModal } from "@/components/affiliates/referral-sharing-modal";

// Mock sonner toast
vi.mock("sonner", () => ({
  toast: { success: vi.fn(), error: vi.fn() },
}));

const mockReferralUrl = "https://agora.app/events/42?ref=demo123";
const mockShareText = "Join me at this event!";

describe("ReferralSharingModal", () => {
  beforeEach(() => {
    vi.clearAllMocks();
  });

  it("renders nothing when isOpen is false", () => {
    const { container } = render(
      <ReferralSharingModal
        isOpen={false}
        onClose={vi.fn()}
        referralUrl={mockReferralUrl}
      />
    );
    expect(container.firstChild).toBeNull();
  });

  it("renders dialog with title when open", () => {
    render(
      <ReferralSharingModal
        isOpen={true}
        onClose={vi.fn()}
        referralUrl={mockReferralUrl}
      />
    );
    const dialog = screen.getByRole("dialog");
    expect(dialog).toBeInTheDocument();
    expect(dialog).toHaveAttribute("aria-modal", "true");
    expect(screen.getByText("Share your referral link")).toBeInTheDocument();
  });

  it("renders WhatsApp and X (Twitter) share links", () => {
    render(
      <ReferralSharingModal
        isOpen={true}
        onClose={vi.fn()}
        referralUrl={mockReferralUrl}
      />
    );

    const whatsappLink = screen.getByText("WhatsApp").closest("a");
    expect(whatsappLink).toHaveAttribute(
      "href",
      expect.stringContaining("https://wa.me/")
    );

    const xLink = screen.getByText("X (Twitter)").closest("a");
    expect(xLink).toHaveAttribute(
      "href",
      expect.stringContaining("https://twitter.com/intent/tweet")
    );
  });

  it("renders WhatsApp and X links with correct referral URL encoded", () => {
    render(
      <ReferralSharingModal
        isOpen={true}
        onClose={vi.fn()}
        referralUrl={mockReferralUrl}
        shareText={mockShareText}
      />
    );

    const whatsappLink = screen.getByText("WhatsApp").closest("a");
    expect(whatsappLink).toHaveAttribute(
      "href",
      expect.stringContaining(encodeURIComponent(mockReferralUrl))
    );
  });

  it("calls onClose when Escape is pressed", () => {
    const onClose = vi.fn();
    render(
      <ReferralSharingModal
        isOpen={true}
        onClose={onClose}
        referralUrl={mockReferralUrl}
      />
    );
    // useFocusTrap listens on document
    fireEvent.keyDown(document, { key: "Escape" });
    expect(onClose).toHaveBeenCalledOnce();
  });

  it("calls onClose when clicking the close button", () => {
    const onClose = vi.fn();
    render(
      <ReferralSharingModal
        isOpen={true}
        onClose={onClose}
        referralUrl={mockReferralUrl}
      />
    );
    const closeButton = screen.getByLabelText("Close sharing options");
    fireEvent.click(closeButton);
    expect(onClose).toHaveBeenCalledOnce();
  });

  it("copies URL to clipboard on Copy Link button click", async () => {
    // Mock clipboard API
    const writeText = vi.fn().mockResolvedValue(undefined);
    Object.assign(navigator, {
      clipboard: { writeText },
    });

    render(
      <ReferralSharingModal
        isOpen={true}
        onClose={vi.fn()}
        referralUrl={mockReferralUrl}
      />
    );

    // On non-native-share browsers, the Copy Link button appears
    const copyBtn = screen.getByText("Copy Link") as HTMLButtonElement;
    fireEvent.click(copyBtn);

    await waitFor(() => {
      expect(writeText).toHaveBeenCalledWith(mockReferralUrl);
    });
  });

  it("has a close button to dismiss the modal", () => {
    render(
      <ReferralSharingModal
        isOpen={true}
        onClose={vi.fn()}
        referralUrl={mockReferralUrl}
      />
    );
    expect(screen.getByLabelText("Close sharing options")).toBeInTheDocument();
  });
});