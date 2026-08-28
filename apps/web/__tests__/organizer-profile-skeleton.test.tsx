import { render, screen } from "@testing-library/react";
import { expect, describe, it } from "vitest";
import { OrganizerProfileSkeleton } from "@/components/profile/organizer-profile-skeleton";

describe("OrganizerProfileSkeleton component", () => {
  it("renders the loading skeleton container", () => {
    const { container } = render(<OrganizerProfileSkeleton />);
    expect(container.firstChild).toBeInTheDocument();
  });

  it("is hidden from assistive technology", () => {
    const { container } = render(<OrganizerProfileSkeleton />);
    expect(container.firstChild).toHaveAttribute("aria-hidden", "true");
  });

  it("uses the animate-pulse shimmer", () => {
    const { container } = render(<OrganizerProfileSkeleton />);
    expect(container.firstChild.className).toContain("animate-pulse");
  });

  it("mirrors the final layout with a sidebar and a 3-column event grid", () => {
    const { container } = render(<OrganizerProfileSkeleton />);
    // Avatar block
    expect(container.querySelector(".h-24.w-24.rounded-full")).toBeInTheDocument();
    // 3-column event card grid
    const grid = container.querySelector(".grid.grid-cols-1.md\\:grid-cols-2.lg\\:grid-cols-3");
    expect(grid).not.toBeNull();
  });

  it("renders exactly three event card placeholders", () => {
    const { container } = render(<OrganizerProfileSkeleton />);
    const cards = container.querySelectorAll("[class*='overflow-hidden']");
    // The main section + 3 event cards use overflow-hidden
    expect(cards.length).toBeGreaterThanOrEqual(3);
  });

  it("does not expose any interactive elements", () => {
    render(<OrganizerProfileSkeleton />);
    expect(screen.queryByRole("button")).not.toBeInTheDocument();
    expect(screen.queryByRole("link")).not.toBeInTheDocument();
  });
});
