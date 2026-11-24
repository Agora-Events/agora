/**
 * Copy strings for the affiliate dashboard (`app/affiliates/dashboard/page.tsx`
 * and its `AffiliateMetricsDashboard` child) — extracted out of the JSX so the
 * marketing copy lives in one place instead of being hardcoded inline (#1579).
 */

// ── app/affiliates/dashboard/page.tsx ──────────────────────────────────────

export const DASHBOARD_EYEBROW = "Affiliate Insights";
export const DASHBOARD_TITLE = "Performance Metrics";
export const DASHBOARD_SUBTITLE =
  "Track your referral clicks, sales, and total commission earned.";

// ── components/analytics/affiliate-metrics-dashboard.tsx ───────────────────

export const DASHBOARD_METRIC_LABELS = {
  referralClicks: "Referral Clicks",
  ticketSales: "Ticket Sales",
  conversionRate: "Conversion Rate",
  commissionEarned: "Commission Earned",
} as const;

export const DASHBOARD_LOADING_ARIA_LABEL = "Loading metric";

export const DASHBOARD_ERROR_TITLE = "Affiliate metrics could not be loaded.";
export const DASHBOARD_ERROR_SUBTITLE = "Refresh the page to try again.";

export const DASHBOARD_EMPTY_TITLE = "No Affiliate Activity Yet";
export const DASHBOARD_EMPTY_SUBTITLE =
  "Share your referral links to start tracking clicks, sales, and earning commissions.";

export const DASHBOARD_FETCH_ERROR_MESSAGE = "Unable to load affiliate metrics";
