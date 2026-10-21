import { describe, expect, it } from "vitest";
import robots from "@/app/robots";

describe("robots.ts", () => {
  const result = robots();
  const rules = Array.isArray(result.rules) ? result.rules[0] : result.rules;

  it("allows crawling the site root for every user agent", () => {
    expect(rules?.userAgent).toBe("*");
    expect(rules?.allow).toBe("/");
  });

  it("disallows API, fixture and per-user routes", () => {
    const disallow = rules?.disallow as string[];
    expect(disallow).toEqual(
      expect.arrayContaining(["/api/", "/__visual_fixtures__", "/auth", "/settings"]),
    );
  });

  it("advertises the sitemap on the canonical origin", () => {
    // Must match `metadataBase` in app/layout.tsx, or the sitemap and the
    // canonical URLs disagree about which origin is authoritative.
    expect(result.sitemap).toBe("https://agora.events/sitemap.xml");
  });

  it("does not disallow the routes we want indexed", () => {
    const disallow = rules?.disallow as string[];
    ["/discover", "/pricing", "/faqs", "/events"].forEach((route) => {
      expect(disallow).not.toContain(route);
    });
  });
});
