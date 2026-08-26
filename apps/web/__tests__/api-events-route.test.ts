import { describe, it, expect, vi, beforeEach } from "vitest";
import { NextRequest } from "next/server";

vi.mock("@/lib/prisma", () => ({
  prisma: {
    event: {
      findMany: vi.fn(),
    },
  },
}));

vi.mock("@/lib/auth", () => ({
  getAuthFromRequest: vi.fn(),
}));

import { prisma } from "@/lib/prisma";
import { getAuthFromRequest } from "@/lib/auth";
import { GET } from "@/app/api/events/route";

const findManyMock = prisma.event.findMany as unknown as ReturnType<
  typeof vi.fn
>;
const getAuthMock = getAuthFromRequest as unknown as ReturnType<typeof vi.fn>;

function makeRequest(pathAndQuery: string) {
  return new NextRequest(`http://localhost:3000${pathAndQuery}`);
}

// A no-op params context, satisfying the RouteHandler<T> signature the
// route is wrapped with (withErrorHandler always passes a context object).
const emptyContext = { params: Promise.resolve({}) };

describe("GET /api/events", () => {
  beforeEach(() => {
    vi.clearAllMocks();
    findManyMock.mockResolvedValue([]);
    getAuthMock.mockReturnValue(null);
  });

  it("returns 400 for an invalid tab value", async () => {
    const response = await GET(makeRequest("/api/events?tab=bogus"), emptyContext);

    expect(response.status).toBe(400);
    const body = await response.json();
    expect(body.error).toMatch(/invalid tab/i);
    expect(findManyMock).not.toHaveBeenCalled();
  });

  it("returns 401 for type=my without auth", async () => {
    getAuthMock.mockReturnValue(null);

    const response = await GET(makeRequest("/api/events?type=my"), emptyContext);

    expect(response.status).toBe(401);
    const body = await response.json();
    expect(body.error).toMatch(/unauthorized/i);
    expect(findManyMock).not.toHaveBeenCalled();
  });

  it("builds a startsAt: { lt: now } where-clause for type=my&tab=past", async () => {
    getAuthMock.mockReturnValue({ email: "host@agora.dev" });

    const before = new Date();
    const response = await GET(
      makeRequest("/api/events?type=my&tab=past"),
      emptyContext,
    );
    const after = new Date();

    expect(response.status).toBe(200);
    expect(findManyMock).toHaveBeenCalledTimes(1);

    const callArg = findManyMock.mock.calls[0][0];
    expect(callArg.where.hostEmail).toBe("host@agora.dev");
    expect(callArg.where.startsAt).toHaveProperty("lt");
    expect(callArg.where.startsAt.lt).toBeInstanceOf(Date);
    expect(callArg.where.startsAt.lt.getTime()).toBeGreaterThanOrEqual(
      before.getTime(),
    );
    expect(callArg.where.startsAt.lt.getTime()).toBeLessThanOrEqual(
      after.getTime(),
    );
    expect(callArg.orderBy).toEqual({ startsAt: "asc" });

    const body = await response.json();
    expect(body.tab).toBe("past");
    expect(body.type).toBe("my");
  });

  it("builds a startsAt: { gte: now } where-clause for type=my&tab=upcoming", async () => {
    getAuthMock.mockReturnValue({ email: "host@agora.dev" });

    const response = await GET(
      makeRequest("/api/events?type=my&tab=upcoming"),
      emptyContext,
    );

    expect(response.status).toBe(200);
    const callArg = findManyMock.mock.calls[0][0];
    expect(callArg.where.hostEmail).toBe("host@agora.dev");
    expect(callArg.where.startsAt).toHaveProperty("gte");
    expect(callArg.where.startsAt.gte).toBeInstanceOf(Date);
    expect(callArg.orderBy).toEqual({ startsAt: "asc" });
  });

  it("treats type=my&tab=hosting the same as upcoming (gte now)", async () => {
    getAuthMock.mockReturnValue({ email: "host@agora.dev" });

    const response = await GET(
      makeRequest("/api/events?type=my&tab=hosting"),
      emptyContext,
    );

    expect(response.status).toBe(200);
    const callArg = findManyMock.mock.calls[0][0];
    expect(callArg.where.startsAt).toHaveProperty("gte");
  });

  it("returns all events ordered by startsAt ascending for the default (no params) path", async () => {
    const events = [{ id: "1" }, { id: "2" }];
    findManyMock.mockResolvedValue(events);

    const response = await GET(makeRequest("/api/events"), emptyContext);

    expect(response.status).toBe(200);
    const body = await response.json();
    expect(body.items).toEqual(events);
    expect(body.tab).toBe("upcoming");
    expect(body.type).toBe("all");

    expect(findManyMock).toHaveBeenCalledTimes(1);
    const callArg = findManyMock.mock.calls[0][0];
    expect(callArg).toEqual({ orderBy: { startsAt: "asc" } });
    expect(callArg.where).toBeUndefined();

    // No real database should ever be touched -- the prisma client is fully
    // mocked above via vi.mock("@/lib/prisma", ...).
    expect(getAuthMock).not.toHaveBeenCalled();
  });
});
