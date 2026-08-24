import { render, screen, fireEvent } from "@testing-library/react";
import { describe, it, expect, vi } from "vitest";
import { EventPreviewBottomSheet } from "@/components/events/event-preview-bottom-sheet";

vi.mock("next/image", () => ({
  default: ({ src, alt, ...props }: { src: string; alt: string; [key: string]: unknown }) => (
    // eslint-disable-next-line @next/next/no-img-element
    <img src={src} alt={alt} {...props} />
  ),
}));

const mockEvent = {
  id: 1,
  title: "Tech Conference 2026",
  date: "Sat, 15 Aug 2026",
  venue: "San Francisco, CA",
  imageUrl: "https://example.com/event.jpg",
};

describe("EventPreviewBottomSheet", () => {
  it("renders event details when open", () => {
    render(
      <EventPreviewBottomSheet
        isOpen={true}
        onClose={vi.fn()}
        event={mockEvent}
      />
    );

    expect(screen.getByText("Tech Conference 2026")).toBeInTheDocument();
    expect(screen.getByText("Sat, 15 Aug 2026")).toBeInTheDocument();
    expect(screen.getByText("San Francisco, CA")).toBeInTheDocument();
  });

  it("has role=dialog and aria-modal when open", () => {
    render(
      <EventPreviewBottomSheet
        isOpen={true}
        onClose={vi.fn()}
        event={mockEvent}
      />
    );

    const dialog = screen.getByRole("dialog");
    expect(dialog).toBeInTheDocument();
    expect(dialog).toHaveAttribute("aria-modal", "true");
  });

  it("does not render when isOpen is false", () => {
    render(
      <EventPreviewBottomSheet
        isOpen={false}
        onClose={vi.fn()}
        event={mockEvent}
      />
    );

    expect(screen.queryByRole("dialog")).not.toBeInTheDocument();
  });

  it("does not render when event is null", () => {
    render(
      <EventPreviewBottomSheet
        isOpen={true}
        onClose={vi.fn()}
        event={null}
      />
    );

    expect(screen.queryByRole("dialog")).not.toBeInTheDocument();
  });

  it("contains a link to the event details page", () => {
    render(
      <EventPreviewBottomSheet
        isOpen={true}
        onClose={vi.fn()}
        event={mockEvent}
      />
    );

    const link = screen.getByRole("link", { name: /view event details/i });
    expect(link).toBeInTheDocument();
    expect(link).toHaveAttribute("href", "/events/1");
  });

  it("calls onClose when close button is clicked", () => {
    const onClose = vi.fn();
    render(
      <EventPreviewBottomSheet
        isOpen={true}
        onClose={onClose}
        event={mockEvent}
      />
    );

    const closeButton = screen.getByLabelText("Close preview");
    fireEvent.click(closeButton);
    expect(onClose).toHaveBeenCalledOnce();
  });

  it("calls onClose when Escape is pressed", () => {
    const onClose = vi.fn();
    render(
      <EventPreviewBottomSheet
        isOpen={true}
        onClose={onClose}
        event={mockEvent}
      />
    );

    fireEvent.keyDown(document, { key: "Escape" });
    expect(onClose).toHaveBeenCalled();
  });

  it("renders the event image when imageUrl is provided", () => {
    render(
      <EventPreviewBottomSheet
        isOpen={true}
        onClose={vi.fn()}
        event={mockEvent}
      />
    );

    const img = screen.getByAltText("Tech Conference 2026");
    expect(img).toBeInTheDocument();
    expect(img).toHaveAttribute("src", "https://example.com/event.jpg");
  });

  it("renders without image when imageUrl is not provided", () => {
    const eventWithoutImage = { ...mockEvent, imageUrl: undefined };
    render(
      <EventPreviewBottomSheet
        isOpen={true}
        onClose={vi.fn()}
        event={eventWithoutImage}
      />
    );

    expect(screen.getByText("Tech Conference 2026")).toBeInTheDocument();
    expect(screen.queryByAltText("Tech Conference 2026")).not.toBeInTheDocument();
  });
});