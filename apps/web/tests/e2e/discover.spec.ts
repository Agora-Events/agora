import { test, expect, type Page, type Locator } from "@playwright/test";

/**
 * End-to-end coverage for the discover/filter flow:
 *  - apps/web/app/discover/page.tsx
 *  - apps/web/components/events/category-section.tsx / category-chips.tsx
 *  - apps/web/components/events/popular-events-section.tsx
 *
 * The category chips and event cards are backed by live data from
 * `/api/events/discover` (derived from whatever events exist in the
 * database), so this suite intentionally avoids hardcoding category names
 * or event titles. Instead it discovers the real chips/cards rendered at
 * test time and asserts the *behavior* (narrowing, empty state, navigation)
 * against whichever ones actually produce that behavior.
 */

/** Event cards are `<Link href="/events/[id]">` wrapping the whole card; every card's accessible content includes the "View Event" affordance exactly once, regardless of viewport (mobile/desktop variants toggle via CSS, not by removing the element). */
function eventCards(page: Page): Locator {
  return page.getByRole("link").filter({ hasText: "View Event" });
}

/** The category chip bar rendered under the "Browse by Category" heading. Index 0 is always the "All" chip. */
function categoryChips(page: Page): Locator {
  return page
    .locator("section")
    .filter({ has: page.getByRole("heading", { name: "Browse by Category" }) })
    .getByRole("button");
}

test.describe("Discover page", () => {
  test("loads with an Explore events heading and at least one event card", async ({
    page,
  }) => {
    await page.goto("/discover");

    await expect(
      page.getByRole("heading", { name: "Explore events" })
    ).toBeVisible();

    const cards = eventCards(page);
    await expect(cards.first()).toBeVisible();
    expect(await cards.count()).toBeGreaterThan(0);
  });

  test("selecting a category chip narrows the visible event cards", async ({
    page,
  }) => {
    await page.goto("/discover");

    const cards = eventCards(page);
    await expect(cards.first()).toBeVisible();
    const initialCount = await cards.count();

    const chips = categoryChips(page);
    await expect(chips.first()).toBeVisible(); // wait out the chip-loading skeleton
    const chipCount = await chips.count();
    expect(chipCount).toBeGreaterThan(1); // "All" plus at least one real category

    // Try each specific category (skipping "All" at index 0) until one
    // narrows the result set — the real distinct categories present, and
    // which of them narrow vs. empty the list, depend on live event data.
    let narrowed = false;
    for (let i = 1; i < chipCount; i++) {
      await chips.nth(i).click();
      const count = await cards.count();
      if (count > 0 && count < initialCount) {
        narrowed = true;
        break;
      }
    }

    expect(narrowed).toBe(true);
  });

  test("a category with no matching events renders the empty state", async ({
    page,
  }) => {
    await page.goto("/discover");
    await expect(eventCards(page).first()).toBeVisible();

    const chips = categoryChips(page);
    await expect(chips.first()).toBeVisible(); // wait out the chip-loading skeleton
    const chipCount = await chips.count();

    let sawEmptyState = false;
    for (let i = 1; i < chipCount; i++) {
      await chips.nth(i).click();
      if ((await eventCards(page).count()) === 0) {
        sawEmptyState = true;
        await expect(page.getByText("No events found").first()).toBeVisible();
        break;
      }
    }

    expect(sawEmptyState).toBe(true);
  });

  test("clicking an event card navigates to its event page", async ({
    page,
  }) => {
    await page.goto("/discover");

    const card = eventCards(page).first();
    await expect(card).toBeVisible();
    await card.click();

    await expect(page).toHaveURL(/\/events\//);
  });
});
