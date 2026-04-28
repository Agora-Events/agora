import { NextRequest, NextResponse } from "next/server";
import { getAuthFromRequest } from "@/lib/auth";
import { listEvents } from "@/lib/events-store";

export async function GET(request: NextRequest) {
  const auth = getAuthFromRequest(request);
  if (!auth?.email) {
    return NextResponse.json({ error: "Unauthorized" }, { status: 401 });
  }

  const items = listEvents().filter((event) => event.followersOnly);
  return NextResponse.json({ items });
}
