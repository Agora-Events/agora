import { describe, it, expect } from "vitest";
import { slugify, withRandomSuffix } from "@/lib/slugify";

describe("slugify", () => {
  it("converts a normal title to a lowercase, hyphenated slug", () => {
    expect(slugify("Lagos Tech Meetup")).toBe("lagos-tech-meetup");
  });

  it("collapses extra spaces and strips punctuation and emojis", () => {
    expect(slugify("  Hello,   World! \u{1F389}\u{1F389} ")).toBe("hello-world");
  });

  it("strips accented characters down to their base letters", () => {
    expect(slugify("café")).toBe("cafe");
  });

  it("falls back to a default slug for an empty string", () => {
    expect(slugify("")).toBe("event");
  });

  it("falls back to a default slug for whitespace-only input", () => {
    expect(slugify("   ")).toBe("event");
  });

  it("falls back to a default slug when nothing alphanumeric remains", () => {
    expect(slugify("!!!???")).toBe("event");
  });

  it("truncates long titles to 80 characters", () => {
    const slug = slugify("a".repeat(100));
    expect(slug).toHaveLength(80);
    expect(slug).toBe("a".repeat(80));
  });

  it("does not leave a trailing hyphen after truncation", () => {
    const slug = slugify(`${"a".repeat(79)}---!!!`);
    expect(slug.endsWith("-")).toBe(false);
  });
});

describe("withRandomSuffix", () => {
  it("keeps the original slug as a prefix of the result", () => {
    const result = withRandomSuffix("lagos-tech-meetup");
    expect(result.startsWith("lagos-tech-meetup-")).toBe(true);
  });

  it("appends a suffix separated by a hyphen", () => {
    const result = withRandomSuffix("lagos-tech-meetup");
    const suffix = result.slice("lagos-tech-meetup-".length);
    expect(suffix).toMatch(/^[a-z0-9]+$/);
    expect(suffix.length).toBeGreaterThan(0);
  });

  it("gives different results on two calls for the same slug", () => {
    const first = withRandomSuffix("lagos-tech-meetup");
    const second = withRandomSuffix("lagos-tech-meetup");
    expect(first).not.toBe(second);
  });
});
