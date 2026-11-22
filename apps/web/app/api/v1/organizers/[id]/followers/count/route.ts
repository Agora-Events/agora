import { type NextRequest, NextResponse } from "next/server";

const BACKEND_URL =
  process.env.BACKEND_URL ??
  process.env.NEXT_PUBLIC_API_URL ??
  "http://localhost:3001";

export async function GET(
  _request: NextRequest,
  { params }: { params: Promise<{ id: string }> },
) {
  const { id } = await params;

  try {
    const backendUrl = new URL(
      `/api/v1/organizers/${encodeURIComponent(id)}/followers/count`,
      BACKEND_URL,
    );
    const response = await fetch(backendUrl, { cache: "no-store" });
    if (!response.ok) {
      return NextResponse.json(
        { message: "Could not load follower count" },
        { status: response.status },
      );
    }
    return NextResponse.json(await response.json());
  } catch {
    return NextResponse.json(
      { message: "Could not load follower count" },
      { status: 502 },
    );
  }
}
