import { render, screen, fireEvent } from "@testing-library/react";
import { describe, it, expect, vi } from "vitest";
import EventError from "../app/events/[id]/error";

describe("EventError boundary", () => {
  it("renders error message and retry button, logging error to console.error", () => {
    const consoleErrorSpy = vi.spyOn(console, "error").mockImplementation(() => {});
    const resetMock = vi.fn();
    const testError = new Error("Failed to fetch event data");

    render(<EventError error={testError} reset={resetMock} />);

    expect(screen.getByText("Failed to load event details")).toBeInTheDocument();
    expect(screen.getByText("Failed to fetch event data")).toBeInTheDocument();
    expect(consoleErrorSpy).toHaveBeenCalledWith("Unhandled error in /events/[id]:", testError);

    const retryButton = screen.getByRole("button", { name: "Retry" });
    expect(retryButton).toBeInTheDocument();

    fireEvent.click(retryButton);
    expect(resetMock).toHaveBeenCalledTimes(1);

    consoleErrorSpy.mockRestore();
  });

  it("uses default fallback description when error.message is empty", () => {
    const consoleErrorSpy = vi.spyOn(console, "error").mockImplementation(() => {});
    const resetMock = vi.fn();
    const testError = new Error("");

    render(<EventError error={testError} reset={resetMock} />);

    expect(screen.getByText("An unexpected error occurred. Please try again.")).toBeInTheDocument();

    consoleErrorSpy.mockRestore();
  });
});
