import { describe, it, expect, vi, beforeEach } from "vitest";
import { NextRequest } from "next/server";
import { GET } from "@/app/api/v1/events/[id]/export-attendees/route";

describe("GET /api/v1/events/[id]/export-attendees", () => {
  beforeEach(() => {
    vi.restoreAllMocks();
  });

  it("returns 401 when auth_token cookie is missing", async () => {
    const req = new NextRequest("http://localhost:3000/api/v1/events/event-123/export-attendees");
    const context = { params: Promise.resolve({ id: "event-123" }) };

    const res = await GET(req, context);
    expect(res.status).toBe(401);
    const body = await res.json();
    expect(body.message).toMatch(/Authentication required/i);
  });

  it("forwards Authorization header and streams CSV from backend", async () => {
    const csvContent = "owner_wallet,buyer_wallet,quantity,created_at\n0x1,0x2,1,2026-01-01T00:00:00Z";

    global.fetch = vi.fn().mockResolvedValue(
      new Response(csvContent, {
        status: 200,
        headers: {
          "Content-Type": "text/csv; charset=utf-8",
          "Content-Disposition": 'attachment; filename="attendees-event-123.csv"',
        },
      })
    );

    const req = new NextRequest("http://localhost:3000/api/v1/events/event-123/export-attendees", {
      headers: {
        cookie: "auth_token=valid-jwt-token",
      },
    });
    const context = { params: Promise.resolve({ id: "event-123" }) };

    const res = await GET(req, context);
    expect(res.status).toBe(200);
    expect(res.headers.get("content-type")).toContain("text/csv");
    expect(res.headers.get("content-disposition")).toBe('attachment; filename="attendees-event-123.csv"');

    const text = await res.text();
    expect(text).toBe(csvContent);
  });

  it("returns upstream error status if backend responds with failure", async () => {
    global.fetch = vi.fn().mockResolvedValue(
      new Response("Forbidden: Not event organizer", {
        status: 403,
        headers: { "Content-Type": "text/plain" },
      })
    );

    const req = new NextRequest("http://localhost:3000/api/v1/events/event-123/export-attendees", {
      headers: {
        cookie: "auth_token=invalid-owner-token",
      },
    });
    const context = { params: Promise.resolve({ id: "event-123" }) };

    const res = await GET(req, context);
    expect(res.status).toBe(403);
    const text = await res.text();
    expect(text).toContain("Forbidden");
  });
});
