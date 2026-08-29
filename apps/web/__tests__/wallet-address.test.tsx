import { render, screen, fireEvent, waitFor } from "@testing-library/react";
import { describe, it, expect, vi, beforeEach } from "vitest";
import { WalletAddress } from "@/components/ui/wallet-address";
import { toast } from "sonner";

vi.mock("sonner", () => ({
  toast: { success: vi.fn(), error: vi.fn() },
}));

const FULL_ADDRESS = "GDAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAA";

describe("WalletAddress", () => {
  beforeEach(() => {
    vi.clearAllMocks();
    Object.assign(navigator, {
      clipboard: {
        writeText: vi.fn().mockResolvedValue(undefined),
      },
    });
  });

  it("truncates long Stellar addresses to first 4 and last 4 characters", () => {
    render(<WalletAddress address={FULL_ADDRESS} />);

    expect(screen.getByLabelText(FULL_ADDRESS)).toHaveTextContent("GDAA…AAAA");
    expect(screen.queryByText(FULL_ADDRESS)).not.toBeInTheDocument();
  });

  it("renders the full string when the address is shorter than 12 characters", () => {
    render(<WalletAddress address="GDAA" />);

    expect(screen.getByLabelText("GDAA")).toHaveTextContent("GDAA");
  });

  it("exposes the full address via title and copies it to the clipboard", async () => {
    render(<WalletAddress address={FULL_ADDRESS} />);

    expect(screen.getByLabelText(FULL_ADDRESS)).toHaveAttribute("title", FULL_ADDRESS);

    fireEvent.click(screen.getByRole("button", { name: `Copy wallet address ${FULL_ADDRESS}` }));

    await waitFor(() => {
      expect(navigator.clipboard.writeText).toHaveBeenCalledWith(FULL_ADDRESS);
      expect(toast.success).toHaveBeenCalledWith("Copied");
    });
  });
});
