import { describe, expect, it } from "vitest";
import { GET } from "../app/api/v1/events/[id]/resale/route";
import { NextRequest } from "next/server";

describe("GET /api/v1/events/:id/resale Endpoint", () => {
  it("returns active resale listings for a valid event ID", async () => {
    const req = new NextRequest("http://localhost:3000/api/v1/events/1/resale");
    const params = Promise.resolve({ id: "1" });
    const response = await GET(req, { params });

    expect(response.status).toBe(200);
    const json = await response.json();
    expect(json.success).toBe(true);
    expect(json.eventId).toBe("1");
    expect(Array.isArray(json.data)).toBe(true);
    expect(json.data.length).toBeGreaterThan(0);
    expect(json.data[0]).toHaveProperty("sellerName");
    expect(json.data[0]).toHaveProperty("price");
  });

  it("returns empty array for an event with no resale listings", async () => {
    const req = new NextRequest("http://localhost:3000/api/v1/events/non-existent/resale");
    const params = Promise.resolve({ id: "non-existent" });
    const response = await GET(req, { params });

    expect(response.status).toBe(200);
    const json = await response.json();
    expect(json.success).toBe(true);
    expect(json.eventId).toBe("non-existent");
    expect(Array.isArray(json.data)).toBe(true);
  });
});
