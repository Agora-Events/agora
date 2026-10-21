import { NextRequest, NextResponse } from "next/server";
import { ApiError } from "./api-errors";

type RouteContext<T = any> = {
  params: Promise<T>;
};

type RouteHandler<T = any> = (
  request: NextRequest,
  context: RouteContext<T>,
) => Promise<NextResponse> | NextResponse;

export function withErrorHandler<T = any>(handler: RouteHandler<T>) {
  return async (request: NextRequest, context: RouteContext<T>) => {
    try {
      return await handler(request, context);
    } catch (error) {
      if (error instanceof ApiError) {
        return NextResponse.json(
          { error: error.message, code: error.status },
          { status: error.status },
        );
      }

      console.error("[API Error]:", error);

      return NextResponse.json(
        { error: "Internal Server Error", code: 500 },
        { status: 500 },
      );
    }
  };
}

// ─── Ticket Sync API Handler ────────────────────────────────────────────────

export interface SyncTicketPayload {
  txHash: string;
  eventId: string;
  contractId?: string;
  buyerAddress?: string;
}

export interface SyncTicketResponse {
  success: boolean;
  ticketId?: string;
  message?: string;
}

/**
 * Syncs a confirmed Soroban transaction with the backend database.
 * Called after a ticket mint transaction is confirmed on-chain.
 *
 * Implements retry logic with exponential backoff (up to 3 attempts)
 * for transient 5xx errors. Permanent failures (4xx) are returned immediately.
 *
 * @param payload - Sync payload with transaction hash, event ID, etc.
 * @param maxRetries - Maximum number of retry attempts (default: 3)
 * @param baseDelayMs - Initial delay for exponential backoff (default: 1000ms)
 * @returns Response from backend with sync status
 * @throws ApiError on permanent failures
 */
export async function syncTicketPurchase(
  payload: SyncTicketPayload,
  maxRetries: number = 3,
  baseDelayMs: number = 1000,
): Promise<SyncTicketResponse> {
  let lastError: Error | null = null;

  for (let attempt = 0; attempt <= maxRetries; attempt++) {
    try {
      const response = await fetch("/api/payments/ticket/sync", {
        method: "POST",
        headers: {
          "Content-Type": "application/json",
        },
        body: JSON.stringify(payload),
      });

      // Permanent client errors — fail immediately
      if (response.status >= 400 && response.status < 500) {
        const data = await response.json();
        throw new ApiError(
          data.error || "Ticket sync failed",
          response.status,
        );
      }

      // Success
      if (response.ok) {
        return await response.json();
      }

      // Transient server errors — retry if attempts remain
      if (response.status >= 500) {
        const data = await response.json();
        lastError = new Error(
          `Backend sync failed (${response.status}): ${data.error || "Unknown error"}`,
        );

        if (attempt < maxRetries) {
          // Exponential backoff: delay = baseDelayMs * 2^attempt
          const delayMs = baseDelayMs * Math.pow(2, attempt);
          console.warn(
            `[Ticket Sync] Attempt ${attempt + 1}/${maxRetries + 1} failed. ` +
              `Retrying in ${delayMs}ms...`,
          );
          await new Promise((resolve) => setTimeout(resolve, delayMs));
          continue;
        } else {
          throw lastError;
        }
      }

      // Unexpected status
      throw new ApiError(
        `Unexpected backend response: ${response.status}`,
        response.status,
      );
    } catch (error) {
      lastError = error instanceof Error ? error : new Error(String(error));

      // Don't retry if it's an ApiError with 4xx status
      if (error instanceof ApiError && error.status >= 400 && error.status < 500) {
        throw error;
      }

      // Network errors or ApiError with 5xx — retry if attempts remain
      if (attempt < maxRetries) {
        const delayMs = baseDelayMs * Math.pow(2, attempt);
        console.warn(
          `[Ticket Sync] Attempt ${attempt + 1}/${maxRetries + 1} failed with error: ${error}. ` +
            `Retrying in ${delayMs}ms...`,
        );
        await new Promise((resolve) => setTimeout(resolve, delayMs));
        continue;
      }

      // Max retries exhausted
      throw lastError;
    }
  }

  // Should not reach here, but throw final error as fallback
  throw (
    lastError ||
    new Error("Ticket sync failed: max retries exceeded with no error captured")
  );
}
