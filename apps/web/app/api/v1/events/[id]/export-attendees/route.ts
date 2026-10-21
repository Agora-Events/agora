import { NextRequest, NextResponse } from "next/server";

const BACKEND_URL =
  process.env.BACKEND_URL ?? process.env.NEXT_PUBLIC_API_URL ?? "http://localhost:3001";

interface RouteContext {
  params: Promise<{ id: string }>;
}

export async function GET(request: NextRequest, context: RouteContext) {
  const { id } = await context.params;

  if (!id) {
    return NextResponse.json({ message: "Event ID is required" }, { status: 400 });
  }

  const token = request.cookies.get("auth_token")?.value;
  if (!token) {
    return NextResponse.json({ message: "Authentication required" }, { status: 401 });
  }

  try {
    const backendUrl = new URL(`/api/v1/events/${id}/export-attendees`, BACKEND_URL);

    const backendRes = await fetch(backendUrl.toString(), {
      method: "GET",
      headers: {
        Authorization: `Bearer ${token}`,
      },
      cache: "no-store",
    });

    if (!backendRes.ok) {
      const errorText = await backendRes.text().catch(() => "");
      return new NextResponse(errorText || "Failed to export attendees", {
        status: backendRes.status,
        headers: {
          "Content-Type": backendRes.headers.get("content-type") ?? "text/plain",
        },
      });
    }

    const csvData = await backendRes.text();
    const contentDisposition =
      backendRes.headers.get("content-disposition") ?? `attachment; filename="attendees-${id}.csv"`;

    return new NextResponse(csvData, {
      status: 200,
      headers: {
        "Content-Type": "text/csv; charset=utf-8",
        "Content-Disposition": contentDisposition,
        "Cache-Control": "no-store, no-cache, must-revalidate",
      },
    });
  } catch (error) {
    console.error("Attendee export proxy failed:", error);
    return NextResponse.json(
      { message: "Unable to export attendees at this time" },
      { status: 502 }
    );
  }
}
