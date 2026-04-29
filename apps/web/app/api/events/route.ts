import { NextRequest, NextResponse } from "next/server";
import { prisma } from "@/lib/prisma";
import { getAuthFromRequest } from "@/lib/auth";
import { withErrorHandler } from "@/lib/api-handler";
import { throwApiError } from "@/lib/api-errors";

const VALID_TABS = new Set(["upcoming", "hosting", "past"]);

export const GET = withErrorHandler(async (request: NextRequest) => {
  const { searchParams } = new URL(request.url);
  const type = searchParams.get("type");
  const tab = searchParams.get("tab") || "upcoming";

  if (!VALID_TABS.has(tab)) {
    throwApiError("Invalid tab value", 400);
  }

  const now = new Date();

  if (type === "my") {
    const auth = getAuthFromRequest(request);
    if (!auth?.email) {
      throwApiError("Unauthorized", 401);
    }

    // eslint-disable-next-line @typescript-eslint/no-explicit-any
    const whereClause: any = { hostEmail: auth.email };
    if (tab === "upcoming" || tab === "hosting") {
      whereClause.startsAt = { gte: now };
    } else {
      whereClause.startsAt = { lt: now };
    }

    const items = await prisma.event.findMany({
      where: whereClause,
      orderBy: { startsAt: "asc" },
    });

    return NextResponse.json({ items, tab, type: "my" });
  }

  const items = await prisma.event.findMany({
    orderBy: { startsAt: "asc" },
  });

  return NextResponse.json({ items, tab, type: type || "all" });
});

export const POST = withErrorHandler(async (request: NextRequest) => {
  const auth = getAuthFromRequest(request);
  if (!auth?.email) {
    throwApiError("Unauthorized", 401);
  }

  let payload: Record<string, unknown>;
  try {
    payload = await request.json();
  } catch {
    throwApiError("Invalid JSON payload", 400);
  }

  const requiredFields = ["title", "startsAt", "location", "category", "organizerName", "organizerWallet"];
  for (const field of requiredFields) {
    if (typeof payload[field] !== "string" || String(payload[field]).trim().length === 0) {
      throwApiError(`Invalid or missing field: ${field}`, 400);
    }
  }

  const created = await prisma.event.create({
    data: {
      title: payload.title as string,
      description: typeof payload.description === "string" ? payload.description : "",
      startsAt: new Date(payload.startsAt as string),
      location: payload.location as string,
      category: payload.category as string,
      organizerName: payload.organizerName as string,
      organizerWallet: payload.organizerWallet as string,
      imageUrl: typeof payload.imageUrl === "string" ? payload.imageUrl : undefined,
      ticketPrice: typeof payload.ticketPrice === "number" ? payload.ticketPrice : 0,
      totalTickets: typeof payload.totalTickets === "number" ? payload.totalTickets : 100,
      followersOnly: typeof payload.followersOnly === "boolean" ? payload.followersOnly : false,
      hostEmail: auth.email,
    },
  });

  return NextResponse.json({ event: created }, { status: 201 });
});


