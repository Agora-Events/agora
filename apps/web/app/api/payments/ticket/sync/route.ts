import { NextRequest, NextResponse } from "next/server";
import { prisma } from "@/lib/prisma";
import { withErrorHandler } from "@/lib/api-handler";
import { throwApiError } from "@/lib/api-errors";

type SyncPayload = {
  txHash?: string;
  eventId?: string;
  contractId?: string;
  buyerAddress?: string;
};

/**
 * POST /api/payments/ticket/sync
 *
 * Syncs a confirmed on-chain Soroban ticket mint transaction with the backend database.
 * Called from the frontend after a transaction is confirmed on-chain.
 *
 * This endpoint:
 * 1. Validates the transaction hash and event ID
 * 2. Updates the corresponding Ticket record with the on-chain transaction hash
 * 3. Optionally resolves the on-chain ticket ID from the contract (future enhancement)
 *
 * Request body:
 * {
 *   "txHash": "0123456789abcdef...",
 *   "eventId": "event-uuid",
 *   "contractId": "CABC...",
 *   "buyerAddress": "GABC..."
 * }
 *
 * Response (success):
 * {
 *   "success": true,
 *   "ticketId": "ticket-uuid",
 *   "message": "Ticket synced successfully"
 * }
 *
 * Response (error):
 * {
 *   "error": "Error message",
 *   "code": 400 or 500
 * }
 */
export const POST = withErrorHandler(async (request: NextRequest) => {
  let payload: SyncPayload;

  try {
    payload = await request.json();
  } catch {
    throwApiError("Invalid JSON payload", 400);
  }

  const { txHash, eventId, buyerAddress } = payload;

  // Validation
  if (!txHash || typeof txHash !== "string") {
    throwApiError("Invalid txHash", 400);
  }

  if (!eventId || typeof eventId !== "string") {
    throwApiError("Invalid eventId", 400);
  }

  if (buyerAddress && typeof buyerAddress !== "string") {
    throwApiError("Invalid buyerAddress", 400);
  }

  // Normalize hash (remove 0x prefix if present)
  const normalizedHash = txHash.startsWith("0x") ? txHash.slice(2) : txHash;

  try {
    // Find the most recent ticket for this event and buyer (if provided)
    // that doesn't already have a Stellar ID (tx hash) associated
    const query: {
      eventId: string;
      stellarId?: { equals: null };
      buyerWallet?: string;
    } = {
      eventId,
      stellarId: { equals: null },
    };

    if (buyerAddress) {
      query.buyerWallet = buyerAddress;
    }

    const ticket = await prisma.ticket.findFirst({
      where: query,
      orderBy: { createdAt: "desc" },
    });

    if (!ticket) {
      // No unlinked ticket found — this could be a duplicate sync or stale request
      console.warn(
        `[Ticket Sync] No unlinked ticket found for event ${eventId} ` +
          `${buyerAddress ? `and buyer ${buyerAddress}` : ""}`,
      );
      throwApiError(
        "No pending ticket found. The ticket may have already been synced.",
        404,
      );
    }

    // Update the ticket with the on-chain transaction hash
    const updatedTicket = await prisma.ticket.update({
      where: { id: ticket.id },
      data: {
        stellarId: normalizedHash,
      },
    });

    return NextResponse.json(
      {
        success: true,
        ticketId: updatedTicket.id,
        message: "Ticket synced successfully",
      },
      { status: 200 },
    );
  } catch (error) {
    if (error instanceof Error && error.message.includes("No pending ticket")) {
      throw error;
    }

    console.error("[Ticket Sync] Database error:", error);
    throwApiError(
      "Failed to sync ticket with on-chain data. Please try again.",
      500,
    );
  }
});
