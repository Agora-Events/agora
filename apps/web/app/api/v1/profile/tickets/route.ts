import { NextRequest, NextResponse } from "next/server";
import { getAuthFromRequest } from "@/lib/auth";
import { withErrorHandler } from "@/lib/api-handler";
import { throwApiError } from "@/lib/api-errors";

const BACKEND_URL = process.env.NEXT_PUBLIC_API_URL ?? "http://localhost:3001";

/**
 * GET /api/v1/profile/tickets
 *
 * Proxies the request to the Rust backend's wallet ticket aggregation
 * endpoint and returns the grouped upcoming / past ticket collections.
 *
 * Issue #1122
 */
export const GET = withErrorHandler(async (request: NextRequest) => {
  const auth = getAuthFromRequest(request);
  if (!auth?.sub) {
    throwApiError("Unauthorized", 401);
  }

  const backendRes = await fetch(`${BACKEND_URL}/api/v1/profile/tickets`, {
    headers: {
      Authorization: request.headers.get("Authorization") ?? "",
      "Content-Type": "application/json",
      Cookie: request.headers.get("Cookie") ?? "",
    },
    cache: "no-store",
  });

  if (!backendRes.ok) {
    const text = await backendRes.text().catch(() => "");
    throwApiError(text || "Failed to fetch wallet tickets", backendRes.status);
  }

  const data = await backendRes.json();
  return NextResponse.json(data);
});
