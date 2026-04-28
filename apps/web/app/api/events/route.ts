import { NextRequest, NextResponse } from "next/server";
import { createEvent, listEvents } from "@/lib/events-store";
import { getAuthFromRequest } from "@/lib/auth";

const VALID_TABS = new Set(["upcoming", "hosting", "past"]);

export async function GET(request: NextRequest) {
  const { searchParams } = new URL(request.url);
  const type = searchParams.get("type");
  const tab = searchParams.get("tab") || "upcoming";

  if (!VALID_TABS.has(tab)) {
    return NextResponse.json({ error: "Invalid tab value" }, { status: 400 });
  }

  if (type === "my") {
    const auth = getAuthFromRequest(request);
    if (!auth?.email) {
      return NextResponse.json({ error: "Unauthorized" }, { status: 401 });
    }

    const now = Date.now();
    const mine = listEvents().filter((event) => event.hostEmail === auth.email);

    const items = mine.filter((event) => {
      const startsAt = new Date(event.startsAt).getTime();
      if (tab === "upcoming" || tab === "hosting") {
        return startsAt >= now;
      }
      return startsAt < now;
    });

    return NextResponse.json({ items, tab, type: "my" });
  }

  return NextResponse.json({ items: listEvents(), tab, type: type || "all" });
}

export async function POST(request: NextRequest) {
  const auth = getAuthFromRequest(request);
  if (!auth?.email) {
    return NextResponse.json({ error: "Unauthorized" }, { status: 401 });
  }

  let payload: Record<string, unknown>;
  try {
    payload = await request.json();
  } catch {
    return NextResponse.json({ error: "Invalid JSON payload" }, { status: 400 });
  }

  const requiredFields = ["title", "startsAt", "location", "category", "organizerName", "organizerWallet"];
  for (const field of requiredFields) {
    if (typeof payload[field] !== "string" || String(payload[field]).trim().length === 0) {
      return NextResponse.json(
        { error: `Invalid or missing field: ${field}` },
        { status: 400 },
      );
    }
  }

  const created = createEvent(
    {
      title: payload.title as string,
      description: typeof payload.description === "string" ? payload.description : "",
      startsAt: payload.startsAt as string,
      location: payload.location as string,
      category: payload.category as string,
      organizerName: payload.organizerName as string,
      organizerWallet: payload.organizerWallet as string,
      imageUrl: typeof payload.imageUrl === "string" ? payload.imageUrl : undefined,
      ticketPrice: typeof payload.ticketPrice === "number" ? payload.ticketPrice : undefined,
      totalTickets: typeof payload.totalTickets === "number" ? payload.totalTickets : undefined,
      followersOnly: typeof payload.followersOnly === "boolean" ? payload.followersOnly : undefined,
    },
    auth.email,
  );

  return NextResponse.json({ event: created }, { status: 201 });
}
