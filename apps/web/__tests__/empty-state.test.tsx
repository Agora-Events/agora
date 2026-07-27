import { render, screen } from "@testing-library/react";
import { expect, describe, it } from "vitest";
import { EmptyState } from "@/components/ui/EmptyState";

// next/image is mocked in vitest.setup.ts / jsdom — works fine with static props
describe("EmptyState component", () => {
  it("renders when the events list is empty", () => {
    const events: unknown[] = [];

    // Simulate the conditional used in discover/page.tsx
    if (events.length === 0) {
      render(
        <EmptyState
          title="No events found"
          message="Try a different category or come back later."
          ctaLabel="Create an Event"
          ctaLink="/events/create"
        />
      );
    }

    expect(screen.getByTestId("empty-state")).toBeInTheDocument();
    expect(screen.getByText("No events found")).toBeInTheDocument();
    expect(
      screen.getByText("Try a different category or come back later.")
    ).toBeInTheDocument();
  });

  it("renders the CTA link when ctaLabel and ctaLink are provided", () => {
    render(
      <EmptyState
        title="No events"
        message="Nothing here yet."
        ctaLabel="Create an Event"
        ctaLink="/events/create"
      />
    );

    const cta = screen.getByRole("link", { name: /create an event/i });
    expect(cta).toBeInTheDocument();
    expect(cta).toHaveAttribute("href", "/events/create");
  });

  it("does NOT render a CTA when ctaLabel/ctaLink are omitted", () => {
    render(<EmptyState title="No events" message="Nothing here yet." />);

    expect(screen.queryByRole("link")).not.toBeInTheDocument();
  });

  it("does not render at all when events array is non-empty", () => {
    const events = [{ id: 1 }];
    const { container } = render(
      <>
        {events.length === 0 && (
          <EmptyState title="No events" message="Nothing here yet." />
        )}
      </>
    );

    expect(container.querySelector("[data-testid='empty-state']")).toBeNull();
  });

  it("renders title and message as accessible text", () => {
    render(
      <EmptyState
        title="No upcoming events"
        message="Check back soon for new events in your area."
      />
    );

    expect(
      screen.getByRole("heading", { name: /no upcoming events/i })
    ).toBeInTheDocument();
    expect(
      screen.getByText("Check back soon for new events in your area.")
    ).toBeInTheDocument();
  });
});
