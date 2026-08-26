/**
 * Unit tests for `createEventSchema` (apps/web/lib/validation.ts).
 *
 * This Zod schema gates the "create event" form and enforces:
 *  - a required, non-empty title
 *  - required start date/time and location
 *  - an optional capacity that, when provided, must be a positive integer
 *  - a required, non-negative price
 *  - a visibility of either "Public" or "Private"
 *
 * Each test uses `safeParse` and inspects the returned issue messages
 * directly, rather than only checking the success boolean, so a change to
 * the validation copy (or the wrong field failing) is caught.
 */

import { describe, it, expect } from "vitest";
import {
  createEventSchema,
  MAX_DESCRIPTION_LENGTH,
} from "@/lib/validation";

/** A minimal payload that satisfies every required field in the schema. */
function validPayload(overrides: Record<string, unknown> = {}) {
  return {
    title: "Community Meetup",
    startDate: "2026-09-01",
    startTime: "18:00",
    location: "Main Hall, Lagos",
    price: "25",
    visibility: "Public",
    ...overrides,
  };
}

describe("createEventSchema", () => {
  it("parses successfully for a fully valid payload", () => {
    const result = createEventSchema.safeParse(validPayload());

    expect(result.success).toBe(true);
    if (result.success) {
      expect(result.data.title).toBe("Community Meetup");
      expect(result.data.visibility).toBe("Public");
    }
  });

  it("fails when the title is empty, with the correct message", () => {
    const result = createEventSchema.safeParse(validPayload({ title: "" }));

    expect(result.success).toBe(false);
    if (!result.success) {
      const titleIssue = result.error.issues.find((issue) =>
        issue.path.includes("title")
      );
      expect(titleIssue).toBeDefined();
      expect(titleIssue?.message).toBe("Event title is required");
    }
  });

  it("fails when the price is negative, with the correct message", () => {
    const result = createEventSchema.safeParse(validPayload({ price: "-10" }));

    expect(result.success).toBe(false);
    if (!result.success) {
      const priceIssue = result.error.issues.find((issue) =>
        issue.path.includes("price")
      );
      expect(priceIssue).toBeDefined();
      expect(priceIssue?.message).toBe("Price cannot be negative");
    }
  });

  it('fails when capacity is "0"', () => {
    const result = createEventSchema.safeParse(validPayload({ capacity: "0" }));

    expect(result.success).toBe(false);
    if (!result.success) {
      const capacityIssue = result.error.issues.find((issue) =>
        issue.path.includes("capacity")
      );
      expect(capacityIssue).toBeDefined();
      expect(capacityIssue?.message).toBe("Capacity must be greater than 0");
    }
  });

  it('passes when capacity is "" (optional / absent)', () => {
    const resultWithEmptyString = createEventSchema.safeParse(
      validPayload({ capacity: "" })
    );
    expect(resultWithEmptyString.success).toBe(true);

    const resultWithoutCapacity = createEventSchema.safeParse(validPayload());
    expect(resultWithoutCapacity.success).toBe(true);
  });

  it("fails when visibility is not Public or Private", () => {
    const result = createEventSchema.safeParse(
      validPayload({ visibility: "Hidden" })
    );

    expect(result.success).toBe(false);
    if (!result.success) {
      const visibilityIssue = result.error.issues.find((issue) =>
        issue.path.includes("visibility")
      );
      expect(visibilityIssue).toBeDefined();
      // Zod's built-in enum message — assert on the code (stable across zod
      // versions) as well as the message actually surfaced to callers.
      expect(visibilityIssue?.code).toBe("invalid_enum_value");
      expect(visibilityIssue?.message).toMatch(/Public/);
      expect(visibilityIssue?.message).toMatch(/Private/);
    }
  });

  it("accepts a description at the max length", () => {
    const result = createEventSchema.safeParse(
      validPayload({ description: "x".repeat(MAX_DESCRIPTION_LENGTH) })
    );
    expect(result.success).toBe(true);
  });

  it("rejects a description longer than the max length", () => {
    const result = createEventSchema.safeParse(
      validPayload({ description: "x".repeat(MAX_DESCRIPTION_LENGTH + 1) })
    );
    expect(result.success).toBe(false);
    if (!result.success) {
      const descriptionIssue = result.error.issues.find((issue) =>
        issue.path.includes("description")
      );
      expect(descriptionIssue).toBeDefined();
      expect(descriptionIssue?.message).toMatch(
        new RegExp(String(MAX_DESCRIPTION_LENGTH))
      );
    }
  });

  it("treats a missing description as optional", () => {
    const result = createEventSchema.safeParse(
      validPayload({ description: undefined })
    );
    expect(result.success).toBe(true);
  });
});
