import { render, screen, fireEvent, waitFor } from "@testing-library/react";
import { describe, it, expect, vi, beforeEach } from "vitest";
import { TicketModal } from "@/components/events/TicketModal";

const mockEvent = {
  id: "evt-123",
  title: "Stellar Developer Summit",
  price: "$100.00",
  location: "San Francisco, CA",
  date: "Nov 15, 2026",
  availableQuantity: 50,
};

describe("TicketModal Promo Code Flow", () => {
  beforeEach(() => {
    vi.clearAllMocks();
    global.fetch = vi.fn();
  });

  it("applies a percentage discount successfully", async () => {
    (global.fetch as any).mockResolvedValueOnce({
      ok: true,
      json: async () => ({
        valid: true,
        discountPercentage: 20,
        discountHash: "0xabc123hash",
      }),
    });

    render(
      <TicketModal
        isOpen={true}
        onClose={vi.fn()}
        event={mockEvent}
        initialQuantity={1}
      />
    );

    const input = screen.getByPlaceholderText(/enter code/i);
    const applyButton = screen.getByRole("button", { name: /apply/i });

    fireEvent.change(input, { target: { value: "SAVE20" } });
    fireEvent.click(applyButton);

    await waitFor(() => {
      expect(screen.getByText(/code "SAVE20" applied: 20% OFF/i)).toBeInTheDocument();
      expect(screen.getByText("-$20.00")).toBeInTheDocument();
      expect(screen.getByText("$80.00")).toBeInTheDocument();
    });
  });

  it("shows an error message when the promo code is invalid", async () => {
    (global.fetch as any).mockResolvedValueOnce({
      ok: false,
      json: async () => ({
        valid: false,
        message: "Code expired or not found",
      }),
    });

    render(
      <TicketModal
        isOpen={true}
        onClose={vi.fn()}
        event={mockEvent}
        initialQuantity={1}
      />
    );

    const input = screen.getByPlaceholderText(/enter code/i);
    const applyButton = screen.getByRole("button", { name: /apply/i });

    fireEvent.change(input, { target: { value: "EXPIRED" } });
    fireEvent.click(applyButton);

    await waitFor(() => {
      expect(screen.getByText(/code expired or not found/i)).toBeInTheDocument();
    });
  });
});