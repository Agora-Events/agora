import { describe, expect, it } from "vitest";
import { ApiError, throwApiError } from "@/lib/api-errors";

describe("ApiError", () => {
  it("keeps message and status values provided to constructor", () => {
    const error = new ApiError("NOT_FOUND", 404);

    expect(error).toBeInstanceOf(Error);
    expect(error).toBeInstanceOf(ApiError);
    expect(error.name).toBe("ApiError");
    expect(error.message).toBe("NOT_FOUND");
    expect(error.status).toBe(404);
  });

  it("uses default status 400 and supports empty message", () => {
    const error = new ApiError("");

    expect(error.message).toBe("");
    expect(error.status).toBe(400);
  });
});

describe("throwApiError", () => {
  it("throws an ApiError with explicit values", () => {
    expect(() => throwApiError("VALIDATION_FAILED", 422)).toThrow(ApiError);

    try {
      throwApiError("VALIDATION_FAILED", 422);
    } catch (error) {
      const apiError = error as ApiError;
      expect(apiError.message).toBe("VALIDATION_FAILED");
      expect(apiError.status).toBe(422);
      return;
    }

    throw new Error("Expected throwApiError to throw");
  });

  it("throws ApiError with default status when omitted", () => {
    try {
      throwApiError("INTERNAL_ERROR");
    } catch (error) {
      const apiError = error as ApiError;
      expect(apiError).toBeInstanceOf(ApiError);
      expect(apiError.message).toBe("INTERNAL_ERROR");
      expect(apiError.status).toBe(400);
      return;
    }

    throw new Error("Expected throwApiError to throw");
  });
});
