import { render, screen, act } from "@testing-library/react";
import { describe, it, expect, vi, beforeEach, afterEach } from "vitest";
import { MapDiscoveryPage } from "../components/events/map-discovery-page";

// Mock Navbar as a lightweight stub (full render causes OOM with framer-motion)
vi.mock("@/components/layout/navbar", () => ({
  Navbar: () => <nav data-testid="navbar" />,
}));

vi.mock("@/components/layout/footer", () => ({
  Footer: () => <footer data-testid="footer" />,
}));

// LoadingBar uses next/navigation hooks — mock them
vi.mock("@/components/ui/loading-bar", () => ({
  default: () => <div data-testid="loading-bar" />,
}));

describe("MapDiscoveryPage", () => {
  beforeEach(() => {
    vi.useFakeTimers();
  });

  afterEach(() => {
    vi.useRealTimers();
  });

  it("shows a loading state initially", () => {
    render(<MapDiscoveryPage />);

    expect(screen.getByText(/loading map data/i)).toBeInTheDocument();
    expect(
      screen.getByRole("region", { name: /event discovery map/i })
    ).toBeInTheDocument();
    expect(screen.getByTestId("navbar")).toBeInTheDocument();
    expect(screen.getByTestId("footer")).toBeInTheDocument();
  });

  it("renders the page heading and description", () => {
    render(<MapDiscoveryPage />);

    expect(
      screen.getByRole("heading", { name: /explore events on the map/i })
    ).toBeInTheDocument();
    expect(
      screen.getByText(/discover events happening near you/i)
    ).toBeInTheDocument();
  });

  it("shows the map placeholder once loading completes", () => {
    render(<MapDiscoveryPage />);

    act(() => {
      vi.advanceTimersByTime(900);
    });

    expect(screen.getByTestId("map-placeholder")).toBeInTheDocument();
    expect(screen.getByText(/map ready to explore/i)).toBeInTheDocument();
  });
});