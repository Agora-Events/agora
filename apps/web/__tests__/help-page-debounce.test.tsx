import { render, screen, fireEvent } from "@testing-library/react";
import { expect, describe, it, beforeEach, afterEach, vi } from "vitest";
import { act } from "@testing-library/react";

// Mock next/image to render a plain element (jsdom can't render optimized sources)
vi.mock("next/image", () => ({
  // eslint-disable-next-line @next/next/no-img-element
  default: ({ src, alt }: { src: string; alt: string }) => <img src={src} alt={alt} />,
}));

// Mock layout components so the help page can render in isolation
vi.mock("@/components/layout/navbar", () => ({
  Navbar: () => <nav>Navbar</nav>,
}));
vi.mock("@/components/layout/footer", () => ({
  Footer: () => <footer>Footer</footer>,
}));

import HelpCenterPage from "@/app/help/page";

describe("HelpCenterPage debounced search", () => {
  beforeEach(() => {
    vi.useFakeTimers();
  });

  afterEach(() => {
    vi.useRealTimers();
  });

  it("keeps the raw input responsive while typing", () => {
    render(<HelpCenterPage />);
    const input = screen.getByLabelText("Search help articles");
    fireEvent.change(input, { target: { value: "Paym" } });
    expect(input).toHaveValue("Paym");
  });

  it("shows Searching… hint while typing, then updates results after debounce", () => {
    render(<HelpCenterPage />);
    const input = screen.getByLabelText("Search help articles");

    // Before typing: heading is "Browse Topics"
    expect(screen.getByText("Browse Topics")).toBeInTheDocument();

    fireEvent.change(input, { target: { value: "Paym" } });

    // Before debounce: heading still shows "Browse Topics" (debouncedQuery is empty),
    // and "Searching…" hint appears
    expect(screen.getByText("Searching…")).toBeInTheDocument();
    expect(screen.getByText("Browse Topics")).toBeInTheDocument();

    act(() => {
      vi.advanceTimersByTime(250);
    });

    // After debounce: heading shows the query, "Searching…" disappears, count appears
    expect(screen.queryByText("Searching…")).not.toBeInTheDocument();
    expect(screen.getByText('Results for "Paym"')).toBeInTheDocument();
    // "Payments" category should match → 1 topic
    expect(screen.getByText(/1 topic/)).toBeInTheDocument();
  });

  it("only recomputes once after a burst of keystrokes using the final value", () => {
    render(<HelpCenterPage />);
    const input = screen.getByLabelText("Search help articles");

    // Type "St" quickly
    fireEvent.change(input, { target: { value: "S" } });
    act(() => {
      vi.advanceTimersByTime(100);
    });
    fireEvent.change(input, { target: { value: "St" } });
    act(() => {
      vi.advanceTimersByTime(100);
    });
    fireEvent.change(input, { target: { value: "Ste" } });
    act(() => {
      vi.advanceTimersByTime(300);
    });

    // "Stellar & Web3" should match → 1 topic
    expect(screen.getByText(/1 topic/)).toBeInTheDocument();
    expect(screen.getByText('Results for "Ste"')).toBeInTheDocument();
  });

  it("clears the search and resets to Browse Topics", () => {
    render(<HelpCenterPage />);
    const input = screen.getByLabelText("Search help articles");

    fireEvent.change(input, { target: { value: "Paym" } });
    act(() => {
      vi.advanceTimersByTime(250);
    });

    expect(screen.getByText('Results for "Paym"')).toBeInTheDocument();

    // Clear search
    fireEvent.click(screen.getByLabelText("Clear search"));
    act(() => {
      vi.advanceTimersByTime(250);
    });

    expect(screen.getByText("Browse Topics")).toBeInTheDocument();
  });
});
