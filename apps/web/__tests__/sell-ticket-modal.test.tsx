import { render, screen, fireEvent, waitFor } from "@testing-library/react";
import { describe, it, expect, vi, beforeEach } from "vitest";
import { SellTicketModal } from "@/components/events/sell-ticket-modal";

// Mock sonner toast
vi.mock("sonner", () => ({
  toast: { success: vi.fn(), error: vi.fn() },
}));

const mockEvent = {
  id: 42,
  title: "Stellar Meetup",
  date: "Sat, 27 Jun",
  location: "Buenos Aires",
  price: "49.00",
};

describe("SellTicketModal", () => {
  beforeEach(() => {
    vi.clearAllMocks();
  });

  it("renders nothing when isOpen is false", () => {
    const { container } = render(
      <SellTicketModal
        isOpen={false}
        onClose={vi.fn()}
        event={mockEvent}
        ticketQuantity={1}
      />
    );
    expect(container.firstChild).toBeNull();
  });

  it("renders dialog with title when open", () => {
    render(
      <SellTicketModal
        isOpen={true}
        onClose={vi.fn()}
        event={mockEvent}
        ticketQuantity={1}
      />
    );
    const dialog = screen.getByRole("dialog");
    expect(dialog).toBeInTheDocument();
    expect(dialog).toHaveAttribute("aria-modal", "true");
    expect(dialog).toHaveAttribute("aria-labelledby", "sell-ticket-title");
    expect(screen.getByText("Sell Your Ticket")).toBeInTheDocument();
  });

  it("displays ticket summary with event details", () => {
    render(
      <SellTicketModal
        isOpen={true}
        onClose={vi.fn()}
        event={mockEvent}
        ticketQuantity={2}
      />
    );
    expect(screen.getByText("Ticket Summary")).toBeInTheDocument();
    expect(screen.getByText("Stellar Meetup")).toBeInTheDocument();
    expect(screen.getByText("Sat, 27 Jun")).toBeInTheDocument();
    expect(screen.getByText("Buenos Aires")).toBeInTheDocument();
    expect(screen.getByText("2 tickets")).toBeInTheDocument();
    expect(screen.getByText("$49.00")).toBeInTheDocument();
  });

  it("shows Free for free events", () => {
    render(
      <SellTicketModal
        isOpen={true}
        onClose={vi.fn()}
        event={{ ...mockEvent, price: "Free" }}
        ticketQuantity={1}
      />
    );
    expect(screen.getByText("Free")).toBeInTheDocument();
  });

  it("shows validation error for empty price on submit", () => {
    render(
      <SellTicketModal
        isOpen={true}
        onClose={vi.fn()}
        event={mockEvent}
        ticketQuantity={1}
      />
    );
    fireEvent.click(screen.getByText("List for $0.00"));
    expect(screen.getByRole("alert")).toHaveTextContent(
      "Resale price is required"
    );
  });

  it("shows validation error for price below minimum", () => {
    render(
      <SellTicketModal
        isOpen={true}
        onClose={vi.fn()}
        event={mockEvent}
        ticketQuantity={1}
      />
    );
    const input = screen.getByLabelText("Your Resale Price") as HTMLInputElement;
    fireEvent.change(input, { target: { value: "0" } });
    fireEvent.click(screen.getByText("List for $0.00"));
    expect(screen.getByRole("alert")).toHaveTextContent(
      "Minimum resale price"
    );
  });

  it("filters out invalid characters and >2 decimal places", () => {
    render(
      <SellTicketModal
        isOpen={true}
        onClose={vi.fn()}
        event={mockEvent}
        ticketQuantity={1}
      />
    );
    const input = screen.getByLabelText("Your Resale Price") as HTMLInputElement;
    // The input filter rejects >2 decimal places and non-numeric characters
    fireEvent.change(input, { target: { value: "12.345" } });
    expect(input).toHaveValue("");

    fireEvent.change(input, { target: { value: "12.34" } });
    expect(input).toHaveValue("12.34");
  });

  it("displays transaction summary with platform fee when price is valid", () => {
    render(
      <SellTicketModal
        isOpen={true}
        onClose={vi.fn()}
        event={mockEvent}
        ticketQuantity={1}
      />
    );
    const input = screen.getByLabelText("Your Resale Price") as HTMLInputElement;
    fireEvent.change(input, { target: { value: "100" } });

    expect(screen.getByText("Transaction Summary")).toBeInTheDocument();
    expect(screen.getByText("-$5.00")).toBeInTheDocument(); // 5% fee
    expect(screen.getByText("$95.00")).toBeInTheDocument(); // net payout
  });

  it("calls onClose when Escape is pressed", () => {
    const onClose = vi.fn();
    render(
      <SellTicketModal
        isOpen={true}
        onClose={onClose}
        event={mockEvent}
        ticketQuantity={1}
      />
    );
    fireEvent.keyDown(document, { key: "Escape" });
    expect(onClose).toHaveBeenCalledOnce();
  });

  it("calls onClose when clicking the close button", () => {
    const onClose = vi.fn();
    render(
      <SellTicketModal
        isOpen={true}
        onClose={onClose}
        event={mockEvent}
        ticketQuantity={1}
      />
    );
    const closeButton = screen.getByLabelText("Close sell ticket modal");
    fireEvent.click(closeButton);
    expect(onClose).toHaveBeenCalledOnce();
  });

  it("calls onClose when clicking Cancel", () => {
    const onClose = vi.fn();
    render(
      <SellTicketModal
        isOpen={true}
        onClose={onClose}
        event={mockEvent}
        ticketQuantity={1}
      />
    );
    fireEvent.click(screen.getByText("Cancel"));
    expect(onClose).toHaveBeenCalledOnce();
  });

  it("submits successfully with valid price and shows toast", async () => {
    // Mock fetch success
    global.fetch = vi.fn().mockResolvedValue({
      ok: true,
      json: vi.fn().mockResolvedValue({}),
    }) as unknown as typeof fetch;

    const onClose = vi.fn();
    render(
      <SellTicketModal
        isOpen={true}
        onClose={onClose}
        event={mockEvent}
        ticketQuantity={1}
      />
    );

    const input = screen.getByLabelText("Your Resale Price") as HTMLInputElement;
    fireEvent.change(input, { target: { value: "75" } });
    fireEvent.click(screen.getByText("List for $75.00"));

    await waitFor(() => {
      expect(onClose).toHaveBeenCalledOnce();
    });

    expect(fetch).toHaveBeenCalledWith(
      "/api/v1/events/42/resale",
      expect.objectContaining({
        method: "POST",
        body: JSON.stringify({ ticketQuantity: 1, resalePrice: 75 }),
      })
    );
  });

  it("shows error toast when submission fails", async () => {
    // Mock fetch rejection
    global.fetch = vi.fn().mockRejectedValue(new Error("network")) as unknown as typeof fetch;
    const { toast } = await import("sonner");

    render(
      <SellTicketModal
        isOpen={true}
        onClose={vi.fn()}
        event={mockEvent}
        ticketQuantity={1}
      />
    );

    const input = screen.getByLabelText("Your Resale Price") as HTMLInputElement;
    fireEvent.change(input, { target: { value: "50" } });
    fireEvent.click(screen.getByText("List for $50.00"));

    await waitFor(() => {
      expect(toast.error).toHaveBeenCalled();
    });
  });

  it("resets form state when reopened", () => {
    const { rerender } = render(
      <SellTicketModal
        isOpen={true}
        onClose={vi.fn()}
        event={mockEvent}
        ticketQuantity={1}
      />
    );

    const input = screen.getByLabelText("Your Resale Price") as HTMLInputElement;
    fireEvent.change(input, { target: { value: "60" } });
    expect(input).toHaveValue("60");

    rerender(
      <SellTicketModal
        isOpen={false}
        onClose={vi.fn()}
        event={mockEvent}
        ticketQuantity={1}
      />
    );
    rerender(
      <SellTicketModal
        isOpen={true}
        onClose={vi.fn()}
        event={mockEvent}
        ticketQuantity={1}
      />
    );

    expect(screen.getByLabelText("Your Resale Price")).toHaveValue("");
  });
});