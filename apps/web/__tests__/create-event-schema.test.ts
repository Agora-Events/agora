/**
 * Unit tests for `createEventSchema` (apps/web/lib/validation.ts).
 *
 * This Zod schema enforces:
 *  - a required, non-empty title
 *  - required start date/time and location
 *  - an optional capacity that, when provided, must be a positive integer
 *  - a required, non-negative price with at most 7 decimal places
 *  - a visibility of either "Public" or "Private"
 *
 * Each test uses `safeParse` and inspects the returned issue messages
 * directly, rather than only checking the success boolean, so a change to
 * the validation copy (or the wrong field failing) is caught.
 */

import { describe, it, expect } from "vitest";
import { createEventSchema } from "@/lib/validation";

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
      const titleIssue = result.error.issues.find((issue) => issue.path.includes("title"));
      expect(titleIssue).toBeDefined();
      expect(titleIssue?.message).toBe("Event title is required");
    }
  });

  it("fails when the price is negative, with the correct message", () => {
    const result = createEventSchema.safeParse(validPayload({ price: "-10" }));

    expect(result.success).toBe(false);
    if (!result.success) {
      const priceIssue = result.error.issues.find((issue) => issue.path.includes("price"));
      expect(priceIssue).toBeDefined();
      expect(priceIssue?.message).toBe("Price cannot be negative");
    }
  });

  it.each(["0", "000", "-10"])("fails for non-positive capacity %j", (capacity) => {
    const result = createEventSchema.safeParse(validPayload({ capacity }));

    expect(result.success).toBe(false);
    if (!result.success) {
      const capacityIssue = result.error.issues.find((issue) => issue.path.includes("capacity"));
      expect(capacityIssue).toBeDefined();
      expect(capacityIssue?.message).toBe("Capacity must be greater than 0");
    }
  });

  it.each(["10abc", "1.5", "1e2", " 10", "10\n"])("rejects malformed capacity %j", (capacity) => {
    const result = createEventSchema.safeParse(validPayload({ capacity }));

    expect(result.success).toBe(false);
    if (!result.success) {
      expect(result.error.issues).toEqual([
        {
          code: "custom",
          path: ["capacity"],
          message: "Capacity must be a whole number containing only digits",
        },
      ]);
    }
  });

  it.each(["5usd", "1.23456789", "1e2", "1..5", "NaN", " 5", "5\n"])(
    "rejects malformed price %j",
    (price) => {
      const result = createEventSchema.safeParse(validPayload({ price }));

      expect(result.success).toBe(false);
      if (!result.success) {
        expect(result.error.issues).toEqual([
          {
            code: "custom",
            path: ["price"],
            message: "Price must be a number with at most 7 decimal places",
          },
        ]);
      }
    }
  );

  it("keeps the required price message for a blank price", () => {
    const result = createEventSchema.safeParse(validPayload({ price: " " }));

    expect(result.success).toBe(false);
    if (!result.success) {
      expect(result.error.issues).toHaveLength(1);
      expect(result.error.issues[0]).toMatchObject({
        path: ["price"],
        message: "Price is required (put 0 for free)",
      });
    }
  });

  it.each(["0", "25", "1.2345678", ".5"])(
    "accepts price %j and a positive whole-number capacity",
    (price) => {
      const result = createEventSchema.safeParse(validPayload({ price, capacity: "10" }));

      expect(result.success).toBe(true);
    }
  );

  it('passes when capacity is "" (optional / absent)', () => {
    const resultWithEmptyString = createEventSchema.safeParse(validPayload({ capacity: "" }));
    expect(resultWithEmptyString.success).toBe(true);

    const resultWithoutCapacity = createEventSchema.safeParse(validPayload());
    expect(resultWithoutCapacity.success).toBe(true);
  });

  it("fails when visibility is not Public or Private", () => {
    const result = createEventSchema.safeParse(validPayload({ visibility: "Hidden" }));

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

  describe("end time validation relative to start time", () => {
    it("passes when end date is the same day with a later end time", () => {
      const result = createEventSchema.safeParse(
        validPayload({
          startDate: "2026-09-01",
          startTime: "18:00",
          endDate: "2026-09-01",
          endTime: "20:00",
        })
      );

      expect(result.success).toBe(true);
    });

    it("fails when end date is the same day with an earlier end time", () => {
      const result = createEventSchema.safeParse(
        validPayload({
          startDate: "2026-09-01",
          startTime: "18:00",
          endDate: "2026-09-01",
          endTime: "10:00",
        })
      );

      expect(result.success).toBe(false);
      if (!result.success) {
        const endTimeIssue = result.error.issues.find((issue) => issue.path.includes("endTime"));
        expect(endTimeIssue).toBeDefined();
        expect(endTimeIssue?.message).toBe("End time must be after the start time");
      }
    });

    it("passes when end date is a later day even with an earlier clock time", () => {
      const result = createEventSchema.safeParse(
        validPayload({
          startDate: "2026-09-01",
          startTime: "18:00",
          endDate: "2026-09-02",
          endTime: "10:00",
        })
      );

      expect(result.success).toBe(true);
    });

    it("fails when end date and end time are identical to start date and start time", () => {
      const result = createEventSchema.safeParse(
        validPayload({
          startDate: "2026-09-01",
          startTime: "18:00",
          endDate: "2026-09-01",
          endTime: "18:00",
        })
      );

      expect(result.success).toBe(false);
      if (!result.success) {
        const endTimeIssue = result.error.issues.find((issue) => issue.path.includes("endTime"));
        expect(endTimeIssue).toBeDefined();
        expect(endTimeIssue?.message).toBe("End time must be after the start time");
      }
    });

    it("preserves existing behaviour when end date or end time is omitted", () => {
      const withoutEndDate = createEventSchema.safeParse(
        validPayload({
          startDate: "2026-09-01",
          startTime: "18:00",
          endTime: "10:00",
        })
      );
      expect(withoutEndDate.success).toBe(true);

      const withoutEndTime = createEventSchema.safeParse(
        validPayload({
          startDate: "2026-09-01",
          startTime: "18:00",
          endDate: "2026-09-01",
        })
      );
      expect(withoutEndTime.success).toBe(true);
    });
  });
});
