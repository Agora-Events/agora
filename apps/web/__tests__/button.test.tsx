import { render, screen, fireEvent } from "@testing-library/react";
import { expect, describe, it, vi } from "vitest";
import { Button } from "@/components/ui/button";

describe("Button Component", () => {
  describe("Rendering", () => {
    it("renders children correctly", () => {
      render(<Button>Click Me</Button>);
      
      const button = screen.getByRole("button", { name: /Click Me/i });
      expect(button).toBeInTheDocument();
      expect(button).toHaveTextContent("Click Me");
    });

    it("renders with complex children (icons + text)", () => {
      render(
        <Button>
          <span>Icon</span>
          <span>Text</span>
        </Button>
      );
      
      const button = screen.getByRole("button");
      expect(button).toBeInTheDocument();
      expect(button).toHaveTextContent("IconText");
    });
  });

  describe("Variant Styling", () => {
    it("applies default secondary variant styles when no variant is specified", () => {
      render(<Button>Default Button</Button>);
      
      const button = screen.getByRole("button");
      expect(button).toHaveClass("bg-white");
      expect(button).toHaveClass("text-black");
    });

    it("applies primary variant styles correctly", () => {
      render(<Button variant="primary">Primary Button</Button>);
      
      const button = screen.getByRole("button");
      expect(button).toHaveClass("bg-[#FDDA23]");
      expect(button).toHaveClass("text-black");
    });

    it("applies secondary variant styles correctly", () => {
      render(<Button variant="secondary">Secondary Button</Button>);
      
      const button = screen.getByRole("button");
      expect(button).toHaveClass("bg-white");
      expect(button).toHaveClass("text-black");
    });

    it("applies dark variant styles correctly", () => {
      render(<Button variant="dark">Dark Button</Button>);
      
      const button = screen.getByRole("button");
      expect(button).toHaveClass("bg-black");
      expect(button).toHaveClass("text-white");
    });

    it("applies ghost variant styles correctly", () => {
      render(<Button variant="ghost">Ghost Button</Button>);
      
      const button = screen.getByRole("button");
      expect(button).toHaveClass("bg-transparent");
      expect(button).toHaveClass("text-black");
    });
  });

  describe("Custom Color Props", () => {
    it("applies custom Tailwind background color class", () => {
      render(
        <Button backgroundColor="bg-red-500">Custom BG</Button>
      );
      
      const button = screen.getByRole("button");
      expect(button).toHaveClass("bg-red-500");
    });

    it("applies custom Tailwind text color class", () => {
      render(
        <Button textColor="text-blue-500">Custom Text</Button>
      );
      
      const button = screen.getByRole("button");
      expect(button).toHaveClass("text-blue-500");
    });

    it("applies custom hex background color via inline style", () => {
      render(
        <Button backgroundColor="#FF5733">Custom Hex BG</Button>
      );
      
      const button = screen.getByRole("button");
      expect(button).toHaveStyle({ backgroundColor: "#FF5733" });
    });

    it("applies custom hex text color via inline style", () => {
      render(
        <Button textColor="#00FF00">Custom Hex Text</Button>
      );
      
      const button = screen.getByRole("button");
      expect(button).toHaveStyle({ color: "#00FF00" });
    });

    it("overrides variant styles with custom color props", () => {
      render(
        <Button variant="primary" backgroundColor="bg-purple-500" textColor="text-yellow-500">
          Override Variant
        </Button>
      );
      
      const button = screen.getByRole("button");
      expect(button).toHaveClass("bg-purple-500");
      expect(button).toHaveClass("text-yellow-500");
      expect(button).not.toHaveClass("bg-[#FDDA23]");
    });
  });

  describe("Shadow Styling", () => {
    it("applies default shadow color", () => {
      render(<Button>Shadow Button</Button>);
      
      const button = screen.getByRole("button");
      expect(button).toHaveStyle({ boxShadow: "-4px 4px 0px 0px rgba(0,0,0,1)" });
    });

    it("applies custom shadow color", () => {
      render(
        <Button shadowColor="rgba(255,0,0,0.5)">Custom Shadow</Button>
      );
      
      const button = screen.getByRole("button");
      expect(button).toHaveStyle({ boxShadow: "-4px 4px 0px 0px rgba(255,0,0,0.5)" });
    });

    it("applies variant-specific shadow color", () => {
      render(<Button variant="dark">Dark Shadow</Button>);
      
      const button = screen.getByRole("button");
      expect(button).toHaveStyle({ boxShadow: "-4px 4px 0px 0px rgba(0,0,0,0.4)" });
    });

    it("applies ghost variant transparent shadow", () => {
      render(<Button variant="ghost">Ghost Shadow</Button>);
      
      const button = screen.getByRole("button");
      expect(button).toHaveStyle({ boxShadow: "-4px 4px 0px 0px transparent" });
    });
  });

  describe("Event Handlers", () => {
    it("handles click events correctly", () => {
      const handleClick = vi.fn();
      
      render(
        <Button onClick={handleClick}>Clickable Button</Button>
      );
      
      const button = screen.getByRole("button");
      fireEvent.click(button);
      
      expect(handleClick).toHaveBeenCalledTimes(1);
    });

    it("handles multiple clicks", () => {
      const handleClick = vi.fn();
      
      render(
        <Button onClick={handleClick}>Multi Click</Button>
      );
      
      const button = screen.getByRole("button");
      fireEvent.click(button);
      fireEvent.click(button);
      fireEvent.click(button);
      
      expect(handleClick).toHaveBeenCalledTimes(3);
    });

    it("does not throw error when no onClick handler is provided", () => {
      render(<Button>No Handler</Button>);
      
      const button = screen.getByRole("button");
      expect(() => fireEvent.click(button)).not.toThrow();
    });
  });

  describe("HTML Attributes", () => {
    it("applies custom className alongside default classes", () => {
      render(
        <Button className="custom-class">Custom Class</Button>
      );
      
      const button = screen.getByRole("button");
      expect(button).toHaveClass("custom-class");
      expect(button).toHaveClass("rounded-full");
      expect(button).toHaveClass("border-black");
    });

    it("applies custom inline styles", () => {
      render(
        <Button style={{ marginTop: "20px", fontSize: "18px" }}>
          Custom Style
        </Button>
      );
      
      const button = screen.getByRole("button");
      expect(button).toHaveStyle({ marginTop: "20px", fontSize: "18px" });
    });

    it("merges custom styles with component styles", () => {
      render(
        <Button style={{ marginTop: "20px" }} backgroundColor="#FF0000">
          Merged Styles
        </Button>
      );
      
      const button = screen.getByRole("button");
      expect(button).toHaveStyle({ 
        marginTop: "20px",
        backgroundColor: "#FF0000"
      });
    });

    it("applies disabled attribute", () => {
      render(<Button disabled>Disabled Button</Button>);
      
      const button = screen.getByRole("button");
      expect(button).toBeDisabled();
    });

    it("applies type attribute", () => {
      render(<Button type="submit">Submit Button</Button>);
      
      const button = screen.getByRole("button");
      expect(button).toHaveAttribute("type", "submit");
    });

    it("defaults to type='button' when no type is specified", () => {
      render(<Button>Default Type</Button>);
      
      const button = screen.getByRole("button");
      expect(button).toHaveAttribute("type", "button");
    });

    it("applies aria-label attribute", () => {
      render(<Button aria-label="Close dialog">X</Button>);
      
      const button = screen.getByRole("button", { name: "Close dialog" });
      expect(button).toBeInTheDocument();
    });

    it("applies data attributes", () => {
      render(<Button data-testid="test-button" data-custom="value">Data Attrs</Button>);
      
      const button = screen.getByTestId("test-button");
      expect(button).toHaveAttribute("data-custom", "value");
    });
  });

  describe("Base Styling Classes", () => {
    it("always includes base structural classes", () => {
      render(<Button>Base Classes</Button>);
      
      const button = screen.getByRole("button");
      expect(button).toHaveClass("flex");
      expect(button).toHaveClass("items-center");
      expect(button).toHaveClass("justify-center");
      expect(button).toHaveClass("gap-2");
      expect(button).toHaveClass("px-6");
      expect(button).toHaveClass("py-3");
      expect(button).toHaveClass("rounded-full");
      expect(button).toHaveClass("border");
      expect(button).toHaveClass("border-black");
      expect(button).toHaveClass("font-semibold");
    });

    it("includes hover and active state classes", () => {
      render(<Button>Interactive</Button>);
      
      const button = screen.getByRole("button");
      expect(button.className).toContain("hover:-translate-x-[2px]");
      expect(button.className).toContain("hover:translate-y-[2px]");
      expect(button.className).toContain("active:-translate-x-[4px]");
      expect(button.className).toContain("active:translate-y-[4px]");
    });
  });
});