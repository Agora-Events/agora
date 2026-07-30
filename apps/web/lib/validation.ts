import { z } from "zod";

export const authSchema = z.object({
  email: z
    .string()
    .min(1, "Email is required")
    .email("Enter a valid email"),
});

export const createEventSchema = z.object({
  title: z.string().trim().min(1, "Event title is required"),
  startDate: z.string().min(1, "Start date is required"),
  startTime: z.string().min(1, "Start time is required"),
  endDate: z.string().optional(),
  endTime: z.string().optional(),
  location: z.string().trim().min(1, "Location is required"),
  description: z.string().optional(),
  capacity: z
    .string()
    .optional()
    .refine((value) => !value || Number.parseInt(value, 10) > 0, {
      message: "Capacity must be greater than 0",
    }),
  price: z
    .string()
    .trim()
    .min(1, "Price is required (put 0 for free)")
    .refine((value) => Number.parseFloat(value) >= 0, {
      message: "Price cannot be negative",
    }),
  visibility: z.enum(["Public", "Private"]),
});

export type AuthFormData = z.infer<typeof authSchema>;
export type CreateEventInput = z.infer<typeof createEventSchema>;

/**
 * Parses a price string into a numeric value.
 *
 * Handles:
 *  - Plain numbers:          "25"       → 25
 *  - Dollar prefix:          "$25"      → 25
 *  - Thousand separators:    "$1,200"   → 1200
 *  - Price ranges:           "$10 - $50" → 10  (returns the minimum / first tier)
 *  - Free / zero:            "free"     → 0
 *
 * @param priceStr - The raw price string from the event data.
 * @returns The parsed numeric value, or 0 when the string cannot be parsed.
 */
export function parsePriceValue(priceStr: string): number {
  if (!priceStr) return 0;

  const normalised = priceStr.trim().toLowerCase();

  if (normalised === "free" || normalised === "0") return 0;

  // For range formats like "$10 - $50" or "10-50", take the first (minimum) value.
  const rangeParts = normalised.split(/\s*[-–—]\s*/);
  const firstPart = rangeParts[0];

  // Strip everything except digits and the decimal point.
  const numeric = firstPart.replace(/[^0-9.]/g, "");

  if (!numeric) return 0;

  const parsed = parseFloat(numeric);
  return isNaN(parsed) ? 0 : parsed;
}