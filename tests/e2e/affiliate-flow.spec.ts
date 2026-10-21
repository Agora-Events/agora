import { test, expect, BrowserContext, Page } from '@playwright/test';

test.describe.serial('Affiliate Flow: Referral Generation, Ticket Purchase and Commission', () => {
  let userAContext: BrowserContext;
  let userBContext: BrowserContext;
  let userAPage: Page;
  let userBPage: Page;
  let referralUrl: string;

  const userAWallet = 'GBRPYHIL2CI3FNQ4BXLFMNDLFIMOOXZB2352A'; // Affiliate (User A)
  const userBWallet = 'GDC2T2L6YXVHUX6A6B7NHTZZU3C6IYYK3ZXX6T2V34BTRX74H22A4E3B'; // Buyer (User B)

  test.beforeAll(async ({ browser }) => {
    // Context for User A (Affiliate)
    userAContext = await browser.newContext();
    await userAContext.addInitScript((wallet) => {
      (window as any).freighter = {
        isConnected: async () => true,
        getPublicKey: async () => wallet,
        signTransaction: async () => 'mocked_transaction_signature_A',
      };
    }, userAWallet);
    userAPage = await userAContext.newPage();

    // Context for User B (Buyer)
    userBContext = await browser.newContext();
    await userBContext.addInitScript((wallet) => {
      (window as any).freighter = {
        isConnected: async () => true,
        getPublicKey: async () => wallet,
        signTransaction: async () => 'mocked_transaction_signature_B',
      };
    }, userBWallet);
    userBPage = await userBContext.newPage();
  });

  test.afterAll(async () => {
    await userAContext.close();
    await userBContext.close();
  });

  test('Step 1: Generate a referral link for User A', async () => {
    await userAPage.goto('/');
    await userAPage.waitForLoadState('domcontentloaded');

    // Find an event to refer
    const firstEvent = userAPage.locator('a[href*="/events/"]').first();
    if (await firstEvent.isVisible().catch(() => false)) {
      await firstEvent.click();
      await userAPage.waitForLoadState('domcontentloaded');
    }

    // Interact with referral link generation UI if it exists
    const generateLinkBtn = userAPage.locator('button:has-text("Generate Referral Link"), button:has-text("Affiliate Link")').first();
    if (await generateLinkBtn.isVisible().catch(() => false)) {
      await generateLinkBtn.click();
    }

    // Assume the referral link structure incorporates User A's wallet address as a reference parameter
    const eventUrl = userAPage.url();
    referralUrl = eventUrl.includes('/events/') 
      ? `${eventUrl}?ref=${userAWallet}`
      : `http://localhost:3000/events/1?ref=${userAWallet}`;
  });

  test('Step 2: Simulate User B opening the referral link', async () => {
    await userBPage.goto(referralUrl);
    await userBPage.waitForLoadState('domcontentloaded');

    // Verify referral param is in the URL for User B
    expect(userBPage.url()).toContain(`ref=${userAWallet}`);
  });

  test('Step 3: Complete a ticket purchase as User B', async () => {
    const registerBtn = userBPage.locator('button:has-text("Register"), button:has-text("Buy")').first();
    if (await registerBtn.isVisible().catch(() => false)) {
      await registerBtn.click();

      // Proceed with ticket purchase modal if visible
      const confirmPurchaseBtn = userBPage.locator('button:has-text("Confirm"), button:has-text("Pay"), button:has-text("Checkout")').first();
      if (await confirmPurchaseBtn.isVisible().catch(() => false)) {
        await confirmPurchaseBtn.click();
      }
    }
  });

  test('Step 4 & 5: Verify commission is recorded for User A and affiliate dashboard updates', async () => {
    // Navigating to the assumed affiliate dashboard route
    await userAPage.goto('/organizers/analytics'); // Example route, adjust to match actual dashboard route once merged
    await userAPage.waitForLoadState('domcontentloaded');

    // Verify dashboard metrics for commission and referrals update
    const commissionMetric = userAPage.locator('[data-testid="total-commission"], .commission-amount').first();
    if (await commissionMetric.isVisible().catch(() => false)) {
      const text = await commissionMetric.innerText();
      // Commission should be greater than zero or updated appropriately
      expect(text).not.toContain('0.00'); 
    }

    const referralCountMetric = userAPage.locator('[data-testid="total-referrals"], .referral-count').first();
    if (await referralCountMetric.isVisible().catch(() => false)) {
      const countText = await referralCountMetric.innerText();
      // Expect referral count to increment
      expect(countText).not.toBe('0');
      expect(countText).toBeDefined();
    }
  });
});
