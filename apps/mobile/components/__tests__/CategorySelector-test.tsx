import React from "react";
import { render, fireEvent } from "@testing-library/react-native";
import CategorySelector, { EVENT_CATEGORIES } from "../CategorySelector";

/** Issue #1014 — category chips. */

describe("CategorySelector", () => {
  it("renders a chip for every category", () => {
    const { getByTestId } = render(<CategorySelector selected={null} onSelect={jest.fn()} />);
    for (const category of EVENT_CATEGORIES) {
      expect(getByTestId(`category-chip-${category}`)).toBeTruthy();
    }
  });

  it("calls the parent callback with the category id", () => {
    const onSelect = jest.fn();
    const { getByTestId } = render(<CategorySelector selected={null} onSelect={onSelect} />);

    fireEvent.press(getByTestId("category-chip-Music"));

    expect(onSelect).toHaveBeenCalledWith("Music");
  });

  it("clears the filter when the active chip is pressed again", () => {
    const onSelect = jest.fn();
    const { getByTestId } = render(<CategorySelector selected="Tech" onSelect={onSelect} />);

    fireEvent.press(getByTestId("category-chip-Tech"));

    // Otherwise there is no way back to "all" without a separate reset control.
    expect(onSelect).toHaveBeenCalledWith(null);
  });

  it("marks the active chip as selected for assistive tech", () => {
    const { getByTestId } = render(<CategorySelector selected="Sports" onSelect={jest.fn()} />);

    expect(getByTestId("category-chip-Sports").props.accessibilityState.selected).toBe(true);
    expect(getByTestId("category-chip-Arts").props.accessibilityState.selected).toBe(false);
  });

  it("accepts a custom category list", () => {
    const { getByTestId, queryByTestId } = render(
      <CategorySelector selected={null} onSelect={jest.fn()} categories={["Food"]} />,
    );

    expect(getByTestId("category-chip-Food")).toBeTruthy();
    expect(queryByTestId("category-chip-Music")).toBeNull();
  });
});
