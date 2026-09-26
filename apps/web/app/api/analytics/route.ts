import { NextRequest, NextResponse } from "next/server";
import { prisma } from "@/lib/prisma";
import { withErrorHandler } from "@/lib/api-handler";
import { throwApiError } from "@/lib/api-errors";

const FUNNEL_EVENTS = new Set(["page_view", "checkout_started"]);

export const POST = withErrorHandler(async (request: NextRequest) => {
  const payload = (await request.json()) as { eventId?: unknown; type?: unknown };
  if (typeof payload.eventId !== "string" || !payload.eventId) {
    throwApiError("Invalid eventId", 400);
  }
  if (typeof payload.type !== "string" || !FUNNEL_EVENTS.has(payload.type)) {
    throwApiError("Invalid analytics event type", 400);
  }

  await prisma.analyticsEvent.create({
    data: { eventId: payload.eventId, type: payload.type },
  });

  return NextResponse.json({ recorded: true }, { status: 201 });
});

export const GET = withErrorHandler(async (request: NextRequest) => {
  const organizerWallet = request.nextUrl.searchParams.get("organizerWallet");
  const eventId = request.nextUrl.searchParams.get("eventId");
  if (!organizerWallet && !eventId) {
    throwApiError("organizerWallet or eventId is required", 400);
  }

  const eventWhere = eventId ? { id: eventId } : { organizerWallet: organizerWallet! };
  const [events, funnelEvents] = await Promise.all([
    prisma.event.findMany({
      where: eventWhere,
      select: {
        id: true,
        ticketPrice: true,
        tickets: { select: { quantity: true, createdAt: true } },
      },
    }),
    prisma.analyticsEvent.groupBy({
      by: ["type"],
      where: { event: eventWhere },
      _count: { _all: true },
    }),
  ]);

  const eventIds = events.map((event) => event.id);
  const tickets = events.flatMap((event) =>
    event.tickets.map((ticket) => ({ ...ticket, price: event.ticketPrice })),
  );
  const countFor = (type: string) =>
    funnelEvents.find((event) => event.type === type)?._count._all ?? 0;
  const purchases = tickets.reduce((sum, ticket) => sum + ticket.quantity, 0);

  const dailySales = new Map<string, number>();
  for (const ticket of tickets) {
    const date = ticket.createdAt.toISOString().slice(0, 10);
    dailySales.set(date, (dailySales.get(date) ?? 0) + ticket.quantity);
  }

  const now = new Date();
  const currentStart = new Date(now);
  currentStart.setUTCDate(currentStart.getUTCDate() - 6);
  currentStart.setUTCHours(0, 0, 0, 0);
  const previousStart = new Date(currentStart);
  previousStart.setUTCDate(previousStart.getUTCDate() - 7);
  const revenueInRange = (start: Date, end: Date) =>
    tickets.reduce(
      (sum, ticket) =>
        ticket.createdAt >= start && ticket.createdAt < end
          ? sum + ticket.quantity * ticket.price
          : sum,
      0,
    );

  return NextResponse.json({
    eventIds,
    funnel: {
      pageViews: countFor("page_view"),
      checkoutStarted: countFor("checkout_started"),
      successfulPurchases: purchases,
    },
    dailySales: Array.from(dailySales, ([date, ticketCount]) => ({
      date,
      tickets: ticketCount,
    })).sort((a, b) => a.date.localeCompare(b.date)),
    revenueComparison: [
      {
        period: "Last 7 days",
        revenue: revenueInRange(currentStart, now),
        previousRevenue: revenueInRange(previousStart, currentStart),
      },
    ],
    ticketTiers: purchases > 0 ? [{ name: "General admission", tickets: purchases }] : [],
  });
});
