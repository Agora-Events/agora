import { NextRequest, NextResponse } from "next/server";
import { prisma } from "@/lib/prisma";
import { mintTicket } from "@/utils/stellar";
import { withErrorHandler } from "@/lib/api-handler";
import { throwApiError, ApiError } from "@/lib/api-errors";

type TicketRequestBody = {
  eventId?: string;
  quantity?: number;
  buyerWallet?: string;
};

export const POST = withErrorHandler(async (request: NextRequest) => {
  let payload: TicketRequestBody;
  try {
    payload = await request.json();
  } catch {
    throwApiError("Invalid JSON payload", 400);
  }

  const { eventId, quantity, buyerWallet } = payload;

  if (!eventId || typeof eventId !== "string") {
    throwApiError("Invalid eventId", 400);
  }
  if (!Number.isInteger(quantity) || (quantity ?? 0) <= 0) {
    throwApiError("Invalid quantity", 400);
  }
  if (!buyerWallet || typeof buyerWallet !== "string") {
    throwApiError("Invalid buyerWallet", 400);
  }

  const event = await prisma.event.findUnique({
    where: { id: eventId },
  });

  if (!event) {
    throwApiError("Event not found", 404);
  }

  const qty = quantity as number;

  if (event.mintedTickets + qty > event.totalTickets) {
    throwApiError("Not enough tickets available", 409);
  }

  try {
    const mintResult = await mintTicket(eventId, buyerWallet, qty);

    await prisma.$transaction([
      prisma.event.update({
        where: { id: eventId },
        data: { mintedTickets: { increment: qty } },
      }),
      prisma.ticket.create({
        data: {
          stellarId: mintResult.ticketId,
          eventId,
          buyerWallet,
          quantity: qty,
        },
      }),
    ]);

    return NextResponse.json(
      {
        ticketId: mintResult.ticketId,
        transactionXdr: mintResult.transactionXdr,
      },
      { status: 200 },
    );
  } catch (error) {
    if (error instanceof ApiError) throw error;
    throwApiError("Failed to mint ticket", 502);
  }
});


