import { render, screen } from "@testing-library/react";
import { expect, describe, it } from "vitest";
import { EmptyState } from "@/components/ui/empty-state";

// next/image is mocked in vitest.setup.ts / jsdom — works fine with static props
describe("EmptyState component", () => {
  it("renders when the events list is empty", () => {
    const events: unknown[] = [];

    // Simulate the conditional used in discover/page.tsx
    if (events.length === 0) {
      render(
        <EmptyState 
          title="No events found"
          description="Try a different category or come back later."
          action={{
            label: "Create an Event",
            href: "/events/create",
          }}
        />
      );
    }

    expect(screen.getByTestId("empty-state")).toBeInTheDocument();
    expect(screen.getByText("No events found")).toBeInTheDocument();
    expect(
      screen.getByText("Try a different category or come back later.")
    ).toBeInTheDocument();
  });

  it("renders the action link when an href action is provided", () => {
    render(
      <EmptyState 
        title="No events"
        description="Nothing here yet."
        action={{
          label: "Create an Event",
          href: "/events/create",
        }}
      />
    );

    const cta = screen.getByRole("link", { name: /create an event/i });
    expect(cta).toBeInTheDocument();
    expect(cta).toHaveAttribute("href", "/events/create");
  });

  it("does not render an action link when no action is provided", () => {
    render(<EmptyState title="No events" description="Nothing here yet." />);

    expect(screen.queryByRole("link")).not.toBeInTheDocument();
  });

  it("does not render at all when events array is non-empty", () => {
    const events = [{ id: 1 }];
    const { container } = render(
      <>
        {events.length === 0 && (
          <EmptyState title="No events" description="Nothing here yet." />
        )}
      </>
    );

    expect(container.querySelector("[data-testid='empty-state']")).toBeNull();
  });

  it("renders title and description as accessible text", () => {
    render(
      <EmptyState
        title="No upcoming events"
        description="Check back soon for new events in your area."
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
