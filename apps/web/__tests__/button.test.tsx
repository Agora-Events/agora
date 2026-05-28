import { render, screen, fireEvent } from "@testing-library/react";
import { expect, afterEach, describe, it, vi } from "vitest";
import { cleanup } from "@testing-library/react";
import { waitFor } from "@testing-library/react";
import "@testing-library/jest-dom";
import { Button } from "@/components/ui/button";

// Mock next/image
vi.mock("next/image", () => ({
  default: ({ src, alt, width, height }: { src: string; alt: string; width: number; height: number }) => (
    <img src={src} alt={alt} width={width} height={height} />
  ),
}));

describe("Button", () => {
  afterEach(() => {
    cleanup();
  });

  it("renders correctly with default props", () => {
    render(<Button>Click Me</Button>);
    
    const button = screen.getByRole("button", { name: /Click Me/i });
    expect(button).toBeInTheDocument();
    expect(button).toHaveClass("bg-white");
    expect(button).toHaveClass("text-black");
  });

  it("renders with custom background and text colors", () => {
    render(
      <Button backgroundColor="bg-black" textColor="text-white">
        Custom Button
      </Button>
    );
    
    const button = screen.getByRole("button", { name: /Custom Button/i });
    expect(button).toBeInTheDocument();
    expect(button).toHaveClass("bg-black");
    expect(button).toHaveClass("text-white");
  });

  it("handles click events correctly", () => {
    const handleClick = vi.fn();
    
    render(
      <Button onClick={handleClick}>
        Clickable Button
      </Button>
    );
    
    const button = screen.getByRole("button", { name: /Clickable Button/i });
    fireEvent.click(button);
    
    expect(handleClick).toHaveBeenCalledTimes(1);
  });
});