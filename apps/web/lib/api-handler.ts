import { NextRequest, NextResponse } from "next/server";
import { ApiError } from "./api-errors";

type RouteHandler = (
  request: NextRequest,
  context: any,
) => Promise<NextResponse> | NextResponse;

export function withErrorHandler(handler: RouteHandler) {
  return async (request: NextRequest, context: any) => {
    try {
      return await handler(request, context);
    } catch (error) {
      if (error instanceof ApiError) {
        return NextResponse.json(
          { error: error.message, code: error.status },
          { status: error.status },
        );
      }

      console.error("[API Error]:", error);

      return NextResponse.json(
        { error: "Internal Server Error", code: 500 },
        { status: 500 },
      );
    }
  };
}
