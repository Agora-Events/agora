import { NextRequest, NextResponse } from "next/server";
import { prisma } from "@/lib/prisma";
import { withErrorHandler } from "@/lib/api-handler";
import { throwApiError } from "@/lib/api-errors";

/**
 * GET /api/events/[id]/availability
 *
 * Returns real-time ticket availability information for an event.
 * This endpoint is designed to be called frequently (every few seconds)
 * for live updates on the event detail page.
 *
 * @param request - NextRequest object
 * @param context - Route context with params
 * @returns JSON with ticket availability data
 *
 * @example
 * Response (200 OK):
 * {
 *   "totalTickets": 100,
 *   "mintedTickets": 45,
 *   "availableTickets": 55,
 *   "isSoldOut": false,
 *   "percentageSold": 45
 * }
 */
export const GET = withErrorHandler(
  async (
    request: NextRequest,
    { params }: { params: Promise<{ id: string }> },
  ) => {
    const { id: eventId } = await params;

    // Validate eventId
    if (!eventId || typeof eventId !== "string") {
      throwApiError("Invalid eventId", 400);
    }

    // Fetch event from database
    const event = await prisma.event.findUnique({
      where: { id: eventId },
      select: {
        id: true,
        totalTickets: true,
        mintedTickets: true,
      },
    });

    if (!event) {
      throwApiError("Event not found", 404);
    }

    // Calculate availability metrics
    const availableTickets = event.totalTickets - event.mintedTickets;
    const isSoldOut = availableTickets <= 0;
    const percentageSold =
      event.totalTickets > 0
        ? Math.round((event.mintedTickets / event.totalTickets) * 100)
        : 0;

    // Return availability data
    const response = {
      totalTickets: event.totalTickets,
      mintedTickets: event.mintedTickets,
      availableTickets: Math.max(0, availableTickets),
      isSoldOut,
      percentageSold,
    };

    // Set cache headers to allow browser caching for a short duration
    // This balances real-time updates with reduced server load
    const responseHeaders = new Headers({
      "Content-Type": "application/json",
      "Cache-Control": "public, max-age=2, stale-while-revalidate=10",
    });

    return new NextResponse(JSON.stringify(response), {
      status: 200,
      headers: responseHeaders,
    });
  },
);
