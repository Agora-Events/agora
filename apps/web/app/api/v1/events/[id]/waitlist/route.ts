import { NextRequest, NextResponse } from "next/server";
import { getAuthFromRequest } from "@/lib/auth";
import { withErrorHandler } from "@/lib/api-handler";
import { throwApiError } from "@/lib/api-errors";

type RouteContext = { params: Promise<{ id: string }> };

/**
 * POST /api/v1/events/:id/waitlist
 * Join the waitlist for a sold-out event.
 *
 * Returns { position: number } on success.
 */
export const POST = withErrorHandler(async (request: NextRequest, context: RouteContext) => {
  const auth = getAuthFromRequest(request);
  if (!auth?.email) throwApiError("Unauthorized", 401);

  const { id } = await context.params;
  if (!id) throwApiError("Event ID is required", 400);

  // Stub: replace with real DB insert + Soroban contract call.
  // SELECT COUNT(*) FROM waitlist WHERE event_id = id gives queue length.
  const mockPosition = Math.floor(Math.random() * 50) + 1;

  return NextResponse.json(
    { success: true, position: mockPosition, eventId: id },
    { status: 201 },
  );
});

/**
 * DELETE /api/v1/events/:id/waitlist
 * Leave the waitlist for an event.
 */
export const DELETE = withErrorHandler(async (request: NextRequest, context: RouteContext) => {
  const auth = getAuthFromRequest(request);
  if (!auth?.email) throwApiError("Unauthorized", 401);

  const { id } = await context.params;
  if (!id) throwApiError("Event ID is required", 400);

  // Stub: DELETE FROM waitlist WHERE event_id = id AND email = auth.email
  return NextResponse.json({ success: true, eventId: id });
});

/**
 * GET /api/v1/events/:id/waitlist
 * Get the caller's waitlist position for this event (0 = not on waitlist).
 */
export const GET = withErrorHandler(async (request: NextRequest, context: RouteContext) => {
  const auth = getAuthFromRequest(request);
  if (!auth?.email) throwApiError("Unauthorized", 401);

  const { id } = await context.params;
  if (!id) throwApiError("Event ID is required", 400);

  // Stub: SELECT position FROM waitlist WHERE event_id = id AND email = auth.email
  return NextResponse.json({ position: 0, eventId: id });
});
