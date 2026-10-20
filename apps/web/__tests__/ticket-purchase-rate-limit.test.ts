import { describe, it, expect, vi, beforeEach } from "vitest";
import { NextRequest } from "next/server";

vi.mock("@/lib/prisma", () => ({
  prisma: {
    event: { findUnique: vi.fn(), update: vi.fn() },
    ticket: { create: vi.fn() },
    custodialWallet: { create: vi.fn() },
    $transaction: vi.fn(),
  },
}));

vi.mock("@/utils/stellar", () => ({
  mintTicket: vi.fn(),
}));

import { POST } from "@/app/api/payments/ticket/route";
import { createRateLimiter, ticketPurchaseRateLimiter } from "@/lib/rate-limit";

const emptyContext = { params: Promise.resolve({}) };

// An invalid body keeps each request on the cheap 400 validation path, which
// is exactly the traffic a spamming bot would generate.
function makePurchaseRequest(ip: string) {
  return new NextRequest("http://localhost:3000/api/payments/ticket", {
    method: "POST",
    headers: { "Content-Type": "application/json", "x-forwarded-for": ip },
    body: JSON.stringify({ quantity: 1, buyerWallet: "GBUYER" }),
  });
}

describe("POST /api/payments/ticket rate limiting", () => {
  beforeEach(() => {
    ticketPurchaseRateLimiter.reset();
  });

  it("returns 429 with Retry-After once rapid calls exceed the limit", async () => {
    const ip = "203.0.113.10";

    for (let i = 0; i < ticketPurchaseRateLimiter.limit; i++) {
      const response = await POST(makePurchaseRequest(ip), emptyContext);
      expect(response.status).toBe(400);
    }

    const blocked = await POST(makePurchaseRequest(ip), emptyContext);
    expect(blocked.status).toBe(429);
    expect(Number(blocked.headers.get("Retry-After"))).toBeGreaterThanOrEqual(1);
    expect(blocked.headers.get("X-RateLimit-Remaining")).toBe("0");
    const body = await blocked.json();
    expect(body.code).toBe(429);
  });

  it("tracks each client IP independently", async () => {
    for (let i = 0; i <= ticketPurchaseRateLimiter.limit; i++) {
      await POST(makePurchaseRequest("203.0.113.20"), emptyContext);
    }

    const otherClient = await POST(makePurchaseRequest("203.0.113.21"), emptyContext);
    expect(otherClient.status).toBe(400);
  });
});

describe("createRateLimiter", () => {
  it("allows requests again after the window elapses", () => {
    const limiter = createRateLimiter({ limit: 2, windowMs: 60_000 });
    const start = 1_000_000;

    expect(limiter.check("ip", start).allowed).toBe(true);
    expect(limiter.check("ip", start + 1).allowed).toBe(true);

    const blocked = limiter.check("ip", start + 2);
    expect(blocked.allowed).toBe(false);
    expect(blocked.retryAfterSeconds).toBe(60);

    expect(limiter.check("ip", start + 60_000).allowed).toBe(true);
  });
});
