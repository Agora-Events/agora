import { describe, it, expect, vi, beforeEach } from "vitest";

// In-memory cookie jar backing the next/headers mock, so the route's
// `cookies()` calls can be observed and primed from the test.
let cookieJar: Map<string, string>;

vi.mock("next/headers", () => ({
  cookies: async () => ({
    get: (name: string) => {
      const value = cookieJar.get(name);
      return value === undefined ? undefined : { name, value };
    },
    set: (name: string, value: string) => {
      cookieJar.set(name, value);
    },
    delete: (name: string) => {
      cookieJar.delete(name);
    },
  }),
}));

function makeIdToken(payload: Record<string, unknown>): string {
  const header = Buffer.from(JSON.stringify({ alg: "none", typ: "JWT" })).toString(
    "base64url"
  );
  const body = Buffer.from(JSON.stringify(payload)).toString("base64url");
  return `${header}.${body}.`;
}

describe("GET /api/auth/apple", () => {
  beforeEach(() => {
    cookieJar = new Map();
    vi.restoreAllMocks();
  });

  it("starts the redirect with a freshly generated state stored in a cookie", async () => {
    const { GET } = await import("@/app/api/auth/apple/route");

    const res = await GET(new Request("http://localhost:3000/api/auth/apple"));

    expect(res.status).toBeGreaterThanOrEqual(300);
    expect(res.status).toBeLessThan(400);
    const location = new URL(res.headers.get("location")!);
    expect(location.hostname).toBe("appleid.apple.com");

    const state = location.searchParams.get("state");
    expect(state).toBeTruthy();
    expect(cookieJar.get("apple_oauth_state")).toBe(state);
  });

  it("accepts the GET callback when the returned state matches the cookie", async () => {
    cookieJar.set("apple_oauth_state", "matching-state-value");

    global.fetch = vi.fn().mockResolvedValue(
      new Response(
        JSON.stringify({ id_token: makeIdToken({ email: "a@b.com", sub: "123" }) }),
        { status: 200, headers: { "Content-Type": "application/json" } }
      )
    );

    const { GET } = await import("@/app/api/auth/apple/route");
    const res = await GET(
      new Request(
        "http://localhost:3000/api/auth/apple?code=abc123&state=matching-state-value"
      )
    );

    expect(res.headers.get("location")).toContain("/home");
    expect(cookieJar.has("apple_oauth_state")).toBe(false);
  });

  it("rejects the GET callback when the returned state does not match the cookie", async () => {
    cookieJar.set("apple_oauth_state", "expected-state");
    global.fetch = vi.fn();

    const { GET } = await import("@/app/api/auth/apple/route");
    const res = await GET(
      new Request("http://localhost:3000/api/auth/apple?code=abc123&state=wrong-state")
    );

    expect(res.headers.get("location")).toContain("/auth?error=");
    expect(global.fetch).not.toHaveBeenCalled();
    expect(cookieJar.has("apple_oauth_state")).toBe(false);
  });

  it("rejects the GET callback when the state parameter is missing", async () => {
    cookieJar.set("apple_oauth_state", "expected-state");
    global.fetch = vi.fn();

    const { GET } = await import("@/app/api/auth/apple/route");
    const res = await GET(
      new Request("http://localhost:3000/api/auth/apple?code=abc123")
    );

    expect(res.headers.get("location")).toContain("/auth?error=");
    expect(global.fetch).not.toHaveBeenCalled();
  });

  it("rejects the POST (form_post) callback when the state does not match the cookie", async () => {
    cookieJar.set("apple_oauth_state", "expected-state");
    global.fetch = vi.fn();

    const { POST } = await import("@/app/api/auth/apple/route");
    const form = new FormData();
    form.set("code", "abc123");
    form.set("state", "wrong-state");

    const res = await POST(
      new Request("http://localhost:3000/api/auth/apple", {
        method: "POST",
        body: form,
      })
    );

    expect(res.headers.get("location")).toContain("/auth?error=");
    expect(global.fetch).not.toHaveBeenCalled();
    expect(cookieJar.has("apple_oauth_state")).toBe(false);
  });

  it("accepts the POST (form_post) callback when the state matches the cookie", async () => {
    cookieJar.set("apple_oauth_state", "matching-state-value");

    global.fetch = vi.fn().mockResolvedValue(
      new Response(
        JSON.stringify({ id_token: makeIdToken({ email: "a@b.com", sub: "123" }) }),
        { status: 200, headers: { "Content-Type": "application/json" } }
      )
    );

    const { POST } = await import("@/app/api/auth/apple/route");
    const form = new FormData();
    form.set("code", "abc123");
    form.set("state", "matching-state-value");

    const res = await POST(
      new Request("http://localhost:3000/api/auth/apple", {
        method: "POST",
        body: form,
      })
    );

    expect(res.headers.get("location")).toContain("/home");
    expect(cookieJar.has("apple_oauth_state")).toBe(false);
  });
});
