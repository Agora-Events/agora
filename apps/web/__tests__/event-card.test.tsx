import { render, screen, fireEvent } from "@testing-library/react";
import { expect, afterEach, describe, it, vi } from "vitest";
import { cleanup } from "@testing-library/react";
import "@testing-library/jest-dom";
import { EventCard } from "@/components/events/event-card";
import { useRouter } from "next/navigation";
import Image from "next/image";

// Mock next/image
vi.mock("next/image", () => ({
  default: ({ src, alt, width, height }: { src: string; alt: string; width: number; height: number }) => (
    <Image src={src} alt={alt} width={width} height={height} />
  ),
}));

// Mock next/navigation
vi.mock("next/navigation", () => ({
  useRouter: vi.fn(() => ({ push: vi.fn() })),
}));

const mockEvent = {
  id: "1",
  title: "Test Event",
  date: "Mon, 1 Jan, 12:00",
  location: "Test Location",
  price: "10.00",
  imageUrl: "/test-image.jpg",
};

describe("EventCard", () => {
  afterEach(() => {
    cleanup();
  });

  it("renders correctly with required props", () => {
    render(<EventCard {...mockEvent} />);
    
    // Check that title is rendered
    expect(screen.getByText(/Test Event/i)).toBeInTheDocument();
    
    // Check that date is rendered
    expect(screen.getByText(/Mon, 1 Jan, 12:00/i)).toBeInTheDocument();
    
    // Check that location is rendered
    expect(screen.getByText(/Test Location/i)).toBeInTheDocument();
    
    // Check that price is rendered
    expect(screen.getByText(/\$10\.00/i)).toBeInTheDocument();
  });

  it("renders free price correctly", () => {
    const freeEvent = { ...mockEvent, price: "free" };
    render(<EventCard {...freeEvent} />);
    
    // Check that "Free" is rendered instead of "$0.00"
    expect(screen.getByText(/Free/i)).toBeInTheDocument();
  });

  it("handles click and navigates to event page", () => {
    const mockPush = vi.fn();
    vi.mocked(useRouter).mockReturnValue({ push: mockPush });
    
    render(<EventCard {...mockEvent} />);
    
    // Click on the card (which is a link)
    const cardLink = screen.getByRole("link");
    fireEvent.click(cardLink);
    
    // Check that navigation was called
    expect(mockPush).toHaveBeenCalledWith("/events/1");
  });
});