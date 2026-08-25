import { defineConfig, devices } from "@playwright/test";

/**
 * Playwright configuration for visual regression testing.
 *
 * Runs against a locally-served Next.js app on port 3000.
 * Base snapshots live in tests/visual/__snapshots__ and are committed to git.
 *
 * Update snapshots:
 *   npx playwright test --update-snapshots
 */
export default defineConfig({
  testDir: "./tests/visual",
  // Run snapshot tests sequentially to avoid race conditions on screenshots.
  workers: 1,
  fullyParallel: false,
  retries: 0,
  reporter: [["html", { outputFolder: "playwright-report", open: "never" }]],

  use: {
    baseURL: "http://localhost:3000",
    // Give components time to finish painting before taking a screenshot.
    actionTimeout: 10_000,
    screenshot: "only-on-failure",
    trace: "off",
  },

  // Snapshot comparison thresholds – tight enough to catch token drift.
  expect: {
    toHaveScreenshot: {
      // Allow up to 0.2% pixel diff (accounts for sub-pixel anti-aliasing).
      maxDiffPixelRatio: 0.002,
      // Animate=disabled removes CSS transitions before capturing.
      animations: "disabled",
    },
  },

  projects: [
    {
      name: "chromium-light",
      use: {
        ...devices["Desktop Chrome"],
        colorScheme: "light",
      },
    },
    {
      name: "chromium-dark",
      use: {
        ...devices["Desktop Chrome"],
        colorScheme: "dark",
      },
    },
  ],

  // Start the Next.js dev server before the test suite.
  webServer: {
    command: "pnpm dev",
    url: "http://localhost:3000",
    reuseExistingServer: !process.env.CI,
    timeout: 120_000,
  },
});
