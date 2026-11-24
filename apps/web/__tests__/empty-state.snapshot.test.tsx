import { render } from "@testing-library/react";
import { expect, describe, it } from "vitest";
import { EmptyState } from "@/components/ui/empty-state";

// Snapshot coverage for EmptyState (Issue #1565). Behavioural assertions live
// in empty-state.test.tsx; these guard the rendered markup against unintended
// visual regressions across each prop combination.
describe("EmptyState snapshots", () => {
  it("matches snapshot with only a title (default illustration, no CTA)", () => {
    const { container } = render(<EmptyState title="No events found" />);
    expect(container).toMatchSnapshot();
  });

  it("matches snapshot with a custom title and description", () => {
    const { container } = render(
      <EmptyState
        title="No upcoming events"
        description="Check back soon for new events in your area."
      />
    );
    expect(container).toMatchSnapshot();
  });

  it("matches snapshot with a custom illustration", () => {
    const { container } = render(
      <EmptyState
        title="Nothing saved yet"
        message="Events you bookmark will appear here."
        illustrationSrc="/icons/empty-state-bg.svg"
      />
    );
    expect(container).toMatchSnapshot();
  });

  it("matches snapshot with an action link CTA", () => {
    const { container } = render(
      <EmptyState
        title="No events found"
        description="Try a different category or come back later."
        action={{ label: "Create an Event", href: "/events/create" }}
      />
    );
    expect(container).toMatchSnapshot();
  });

  it("matches snapshot with an action button CTA and custom icon", () => {
    const { container } = render(
      <EmptyState
        icon={<span aria-hidden="true">🔍</span>}
        title="No results"
        description="Try another search."
        action={{ label: "Clear filters", onClick: () => undefined }}
      />
    );
    expect(container).toMatchSnapshot();
  });

  it("matches snapshot with legacy ctaLabel/ctaLink props", () => {
    const { container } = render(
      <EmptyState
        title="No tickets yet"
        message="Tickets you buy will show up here."
        ctaLabel="Browse events"
        ctaLink="/discover"
      />
    );
    expect(container).toMatchSnapshot();
  });
});
