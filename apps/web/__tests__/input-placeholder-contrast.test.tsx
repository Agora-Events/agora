import { render, screen } from "@testing-library/react";
import { describe, expect, it } from "vitest";
import { Input, PLACEHOLDER_TONE_CLASS } from "@/components/ui/Input";

// WCAG 2.x relative luminance / contrast ratio (#1576).
type Rgb = [number, number, number];
const hex = (h: string): Rgb => [1, 3, 5].map((i) => parseInt(h.slice(i, i + 2), 16)) as Rgb;
const mix = (fg: Rgb, bg: Rgb, alpha: number): Rgb =>
  fg.map((c, i) => Math.round(c * alpha + bg[i] * (1 - alpha))) as Rgb;
const luminance = (rgb: Rgb) => {
  const [r, g, b] = rgb.map((v) => {
    const c = v / 255;
    return c <= 0.03928 ? c / 12.92 : ((c + 0.055) / 1.055) ** 2.4;
  });
  return 0.2126 * r + 0.7152 * g + 0.0722 * b;
};
const contrast = (a: Rgb, b: Rgb) => {
  const [hi, lo] = [luminance(a), luminance(b)].sort((x, y) => y - x);
  return (hi + 0.05) / (lo + 0.05);
};

// Tailwind v4 gray-500 and the theme tokens in app/globals.css.
const WHITE = hex("#ffffff");
const GRAY_500 = hex("#6a7282");
const THEMES = {
  light: { ink: hex("#131517"), surface: hex("#FFEFD3") },
  dark: { ink: hex("#a8aab0"), surface: hex("#1c1f25") },
};
const AA = 4.5;

describe("search placeholder contrast", () => {
  it("light tone (gray-500 on white) meets WCAG AA", () => {
    expect(PLACEHOLDER_TONE_CLASS.light).toBe("placeholder:text-gray-500");
    expect(contrast(GRAY_500, WHITE)).toBeGreaterThanOrEqual(AA);
  });

  it.each(Object.entries(THEMES))(
    "surface tone (ink-deep/80) meets WCAG AA in the %s theme",
    (_, { ink, surface }) => {
      expect(PLACEHOLDER_TONE_CLASS.surface).toBe("placeholder:text-ink-deep/80");
      expect(contrast(mix(ink, surface, 0.8), surface)).toBeGreaterThanOrEqual(AA);
    },
  );

  it("the old ink-deep/40 placeholder failed AA, guarding against a regression", () => {
    const { ink, surface } = THEMES.light;
    expect(contrast(mix(ink, surface, 0.4), surface)).toBeLessThan(AA);
  });
});

describe("Input", () => {
  it("applies the light placeholder tone by default and keeps caller classes", () => {
    render(<Input placeholder="Search" className="text-sm" />);
    const input = screen.getByPlaceholderText("Search");
    expect(input).toHaveClass("placeholder:text-gray-500", "text-sm");
  });

  it("applies the surface tone when requested and forwards input props", () => {
    render(<Input tone="surface" type="search" aria-label="Filter conversations" />);
    const input = screen.getByRole("searchbox", { name: "Filter conversations" });
    expect(input).toHaveClass("placeholder:text-ink-deep/80");
    expect(input).not.toHaveClass("placeholder:text-gray-500");
  });
});
