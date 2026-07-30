import type { TicketTierOption } from '@/types/checkout';

/**
 * Ticket tier catalogue keyed by event id.
 *
 * The event/pricing endpoints this would normally come from aren't wired up in
 * the mobile client yet, so this mirrors the mock-data convention already used
 * by `app/event/[id].tsx` and `app/ticket/[id].tsx` until a real
 * `/api/events/:id/tiers` call replaces it.
 *
 * Lives here rather than in the checkout screen so the event detail screen can
 * show the same availability the checkout screen will (issue #1010).
 */
const MOCK_TIERS_BY_EVENT: Record<string, TicketTierOption[]> = {
  '1': [
    { id: 'tier-ga', name: 'General Admission', priceUsdc: 150, remaining: 340 },
    {
      id: 'tier-vip',
      name: 'VIP',
      description: 'Front-row seating + backstage pass',
      priceUsdc: 450,
      remaining: 12,
    },
  ],
  '2': [
    { id: 'tier-ga', name: 'General Admission', priceUsdc: 80, remaining: 500 },
    { id: 'tier-pro', name: 'Pro Pass', description: 'Includes workshop access', priceUsdc: 220, remaining: 45 },
  ],
  '3': [
    { id: 'tier-ga', name: 'General Admission', priceUsdc: 60, remaining: 800 },
    { id: 'tier-early', name: 'Early Bird', priceUsdc: 45, remaining: 0 },
  ],
};

export const DEFAULT_TIERS: TicketTierOption[] = [
  { id: 'tier-ga', name: 'General Admission', priceUsdc: 100, remaining: 200 },
];

export function getTiersForEvent(eventId: string | undefined): TicketTierOption[] {
  if (!eventId) return DEFAULT_TIERS;
  return MOCK_TIERS_BY_EVENT[eventId] ?? DEFAULT_TIERS;
}

/**
 * Total tickets left across all tiers, or undefined when no tier reports a
 * count (so callers can distinguish "sold out" from "unknown").
 */
export function totalRemaining(tiers: TicketTierOption[]): number | undefined {
  const counts = tiers
    .map((tier) => tier.remaining)
    .filter((value): value is number => typeof value === 'number');

  if (counts.length === 0) return undefined;
  return counts.reduce((sum, value) => sum + value, 0);
}
