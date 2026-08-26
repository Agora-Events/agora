import { test, expect, type Page } from "@playwright/test";

/** Navigate to the fixture page, set fixed viewport, and disable animations for deterministic snapshots. */
async function goToFixtures(page: Page) {
  await page.setViewportSize({ width: 1280, height: 800 });
  await page.goto("/visual-fixtures", { timeout: 30_000 });
  await page.waitForSelector("h2", { state: "visible", timeout: 30_000 });
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

function themeLabel(projectName: string): "light" | "dark" {
  return projectName.includes("dark") ? "dark" : "light";
}

test.describe("EventCard stateful variants – visual snapshots", () => {
  const variants = [
    { name: "free-event", testId: "fixture-event-card-free" },
    { name: "paid-event", testId: "fixture-event-card-paid" },
    { name: "sold-out", testId: "fixture-event-card-sold-out" },
    { name: "followers-only", testId: "fixture-event-card-followers-only" },
    { name: "long-title", testId: "fixture-event-card-long-title" },
    { name: "missing-image", testId: "fixture-event-card-missing-image" },
  ];

  for (const variant of variants) {
    test(`matches snapshot for ${variant.name} variant`, async ({ page }, testInfo) => {
      await goToFixtures(page);
      const card = page.getByTestId(variant.testId);
      await expect(card).toBeVisible();

      const theme = themeLabel(testInfo.project.name);
      await expect(card).toHaveScreenshot(`event-card-${variant.name}-${theme}.png`);
    });
  }
});
