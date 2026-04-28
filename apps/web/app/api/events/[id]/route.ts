import { NextRequest, NextResponse } from "next/server";
import { getEventById } from "@/lib/events-store";

type Params = {
  params: Promise<{ id: string }>;
};

export async function GET(_request: NextRequest, { params }: Params) {
  const { id } = await params;
  const event = getEventById(id);

  if (!event) {
    return NextResponse.json({ error: "Event not found" }, { status: 404 });
  }

  return NextResponse.json({ event });
}
