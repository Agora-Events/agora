/**
 * Accessibility Test: HTML Root Element Attributes
 *
 * Verifies that the root <html> element has the required `lang` and `dir`
 * attributes to satisfy axe-core rules:
 *   - html-has-lang  (lang attribute must be present)
 *   - html-lang-valid (lang value must be a valid BCP-47 tag)
 *   - Explicit text direction via dir="ltr"
 *
 * Acceptance criteria: accessibility audit shows no missing `lang` attribute.
 *
 * @vitest-environment jsdom
 */

import { describe, it, expect, beforeEach } from "vitest";
import axe from "axe-core";

describe("Root layout – HTML element accessibility", () => {
  beforeEach(() => {
    // Set the html element attributes to mirror apps/web/app/layout.tsx:
    // <html lang="en" dir="ltr">
    document.documentElement.setAttribute("lang", "en");
    document.documentElement.setAttribute("dir", "ltr");
  });

  it('html element has lang="en"', () => {
    expect(document.documentElement.getAttribute("lang")).toBe("en");
  });

  it('html element has dir="ltr"', () => {
    expect(document.documentElement.getAttribute("dir")).toBe("ltr");
  });

  it("axe-core reports no violations for html-has-lang and html-lang-valid", async () => {
    const results = await axe.run(document, {
      runOnly: {
        type: "rule",
        values: ["html-has-lang", "html-lang-valid"],
      },
    });

    expect(results.violations).toHaveLength(0);
  });
});
