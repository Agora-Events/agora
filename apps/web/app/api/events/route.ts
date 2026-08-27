import { NextRequest, NextResponse } from "next/server";
import { prisma } from "@/lib/prisma";
import { Prisma } from "@prisma/client";
import { getAuthFromRequest } from "@/lib/auth";
import { withErrorHandler } from "@/lib/api-handler";
import { throwApiError } from "@/lib/api-errors";
import { slugify, withRandomSuffix } from "@/lib/slugify";

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

    const whereClause: Prisma.EventWhereInput = { hostEmail: auth.email };
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
    where: { status: "PUBLISHED" },
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

  const baseSlug = slugify(payload.title as string);
  let slug = baseSlug;
  let created;

  // Retry with a random suffix instead of failing the insert on a slug collision.
  for (let attempt = 0; ; attempt += 1) {
    try {
      created = await prisma.event.create({
        data: {
          slug,
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
      break;
    } catch (error) {
      const isSlugConflict =
        error instanceof Prisma.PrismaClientKnownRequestError &&
        error.code === "P2002" &&
        (error.meta?.target as string[] | undefined)?.includes("slug");

      if (!isSlugConflict || attempt >= 4) {
        throw error;
      }

      slug = withRandomSuffix(baseSlug);
    }
  }

  return NextResponse.json({ event: created }, { status: 201 });
});


