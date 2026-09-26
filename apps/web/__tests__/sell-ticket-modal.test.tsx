import React from "react";
import { describe, expect, it, vi } from "vitest";
import { render, screen, fireEvent } from "@testing-library/react";
import { SellTicketModal } from "../components/wallet/SellTicketModal";

vi.mock("sonner", () => ({
  toast: {
    success: vi.fn(),
    error: vi.fn(),
  },
}));

vi.mock("@stellar/freighter-api", () => ({
  isConnected: vi.fn(async () => true),
  requestAccess: vi.fn(async () => "GSELLERTESTADDRESS123"),
  signTransaction: vi.fn(async () => "signed_xdr"),
}));

vi.mock("../utils/stellar", () => ({
  buildUnsignedResaleTicketTx: vi.fn(async () => ({
    transactionXdr: "mock_xdr",
    listingId: "resale_12345",
    unsigned: true,
  })),
}));

describe("SellTicketModal Component", () => {
  const mockTicket = {
    id: "ticket-101",
    status: "active",
    ticket_tier_id: "tier-1",
    ticket_tier_name: "VIP Ticket",
    ticket_price: "50.00",
    event_id: "evt-1",
    event_title: "Stellar Developer Conference",
    event_location: "San Francisco, CA",
    event_start_time: "2026-10-15T09:00:00Z",
    event_image_url: "/images/event1.png",
    created_at: "2026-08-01T00:00:00Z",
  };

  it("does not render when isOpen is false or ticket is null", () => {
    const { container } = render(
      <SellTicketModal isOpen={false} onClose={vi.fn()} ticket={mockTicket} />
    );
    expect(container.firstChild).toBeNull();
  });

  it("renders modal header and ticket info when open", () => {
    render(
      <SellTicketModal isOpen={true} onClose={vi.fn()} ticket={mockTicket} />
    );

    expect(screen.getByText("List Ticket for Resale")).toBeInTheDocument();
    expect(screen.getByText("Stellar Developer Conference")).toBeInTheDocument();
    expect(screen.getByText("Tier: VIP Ticket")).toBeInTheDocument();
  });

  it("updates price breakdown as resale price is typed", () => {
    render(
      <SellTicketModal isOpen={true} onClose={vi.fn()} ticket={mockTicket} />
    );

    const input = screen.getByLabelText(/Resale Price/i);
    fireEvent.change(input, { target: { value: "100" } });

    expect(screen.getByText("Listing Price:")).toBeInTheDocument();
    expect(screen.getByText("$100.00")).toBeInTheDocument();
    expect(screen.getByText("-$5.00")).toBeInTheDocument();
    expect(screen.getByText("$95.00")).toBeInTheDocument();
  });
});
