import { render, screen, fireEvent } from "@testing-library/react";
import { expect, describe, it, vi } from "vitest";

// Mock next/image to render a plain element (jsdom can't render optimized sources)
vi.mock("next/image", () => ({
  // eslint-disable-next-line @next/next/no-img-element
  default: ({ src, alt }: { src: string; alt: string }) => <img src={src} alt={alt} />,
}));

// Mock layout components so the notifications page can render in isolation
vi.mock("@/components/layout/navbar", () => ({
  Navbar: () => <nav>Navbar</nav>,
}));
vi.mock("@/components/layout/footer", () => ({
  Footer: () => <footer>Footer</footer>,
}));

import NotificationsPage from "@/app/notifications/page";

describe("NotificationsPage empty state", () => {
  it("renders the reusable EmptyState when there are no notifications", () => {
    render(<NotificationsPage />);

    // Initially the page lists mock notifications
    expect(screen.getByText("Ticket Confirmed! 🎉")).toBeInTheDocument();

    // Clear all notifications to trigger the empty state
    fireEvent.click(screen.getByText("Clear all"));

    // The reusable EmptyState component (with data-testid="empty-state") is shown
    expect(screen.getByTestId("empty-state")).toBeInTheDocument();
    expect(screen.getByText("You're all caught up!")).toBeInTheDocument();
    expect(
      screen.getByText(
        "You don't have any new notifications at the moment. Check back later for updates on your events and tickets."
      )
    ).toBeInTheDocument();
  });

  it("links back to the discover page from the empty state CTA", () => {
    render(<NotificationsPage />);
    fireEvent.click(screen.getByText("Clear all"));

    const cta = screen.getByRole("link", { name: /discover events/i });
    expect(cta).toBeInTheDocument();
    expect(cta).toHaveAttribute("href", "/discover");
  });

  it("shows the notification icon in the empty state", () => {
    render(<NotificationsPage />);
    fireEvent.click(screen.getByText("Clear all"));

    expect(screen.getByAltText("No notifications")).toBeInTheDocument();
  });
});
