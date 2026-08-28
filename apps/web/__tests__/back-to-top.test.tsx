import { render, screen, fireEvent, act } from "@testing-library/react";
import { expect, describe, it, vi, beforeEach, afterEach } from "vitest";
import { BackToTop } from "@/components/ui/back-to-top";

describe("BackToTop component", () => {
  beforeEach(() => {
    // Fresh scroll state for each test.
    window.scrollY = 0;
    window.innerHeight = 800;
  });

  afterEach(() => {
    vi.restoreAllMocks();
  });

  it("renders a button with an accessible label", () => {
    render(<BackToTop />);
    expect(screen.getByRole("button", { name: "Back to top" })).toBeInTheDocument();
  });

  it("is hidden when the page is at the top", () => {
    render(<BackToTop />);
    const button = screen.getByRole("button", { name: "Back to top" });
    expect(button.className).toContain("opacity-0");
    expect(button.className).toContain("pointer-events-none");
  });

  it("becomes visible after scrolling past one viewport", () => {
    render(<BackToTop />);
    const button = screen.getByRole("button", { name: "Back to top" });
    expect(button.className).toContain("opacity-0");

    window.scrollY = 1000; // > innerHeight (800)
    act(() => {
      window.dispatchEvent(new Event("scroll"));
    });

    expect(button.className).toContain("opacity-100");
    expect(button.className).toContain("pointer-events-auto");
  });

  it("hides again when scrolled back to the top", () => {
    render(<BackToTop />);
    const button = screen.getByRole("button", { name: "Back to top" });

    window.scrollY = 1000;
    act(() => {
      window.dispatchEvent(new Event("scroll"));
    });
    expect(button.className).toContain("opacity-100");

    window.scrollY = 200;
    act(() => {
      window.dispatchEvent(new Event("scroll"));
    });
    expect(button.className).toContain("opacity-0");
  });

  it("smooth-scrolls to the top on click", () => {
    const scrollTo = vi.fn();
    vi.stubGlobal("scrollTo", scrollTo);
    window.matchMedia = vi.fn().mockReturnValue({ matches: false });

    render(<BackToTop />);
    fireEvent.click(screen.getByRole("button", { name: "Back to top" }));

    expect(scrollTo).toHaveBeenCalledWith({ top: 0, behavior: "smooth" });
  });

  it("scrolls instantly when the user prefers reduced motion", () => {
    const scrollTo = vi.fn();
    vi.stubGlobal("scrollTo", scrollTo);
    window.matchMedia = vi.fn().mockReturnValue({ matches: true });

    render(<BackToTop />);
    fireEvent.click(screen.getByRole("button", { name: "Back to top" }));

    expect(scrollTo).toHaveBeenCalledWith({ top: 0, behavior: "instant" });
  });

  it("removes the scroll listener on unmount", () => {
    const removeSpy = vi.spyOn(window, "removeEventListener");
    const { unmount } = render(<BackToTop />);
    unmount();

    expect(removeSpy).toHaveBeenCalledWith("scroll", expect.any(Function));
  });
});
