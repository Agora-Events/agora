import { NextRequest, NextResponse } from "next/server";
import { prisma } from "@/lib/prisma";
import { withErrorHandler } from "@/lib/api-handler";
import { throwApiError } from "@/lib/api-errors";

type RouteContext = { params: Promise<{ id: string }> };

export const GET = withErrorHandler(async (request: NextRequest, context: RouteContext) => {
  const { id } = await context.params;

  const event = await prisma.event.findUnique({
    where: { id },
    select: {
      id: true,
      title: true,
      ticketPrice: true,
      totalTickets: true,
      mintedTickets: true,
      startsAt: true,
      createdAt: true,
      tickets: {
        select: {
          id: true,
          quantity: true,
          createdAt: true,
        },
      },
    },
  });

  if (!event) {
    throwApiError("Event not found", 404);
  }

  const tickets = event.tickets;
  const totalTicketsSold = tickets.reduce((sum, t) => sum + t.quantity, 0);
  const totalRevenue = totalTicketsSold * event.ticketPrice;
  const remainingTickets = event.totalTickets - totalTicketsSold;
  const sellThroughRate = event.totalTickets > 0 ? totalTicketsSold / event.totalTickets : 0;

  const priceTier = event.ticketPrice === 0
    ? "free"
    : event.ticketPrice < 25
      ? "low"
      : event.ticketPrice < 100
        ? "mid"
        : "high";

  const tierLabel =
    priceTier === "free"
      ? "Free"
      : priceTier === "low"
        ? "Low ($1–25)"
        : priceTier === "mid"
          ? "Mid ($26–100)"
          : "High ($100+)";

  const ticketTierPopularity = [
    { tier: priceTier, label: tierLabel, eventPrice: event.ticketPrice, sold: totalTicketsSold },
  ];

  const salesByDate: Record<string, { date: string; ticketsSold: number; revenue: number }> = {};
  for (const ticket of tickets) {
    const dateKey = new Date(ticket.createdAt).toISOString().slice(0, 10);
    if (!salesByDate[dateKey]) {
      salesByDate[dateKey] = { date: dateKey, ticketsSold: 0, revenue: 0 };
    }
    salesByDate[dateKey].ticketsSold += ticket.quantity;
    salesByDate[dateKey].revenue += ticket.quantity * event.ticketPrice;
  }

  const attendanceTrends = Object.values(salesByDate).sort((a, b) => a.date.localeCompare(b.date));

  const uniqueBuyers = new Set(tickets.map((t) => t.id)).size;

  return NextResponse.json({
    eventId: event.id,
    eventTitle: event.title,
    generatedAt: new Date().toISOString(),
    kpi: {
      totalTicketsSold,
      totalRevenue,
      remainingTickets,
      sellThroughRate,
      uniqueBuyers,
      ticketPrice: event.ticketPrice,
      totalCapacity: event.totalTickets,
    },
    ticketTierPopularity,
    attendanceTrends,
  });
});
