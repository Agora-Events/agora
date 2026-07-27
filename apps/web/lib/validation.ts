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