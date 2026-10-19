import { render, screen } from "@testing-library/react";
import { describe, expect, it } from "vitest";
import { OrganizerProfileSkeleton } from "@/components/profile/organizer-profile-skeleton";

describe("OrganizerProfileSkeleton", () => {
  it("renders an aria-busy loading region with hidden skeleton blocks", () => {
    render(<OrganizerProfileSkeleton />);

    const region = screen.getByTestId("organizer-profile-skeleton");
    expect(region).toHaveAttribute("aria-busy", "true");
    expect(region).toHaveAttribute("aria-live", "polite");

    const hiddenSkeleton = region.querySelector('[aria-hidden="true"]');
    expect(hiddenSkeleton).not.toBeNull();
    expect(hiddenSkeleton?.querySelectorAll("div").length).toBeGreaterThan(0);
  });
});
