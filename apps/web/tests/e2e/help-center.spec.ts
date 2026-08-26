import { test, expect } from "@playwright/test";

/**
 * End-to-end coverage for the help center:
 *  - apps/web/app/help/page.tsx            (search + category grid)
 *  - apps/web/app/help/[category]/[slug]/page.tsx (article + related-articles sidebar)
 *  - apps/web/content/help/*.mdx           (markdown-backed articles)
 *
 * NOTE on navigation: there is currently no `app/help/[category]/page.tsx`
 * index route, so clicking a category card on `/help` navigates to a URL
 * that has no page and falls through to the global not-found page (this is
 * called out explicitly in a comment in `[category]/[slug]/page.tsx`). To
 * stay accurate to real behavior, the "into an article" hop below is done
 * via a real article URL and the in-article "Related Articles" sidebar
 * (the one working piece of category-scoped navigation today), rather than
 * via the not-yet-implemented category listing page.
 */

const ARTICLE_ONE = {
  path: "/help/stellar-web3/gas-transaction-fees",
  title: "Gas & Transaction Fees on Stellar",
};
const ARTICLE_TWO = {
  path: "/help/stellar-web3/how-to-set-up-stellar-wallet",
  title: "How to set up a Stellar Wallet (Freighter, Albedo)",
};

test.describe("Help center", () => {
  test("renders the search input and at least one category card", async ({
    page,
  }) => {
    await page.goto("/help");

    await expect(
      page.getByRole("textbox", { name: /search help articles/i })
    ).toBeVisible();

    // Category cards render as headings ("Getting Started", "Payments", ...)
    // inside linked cards — assert at least one is present.
    const categoryHeadings = page.getByRole("heading", { level: 3 });
    await expect(categoryHeadings.first()).toBeVisible();
    expect(await categoryHeadings.count()).toBeGreaterThan(0);

    // The card itself should be a real link into /help/<category-slug>.
    await expect(
      page.getByRole("link", { name: /Getting Started/i })
    ).toHaveAttribute("href", "/help/getting-started");
  });

  test("navigating into an article renders its markdown content and a non-default title", async ({
    page,
  }) => {
    await page.goto(ARTICLE_ONE.path);

    await expect(
      page.getByRole("heading", { level: 1, name: ARTICLE_ONE.title })
    ).toBeVisible();

    // Markdown body rendered via react-markdown — check a heading and a
    // bullet point from the source .mdx actually made it to the DOM.
    await expect(
      page.getByRole("heading", { name: "What is a Transaction Fee?" })
    ).toBeVisible();
    await expect(page.getByText(/Base Fee:/)).toBeVisible();

    // Article page sets a real per-article <title> via generateMetadata —
    // assert it's not the app's default title.
    await expect(page).toHaveTitle(`${ARTICLE_ONE.title} | Help Center - Agora`);

    // Click into a sibling article via the "Related Articles" sidebar — the
    // real, working category-scoped navigation in this app today.
    await page
      .getByRole("link", { name: ARTICLE_TWO.title })
      .first()
      .click();

    await expect(page).toHaveURL(new RegExp(`${ARTICLE_TWO.path}$`));
    await expect(
      page.getByRole("heading", { level: 1, name: ARTICLE_TWO.title })
    ).toBeVisible();
    await expect(page).toHaveTitle(`${ARTICLE_TWO.title} | Help Center - Agora`);
  });

  test("an unknown article slug renders the not-found page instead of a 500", async ({
    page,
  }) => {
    const response = await page.goto(
      "/help/getting-started/this-article-does-not-exist"
    );

    expect(response?.status()).toBe(404);
    await expect(page.getByRole("heading", { name: /404/i })).toBeVisible();
  });
});
