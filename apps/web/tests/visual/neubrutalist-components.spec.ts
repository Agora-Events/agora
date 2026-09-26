/**
 * Visual regression tests for core Neubrutalist UI components.
 *
 * Snapshots are captured in both light and dark mode via the "chromium-light"
 * and "chromium-dark" Playwright projects defined in playwright.config.ts.
 *
 * To update base snapshots after an intentional design change:
 *   npx playwright test --update-snapshots
 *
 * See docs/TESTING.md for the full contributor workflow.
 */

import { test, expect, type Page } from "@playwright/test";

// ─── Helpers ─────────────────────────────────────────────────────────────────

/** Navigate to the fixture page and wait for it to be fully hydrated. */
async function goToFixtures(page: Page) {
  await page.goto("/visual-fixtures");
  // Wait for the fixture page heading to be visible before taking any screenshot.
  await page.waitForSelector("h2", { state: "visible" });
  // Freeze CSS animations/transitions so snapshots are deterministic.
  await page.addStyleTag({
    content: `
      *, *::before, *::after {
        animation-duration: 0s !important;
        animation-delay: 0s !important;
        transition-duration: 0s !important;
        transition-delay: 0s !important;
      }
    `,
  });
}

/** Resolve the colour scheme label from the project name for snapshot naming. */
function themeLabel(projectName: string): "light" | "dark" {
  return projectName.includes("dark") ? "dark" : "light";
}

// ─── Button ──────────────────────────────────────────────────────────────────

test.describe("Button – visual snapshot", () => {
  test("matches snapshot in current theme", async ({ page }, testInfo) => {
    await goToFixtures(page);
    const section = page.getByTestId("fixture-button");
    await expect(section).toBeVisible();

    const theme = themeLabel(testInfo.project.name);
    await expect(section).toHaveScreenshot(`button-${theme}.png`);
  });
});

// ─── EventCard ───────────────────────────────────────────────────────────────

test.describe("EventCard – visual snapshot", () => {
  test("matches snapshot in current theme", async ({ page }, testInfo) => {
    await goToFixtures(page);
    const section = page.getByTestId("fixture-event-card");
    await expect(section).toBeVisible();

    const theme = themeLabel(testInfo.project.name);
    await expect(section).toHaveScreenshot(`event-card-${theme}.png`);
  });
});

// ─── RegistrationBox ─────────────────────────────────────────────────────────

test.describe("RegistrationBox – visual snapshot", () => {
  test("matches snapshot in current theme", async ({ page }, testInfo) => {
    await goToFixtures(page);
    const section = page.getByTestId("fixture-registration-box");
    await expect(section).toBeVisible();

    const theme = themeLabel(testInfo.project.name);
    await expect(section).toHaveScreenshot(`registration-box-${theme}.png`);
  });
});

// ─── TicketModal – purchase view ─────────────────────────────────────────────

test.describe("TicketModal (purchase) – visual snapshot", () => {
  test("matches snapshot in current theme", async ({ page }, testInfo) => {
    await goToFixtures(page);

    // Open the purchase modal.
    await page.getByTestId("open-ticket-modal").click();

    // Wait for the modal to be fully visible.
    const modal = page.getByRole("dialog");
    await expect(modal).toBeVisible();

    // Freeze animations before screenshot.
    await page.addStyleTag({
      content: `
        *, *::before, *::after {
          animation-duration: 0s !important;
          transition-duration: 0s !important;
        }
      `,
    });

    const theme = themeLabel(testInfo.project.name);
    await expect(modal).toHaveScreenshot(`ticket-modal-purchase-${theme}.png`);

    // Clean up: close the modal before the next test.
    await page.keyboard.press("Escape");
    await expect(modal).not.toBeVisible();
  });
});

// ─── TicketModal – sold-out / waitlist view ───────────────────────────────────

test.describe("TicketModal (waitlist) – visual snapshot", () => {
  test("matches snapshot in current theme", async ({ page }, testInfo) => {
    await goToFixtures(page);

    // Open the sold-out waitlist modal.
    await page.getByTestId("open-waitlist-modal").click();

    const modal = page.getByRole("dialog");
    await expect(modal).toBeVisible();

    await page.addStyleTag({
      content: `
        *, *::before, *::after {
          animation-duration: 0s !important;
          transition-duration: 0s !important;
        }
      `,
    });

    const theme = themeLabel(testInfo.project.name);
    await expect(modal).toHaveScreenshot(`ticket-modal-waitlist-${theme}.png`);

    await page.keyboard.press("Escape");
    await expect(modal).not.toBeVisible();
  });
});

// ─── CSS token regression guard ───────────────────────────────────────────────
// This test validates that the neubrutalist shadow tokens are applied.
// A change to the CSS custom property or Tailwind class will fail this assertion
// _before_ a screenshot comparison is even needed.

test.describe("Neubrutalist design tokens – runtime assertion", () => {
  test("Button has hard black drop-shadow", async ({ page }) => {
    await goToFixtures(page);

    const firstButton = page
      .getByTestId("fixture-button")
      .locator("button")
      .first();

    await expect(firstButton).toBeVisible();

    const boxShadow = await firstButton.evaluate(
      (el) => window.getComputedStyle(el).boxShadow,
    );

    // The neubrutalist shadow must have a hard black (0,0,0) component and no
    // blur radius (third value is 0px).
    expect(boxShadow).toMatch(/rgba\(0,\s*0,\s*0/);
    // Hard shadow: no blur (third parameter = 0px).
    expect(boxShadow).toMatch(/-?\d+px\s+-?\d+px\s+0px/);
  });

  test("EventCard has hard black box-shadow", async ({ page }) => {
    await goToFixtures(page);

    const card = page
      .getByTestId("fixture-event-card")
      .locator('[class*="shadow"]')
      .first();

    await expect(card).toBeVisible();

    const boxShadow = await card.evaluate(
      (el) => window.getComputedStyle(el).boxShadow,
    );

    expect(boxShadow).toMatch(/rgba\(0,\s*0,\s*0/);
  });

  test("EventCard has border-radius (rounded corners)", async ({ page }) => {
    await goToFixtures(page);

    const card = page
      .getByTestId("fixture-event-card")
      .locator("a")
      .first()
      .locator("> div");

    await expect(card).toBeVisible();

    const borderRadius = await card.evaluate(
      (el) => window.getComputedStyle(el).borderRadius,
    );

    // Must be non-zero (neubrutalist cards use rounded-xl ≈ 12px).
    const parsed = parseFloat(borderRadius);
    expect(parsed).toBeGreaterThan(0);
  });
});
