import { test, expect, type Page } from "@playwright/test";
import AxeBuilder from "@axe-core/playwright";

/**
 * Automated accessibility regression scan for key public routes, using
 * axe-core (already exercised at the unit level in
 * `__tests__/layout-accessibility.test.ts`) driven end-to-end via
 * @axe-core/playwright against a real rendered page.
 *
 * Only "serious" and "critical" impact violations fail the build. Lower
 * severities (minor/moderate) are still logged for visibility but don't
 * fail CI — they're usually cosmetic and would otherwise create noise that
 * trains people to ignore this suite.
 *
 * If a real, unavoidable violation shows up from a third-party
 * script/embed we don't control (analytics, chat widget, payment iframe,
 * etc.), add it to KNOWN_VIOLATION_ALLOWLIST below with a comment
 * explaining why — do NOT raise/lower FAIL_ON_IMPACT to paper over it.
 * First-party violations should be fixed, not allow-listed.
 */

type AllowlistEntry = {
  /** Route this entry applies to (must match one of the `routes` below). */
  route: string;
  /** The axe-core rule id being suppressed, e.g. "color-contrast". */
  ruleId: string;
  /** Why this specific violation is accepted — link a tracking issue if one exists. */
  reason: string;
};

// Intentionally empty for now — no third-party violation has needed to be
// allow-listed yet. Example of how to add one when it does:
//
// const KNOWN_VIOLATION_ALLOWLIST: AllowlistEntry[] = [
//   {
//     route: "/",
//     ruleId: "color-contrast",
//     reason:
//       "Low-contrast text injected by the PostHog feedback widget, which " +
//       "we don't control. Tracked in AGORA-1234.",
//   },
// ];
const KNOWN_VIOLATION_ALLOWLIST: AllowlistEntry[] = [];

/** Impacts that fail the test when not explicitly allow-listed. */
const FAIL_ON_IMPACT = new Set(["serious", "critical"]);

async function assertNoSeriousViolations(page: Page, route: string) {
  const results = await new AxeBuilder({ page }).analyze();

  const actionable = results.violations.filter(
    (violation) => violation.impact && FAIL_ON_IMPACT.has(violation.impact)
  );
  const allowed = actionable.filter((violation) =>
    KNOWN_VIOLATION_ALLOWLIST.some(
      (entry) => entry.route === route && entry.ruleId === violation.id
    )
  );
  const blocking = actionable.filter((violation) => !allowed.includes(violation));

  const otherViolations = results.violations.filter(
    (violation) => !actionable.includes(violation)
  );
  if (otherViolations.length > 0) {
    console.log(
      `[a11y] ${route}: ${otherViolations.length} lower-severity violation(s) (not failing):\n` +
        otherViolations
          .map((v) => `  - ${v.id} (${v.impact}): ${v.help}`)
          .join("\n")
    );
  }

  if (blocking.length > 0) {
    const details = blocking
      .flatMap((violation) =>
        violation.nodes.map(
          (node) =>
            `  - [${violation.impact}] ${violation.id}: ${node.target.join(", ")}\n` +
            `    ${violation.help}`
        )
      )
      .join("\n");
    console.error(`[a11y] ${route}: blocking violations:\n${details}`);
  }

  expect(
    blocking.map((v) => v.id),
    `Accessibility violations on ${route}:\n${blocking
      .flatMap((violation) =>
        violation.nodes.map(
          (node) => `[${violation.impact}] ${violation.id}: ${node.target.join(", ")}`
        )
      )
      .join("\n")}`
  ).toHaveLength(0);
}

const STATIC_ROUTES = ["/", "/discover", "/help", "/pricing"];

test.describe("Accessibility scan (axe-core)", () => {
  for (const route of STATIC_ROUTES) {
    test(`${route} has no serious or critical accessibility violations`, async ({
      page,
    }) => {
      await page.goto(route);
      await assertNoSeriousViolations(page, route);
    });
  }

  test("an event detail page has no serious or critical accessibility violations", async ({
    page,
    baseURL,
  }) => {
    // Prefer a real event id from the live discover API so we scan an
    // actual event page rather than a guessed slug; fall back to a
    // representative path when no events exist yet (e.g. a fresh DB).
    let eventPath = "/events/example-event";
    try {
      const response = await page.request.get(
        `${baseURL ?? "http://localhost:3000"}/api/events/discover`
      );
      if (response.ok()) {
        const data = await response.json();
        const firstId = data?.popularEvents?.[0]?.id;
        if (firstId) {
          eventPath = `/events/${firstId}`;
        }
      }
    } catch {
      // Network/parse failure — fall back to the representative path above.
    }

    await page.goto(eventPath);
    await assertNoSeriousViolations(page, eventPath);
  });
});
