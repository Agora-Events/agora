import { z } from "zod";

export const authSchema = z.object({
  email: z.string().min(1, "Email is required").email("Enter a valid email"),
});

export const createEventSchema = z.object({
  title: z.string().min(1, "Event title is required"),
  startDate: z.string().min(1, "Start date is required"),
  startTime: z.string().min(1, "Start time is required"),
  location: z.string().min(1, "Location is required"),
  price: z.string().min(1, "Price is required (put 0 for free)"),
  // optional fields
  endDate: z.string().optional(),
  endTime: z.string().optional(),
  description: z.string().optional(),
  capacity: z.string().optional(),
  visibility: z.enum(["Public", "Private"]).optional(),
});
