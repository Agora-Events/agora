import { render, screen } from "@testing-library/react";
import { describe, it, expect, vi, beforeEach } from "vitest";
import "@testing-library/jest-dom";
import React from "react";

const reducedMotion = vi.hoisted(() => ({ value: false }));

// Strip animation-only props so the mocked elements render as plain DOM nodes.
vi.mock("framer-motion", () => {
  const strip = ({
    initial: _initial,
    animate: _animate,
    transition: _transition,
    ...rest
  }: Record<string, unknown>) => rest;
  return {
    motion: {
      div: (props: Record<string, unknown>) => <div {...strip(props)} />,
      span: (props: Record<string, unknown>) => <span {...strip(props)} />,
      path: (props: Record<string, unknown>) => <path {...strip(props)} />,
    },
    useReducedMotion: () => reducedMotion.value,
  };
});

import { PurchaseSuccessAnimation } from "@/components/events/PurchaseSuccessAnimation";

describe("PurchaseSuccessAnimation", () => {
  beforeEach(() => {
    reducedMotion.value = false;
  });

  it("renders nothing until the purchase is confirmed", () => {
    render(<PurchaseSuccessAnimation active={false} />);

    expect(screen.queryByTestId("purchase-success-animation")).not.toBeInTheDocument();
    expect(screen.queryByTestId("purchase-success-confetti")).not.toBeInTheDocument();
  });

  it("plays the checkmark and confetti burst once confirmed", () => {
    render(<PurchaseSuccessAnimation active />);

    expect(screen.getByTestId("purchase-success-animation")).toBeInTheDocument();
    const confetti = screen.getByTestId("purchase-success-confetti");
    expect(confetti).toHaveClass("motion-reduce:hidden");
    expect(confetti.children.length).toBeGreaterThan(0);
  });

  it("shows a static checkmark without confetti when reduced motion is preferred", () => {
    reducedMotion.value = true;
    render(<PurchaseSuccessAnimation active />);

    expect(screen.getByTestId("purchase-success-animation")).toBeInTheDocument();
    expect(screen.queryByTestId("purchase-success-confetti")).not.toBeInTheDocument();
  });
});
