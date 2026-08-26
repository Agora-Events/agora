import { render, screen, fireEvent } from "@testing-library/react";
import { expect, afterEach, describe, it, vi } from "vitest";
import { cleanup } from "@testing-library/react";
import "@testing-library/jest-dom";
import CreateEventForm from "@/components/events/create-event-form";
import { MAX_DESCRIPTION_LENGTH } from "@/lib/validation";

// Mock next/image
vi.mock("next/image", () => ({
  default: ({
    src,
    alt,
    width,
    height,
  }: {
    src: string;
    alt: string;
    width: number;
    height: number;
  }) => <img src={src} alt={alt} width={width} height={height} />,
}));

describe("CreateEventForm character counter", () => {
  afterEach(() => {
    cleanup();
  });

  it("renders the character counter and links it to the textarea", () => {
    render(<CreateEventForm />);

    const textarea = screen.getByLabelText(/Add Description/i);
    expect(textarea).toHaveAttribute(
      "aria-describedby",
      "description-character-count",
    );
    expect(textarea).toHaveAttribute(
      "maxlength",
      String(MAX_DESCRIPTION_LENGTH),
    );

    const counter = screen.getByText(
      new RegExp(`0 / ${MAX_DESCRIPTION_LENGTH.toLocaleString()}`),
    );
    expect(counter).toBeInTheDocument();
  });

  it("updates the counter as the user types", () => {
    render(<CreateEventForm />);

    const textarea = screen.getByLabelText(/Add Description/i);
    fireEvent.change(textarea, { target: { value: "hello" } });

    const counter = screen.getByText(
      new RegExp(`5 / ${MAX_DESCRIPTION_LENGTH.toLocaleString()}`),
    );
    expect(counter).toBeInTheDocument();
  });

  it("switches to amber styling at 90% of the limit", () => {
    render(<CreateEventForm />);

    const ninetyPercent = Math.floor(MAX_DESCRIPTION_LENGTH * 0.9);
    const textarea = screen.getByLabelText(/Add Description/i);
    fireEvent.change(textarea, {
      target: { value: "x".repeat(ninetyPercent) },
    });

    const counter = screen.getByText(
      new RegExp(`${ninetyPercent.toLocaleString()} /`),
    );
    expect(counter).toHaveClass("text-amber-500");
  });

  it("switches to red styling at 100% of the limit", () => {
    render(<CreateEventForm />);

    const textarea = screen.getByLabelText(/Add Description/i);
    fireEvent.change(textarea, {
      target: { value: "x".repeat(MAX_DESCRIPTION_LENGTH) },
    });

    const counter = screen.getByText(
      new RegExp(`${MAX_DESCRIPTION_LENGTH.toLocaleString()} /`),
    );
    expect(counter).toHaveClass("text-red-500");
  });
});
