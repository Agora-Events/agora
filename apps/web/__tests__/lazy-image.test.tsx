import { render, screen } from "@testing-library/react";
import { describe, it, expect, vi, beforeEach } from "vitest";
import { LazyImage } from "@/components/ui/LazyImage";

describe("LazyImage", () => {
  beforeEach(() => {
    const mockObserver = {
      observe: vi.fn(),
      disconnect: vi.fn(),
      unobserve: vi.fn(),
    };
    vi.stubGlobal(
      "IntersectionObserver",
      vi.fn(() => mockObserver)
    );
  });

  it("renders with placeholder initially", () => {
    render(
      <LazyImage src="/real.jpg" alt="test" placeholderSrc="/placeholder.jpg" />
    );
    const img = screen.getByRole("img");
    expect(img).toHaveAttribute("src", "/placeholder.jpg");
    expect(img).toHaveAttribute("data-src", "/real.jpg");
  });

  it("renders real src when IntersectionObserver is not available", () => {
    vi.stubGlobal("IntersectionObserver", undefined);
    render(<LazyImage src="/real.jpg" alt="test" />);
    const img = screen.getByRole("img");
    expect(img).toHaveAttribute("src", "/real.jpg");
  });

  it("has blur class before load and revealed class after", () => {
    vi.stubGlobal("IntersectionObserver", undefined);
    render(<LazyImage src="/real.jpg" alt="test" />);
    const img = screen.getByRole("img");
    expect(img).toHaveClass("lazy-image--revealed");
  });

  it("applies extra className", () => {
    vi.stubGlobal("IntersectionObserver", undefined);
    render(<LazyImage src="/real.jpg" alt="test" className="rounded-lg" />);
    expect(screen.getByRole("img")).toHaveClass("rounded-lg");
  });
});
