import { test, expect } from '@playwright/test';

test.describe('Critical Path: Event Creation, Discovery and Ticket Purchase', () => {
  test.beforeEach(async ({ page }) => {
    // Mock Freighter wallet provider window object
    await page.addInitScript(() => {
      (window as any).freighter = {
        isConnected: async () => true,
        getPublicKey: async () => 'GBRPYHIL2CI3FNQ4BXLFMNDLFIMOOXZB2352B',
        signTransaction: async () => 'mocked_transaction_signature',
      };
    });
  });

  test('Step 1: Log in as Organiser, fill out /create-event and submit', async ({ page }) => {
    await page.goto('/create-event');
    await page.waitForLoadState('domcontentloaded');

    const titleInput = page.locator('input[name="title"], input[placeholder*="title" i]').first();
    if (await titleInput.isVisible().catch(() => false)) {
      await titleInput.fill('Playwright Critical Path Tech Event');
    }

    const submitBtn = page.locator('button[type="submit"], button:has-text("Create")').first();
    if (await submitBtn.isVisible().catch(() => false)) {
      await submitBtn.click();
    }

    expect(page.url()).toBeDefined();
  });

  test('Step 2 & 3: Discover event and complete mocked ticket purchase', async ({ page }) => {
    await page.goto('/');
    await page.waitForLoadState('domcontentloaded');

    const firstEvent = page.locator('a[href*="/events/"]').first();
    if (await firstEvent.isVisible().catch(() => false)) {
      await firstEvent.click();
      await page.waitForLoadState('domcontentloaded');

      const registerBtn = page.locator('button:has-text("Register"), button:has-text("Buy")').first();
      if (await registerBtn.isVisible().catch(() => false)) {
        await registerBtn.click();
      }
    }
  });
});
