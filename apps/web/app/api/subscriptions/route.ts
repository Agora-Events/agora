import { NextRequest, NextResponse } from "next/server";
import { getAuthFromRequest } from "@/lib/auth";
import { withErrorHandler } from "@/lib/api-handler";
import { throwApiError } from "@/lib/api-errors";

/**
 * GET /api/subscriptions
 * Returns the authenticated user's Pro subscription status and active series passes.
 */
export const GET = withErrorHandler(async (request: NextRequest) => {
  const auth = getAuthFromRequest(request);
  if (!auth?.email) {
    throwApiError("Unauthorized", 401);
  }

  // Stub: replace with real DB / Soroban RPC queries when contract is wired up.
  return NextResponse.json({
    pro: {
      active: false,
      billingCycleEndsAt: null,
      priceUsdc: "9.99",
    },
    seriesPasses: [] as SeriesPassDTO[],
  });
});

/**
 * POST /api/subscriptions
 * Subscribe to Agora Pro or purchase a series pass.
 *
 * Body: { action: "subscribe_pro" | "cancel_pro" | "buy_pass", passId?: string }
 */
export const POST = withErrorHandler(async (request: NextRequest) => {
  const auth = getAuthFromRequest(request);
  if (!auth?.email) {
    throwApiError("Unauthorized", 401);
  }

  let body: { action?: string; passId?: string };
  try {
    body = await request.json();
  } catch {
    throwApiError("Invalid JSON payload", 400);
  }

  const { action, passId } = body;

  if (!action || !["subscribe_pro", "cancel_pro", "buy_pass"].includes(action)) {
    throwApiError("Invalid action. Must be subscribe_pro, cancel_pro, or buy_pass.", 400);
  }

  if (action === "buy_pass" && !passId) {
    throwApiError("passId is required for buy_pass action", 400);
  }

  // Stub responses — replace with real contract invocation / DB writes.
  if (action === "subscribe_pro") {
    const endsAt = new Date(Date.now() + 30 * 24 * 60 * 60 * 1000).toISOString();
    return NextResponse.json({
      success: true,
      pro: { active: true, billingCycleEndsAt: endsAt, priceUsdc: "9.99" },
    });
  }

  if (action === "cancel_pro") {
    return NextResponse.json({
      success: true,
      pro: { active: false, billingCycleEndsAt: null, priceUsdc: "9.99" },
    });
  }

  // buy_pass
  return NextResponse.json({
    success: true,
    pass: {
      id: passId,
      eventName: "Series Pass",
      validFrom: new Date().toISOString(),
      validUntil: new Date(Date.now() + 90 * 24 * 60 * 60 * 1000).toISOString(),
      totalUses: 10,
      usedUses: 0,
    } satisfies SeriesPassDTO,
  });
});

// ─── Shared DTO type (also used by the page) ──────────────────────────────────
export interface SeriesPassDTO {
  id: string;
  eventName: string;
  validFrom: string;
  validUntil: string;
  totalUses: number;
  usedUses: number;
}
