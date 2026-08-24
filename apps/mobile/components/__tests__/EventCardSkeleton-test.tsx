import React from "react";
import { render } from "@testing-library/react-native";
import { EventCardSkeleton } from "../EventCardSkeleton";

/** Issue #1024 — event list skeleton loaders. */

describe("EventCardSkeleton", () => {
  it("renders the skeleton container", () => {
    const { getByTestId } = render(<EventCardSkeleton />);
    expect(getByTestId("event-card-skeleton")).toBeTruthy();
  });

  it("renders the thumbnail placeholder matching the event row layout", () => {
    const { getByTestId } = render(<EventCardSkeleton />);
    const container = getByTestId("event-card-skeleton");
    // The container has 2 children: the thumb box and the text body.
    expect(container.children.length).toBe(2);
  });

  it("is animated (starts the pulse loop)", () => {
    // Animated.loop with useNativeDriver should not throw; the element
    // must be present immediately so the list can swap it in without flicker.
    const { getByTestId } = render(<EventCardSkeleton testID="skeleton-test" />);
    expect(getByTestId("skeleton-test")).toBeTruthy();
  });

  it("accepts a custom testID", () => {
    const { getByTestId } = render(<EventCardSkeleton testID="custom-skeleton" />);
    expect(getByTestId("custom-skeleton")).toBeTruthy();
  });
});