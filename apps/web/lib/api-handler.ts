import { NextRequest, NextResponse } from "next/server";
import { ApiError } from "./api-errors";

type RouteContext<T = any> = {
  params: Promise<T>;
};

type RouteHandler<T = any> = (
  request: NextRequest,
  context: RouteContext<T>,
) => Promise<NextResponse> | NextResponse;

export function withErrorHandler<T = any>(handler: RouteHandler<T>) {
  return async (request: NextRequest, context: RouteContext<T>) => {
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
