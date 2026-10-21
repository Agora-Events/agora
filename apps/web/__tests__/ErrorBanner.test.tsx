import { render, screen, fireEvent } from "@testing-library/react";
import { describe, it, expect, vi } from "vitest";
import { ErrorBanner } from "@/components/ui/error-banner";

describe("ErrorBanner", () => {
  it("renders the message and description", () => {
    render(
      <ErrorBanner message="Failed to load events" description="The server did not respond." />
    );

    expect(screen.getByText("Failed to load events")).toBeInTheDocument();
    expect(screen.getByText("The server did not respond.")).toBeInTheDocument();
  });

  it("omits the description when none is provided", () => {
    render(<ErrorBanner message="Failed to load events" />);

    expect(screen.getByRole("alert")).toHaveTextContent("Failed to load events");
    expect(screen.queryByText("The server did not respond.")).not.toBeInTheDocument();
  });

  it("exposes role=alert so screen readers announce the error", () => {
    render(<ErrorBanner message="Something went wrong" />);

    const alert = screen.getByRole("alert");
    expect(alert).toHaveTextContent("Something went wrong");
  });

  it("does not render a retry button when onRetry is not supplied", () => {
    render(<ErrorBanner message="Something went wrong" />);

    expect(screen.queryByRole("button", { name: "Retry" })).not.toBeInTheDocument();
  });

  it("renders a retry button that calls onRetry when clicked", () => {
    const onRetry = vi.fn();
    render(<ErrorBanner message="Something went wrong" onRetry={onRetry} />);

    fireEvent.click(screen.getByRole("button", { name: "Retry" }));

    expect(onRetry).toHaveBeenCalledTimes(1);
  });
});
