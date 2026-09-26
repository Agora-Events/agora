import { describe, it, expect } from "vitest";
import { parsePriceValue } from "@/lib/validation";

describe("parsePriceValue", () => {
  it("parses a plain number string", () => {
    expect(parsePriceValue("25")).toBe(25);
  });

  it("parses a value with a dollar prefix", () => {
    expect(parsePriceValue("$25")).toBe(25);
  });

  it("parses a value with a thousands separator", () => {
    expect(parsePriceValue("$1,200")).toBe(1200);
  });

  it("parses a price range and returns the minimum value", () => {
    expect(parsePriceValue("$10 - $50")).toBe(10);
  });

  it('parses the literal "free" (lowercase) as 0', () => {
    expect(parsePriceValue("free")).toBe(0);
  });

  it('parses "Free" case-insensitively as 0', () => {
    expect(parsePriceValue("Free")).toBe(0);
  });

  it('parses the literal "0" as 0', () => {
    expect(parsePriceValue("0")).toBe(0);
  });

  it("returns 0 for an empty string", () => {
    expect(parsePriceValue("")).toBe(0);
  });

  it("returns 0 for undefined input", () => {
    expect(parsePriceValue(undefined as unknown as string)).toBe(0);
  });

  it("returns 0 for a whitespace-only string", () => {
    expect(parsePriceValue("   ")).toBe(0);
  });

  it("returns 0 (not NaN) for a non-numeric string like 'TBA'", () => {
    const result = parsePriceValue("TBA");
    expect(result).toBe(0);
    expect(Number.isNaN(result)).toBe(false);
  });

  it("returns 0 (not NaN) for other unparseable input", () => {
    const result = parsePriceValue("call for pricing");
    expect(result).toBe(0);
    expect(Number.isNaN(result)).toBe(false);
  });

  it("handles a range without dollar signs, returning the minimum value", () => {
    expect(parsePriceValue("10-50")).toBe(10);
  });

  it("trims surrounding whitespace before parsing", () => {
    expect(parsePriceValue("  $25  ")).toBe(25);
  });
});
