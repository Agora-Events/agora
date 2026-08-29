import React from "react";
import { describe, expect, it, vi } from "vitest";
import { render, screen, fireEvent } from "@testing-library/react";
import { EventPreviewBottomSheet } from "../components/events/EventPreviewBottomSheet";

describe("EventPreviewBottomSheet Component", () => {
  const mockEvent = {
    id: "evt-99",
    title: "Stellar Builders Meetup",
    date: "Sep 12, 2026",
    location: "San Francisco, CA",
    price: "$25.00",
    imageUrl: "/images/event.png",
    category: "Workshop",
  };

  it("does not render when isOpen is false or event is null", () => {
    const { container } = render(
      <EventPreviewBottomSheet isOpen={false} onClose={vi.fn()} event={mockEvent} />
    );
    expect(container.firstChild).toBeNull();
  });

  it("renders event details and CTA button when open", () => {
    render(
      <EventPreviewBottomSheet isOpen={true} onClose={vi.fn()} event={mockEvent} />
    );

    expect(screen.getByText("Stellar Builders Meetup")).toBeInTheDocument();
    expect(screen.getByText("Sep 12, 2026")).toBeInTheDocument();
    expect(screen.getByText("San Francisco, CA")).toBeInTheDocument();
    expect(screen.getByText("$25.00")).toBeInTheDocument();
    expect(screen.getByText("View Event Details →")).toBeInTheDocument();
  });

  it("calls onClose when close button is clicked", () => {
    const onCloseMock = vi.fn();
    render(
      <EventPreviewBottomSheet isOpen={true} onClose={onCloseMock} event={mockEvent} />
    );

    const closeBtn = screen.getByLabelText(/Close bottom sheet/i);
    fireEvent.click(closeBtn);
    expect(onCloseMock).toHaveBeenCalledTimes(1);
  });
});
