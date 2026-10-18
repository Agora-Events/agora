import { NextRequest, NextResponse } from "next/server";
import { getAuthFromRequest } from "@/lib/auth";
import { prisma } from "@/lib/prisma";

const BACKEND_URL =
  process.env.BACKEND_URL ??
  process.env.NEXT_PUBLIC_API_URL ??
  "http://localhost:3001";

export async function GET(request: NextRequest) {
  const backendUrl = new URL("/api/v1/events", BACKEND_URL);
  request.nextUrl.searchParams.forEach((value, key) => {
    backendUrl.searchParams.set(key, value);
  });
  try {
    const response = await fetch(backendUrl, { cache: "no-store" });
    return new NextResponse(await response.text(), {
      status: response.status,
      headers: {
        "Content-Type": response.headers.get("content-type") ?? "application/json",
      },
    });
  } catch (error) {
    console.error("Event list proxy failed:", error);
    return NextResponse.json({ message: "Unable to load events" }, { status: 502 });
  }
}

export async function POST(request: NextRequest) {
  const auth = getAuthFromRequest(request);
  const token = request.cookies.get("auth_token")?.value;
  if (!auth?.sub || !token) {
    return NextResponse.json({ message: "Authentication required" }, { status: 401 });
  }

  let payload: Record<string, unknown>;
  try {
    payload = await request.json();
  } catch {
    return NextResponse.json({ message: "Invalid JSON request body" }, { status: 400 });
  }

  try {
    const organizers = await prisma.$queryRaw<Array<{ id: string }>>`
      SELECT id::text AS id
      FROM organizers
      WHERE wallet_address = ${auth.sub}
      LIMIT 1
    `;
    const organizer = organizers[0];
    if (!organizer) {
      return NextResponse.json(
        { message: "Your wallet is not associated with an organizer account" },
        { status: 404 },
      );
    }

    const backendResponse = await fetch(`${BACKEND_URL}/api/v1/events`, {
      method: "POST",
      headers: {
        Authorization: `Bearer ${token}`,
        "Content-Type": "application/json",
      },
      body: JSON.stringify({
        ...payload,
        organizer_id: organizer.id,
        host_email: auth.email ?? payload.host_email,
      }),
      cache: "no-store",
    });

    return new NextResponse(await backendResponse.text(), {
      status: backendResponse.status,
      headers: {
        "Content-Type": backendResponse.headers.get("content-type") ?? "application/json",
      },
    });
  } catch (error) {
    console.error("Event creation proxy failed:", error);
    return NextResponse.json({ message: "Unable to create event" }, { status: 502 });
  }
}