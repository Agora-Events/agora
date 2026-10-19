import { computeOrderTotals, DEFAULT_PLATFORM_FEE_BPS } from '../pricing';

describe('computeOrderTotals', () => {
  it('multiplies unit price by quantity for the subtotal', () => {
    const totals = computeOrderTotals({ unitPriceUsdc: 25, quantity: 3 });
    expect(totals.subtotalUsdc).toBe(75);
  });

  it('estimates the platform fee using the default bps without adding it to the total', () => {
    const totals = computeOrderTotals({ unitPriceUsdc: 10, quantity: 3 });
    // subtotal = 30, fee = 30 * 250 / 10000 = 0.75
    expect(totals.subtotalUsdc).toBe(30);
    expect(totals.estimatedPlatformFeeUsdc).toBe(0.75);
    // The contract deducts the fee from the organizer's share — it is never
    // charged on top of the buyer's transfer, so total === subtotal always.
    expect(totals.totalUsdc).toBe(totals.subtotalUsdc);
  });

  it('supports a custom fee bps (e.g. 0 for a Pro-subscribed organizer)', () => {
    const totals = computeOrderTotals({ unitPriceUsdc: 10, quantity: 2, platformFeeBps: 0 });
    expect(totals.estimatedPlatformFeeUsdc).toBe(0);
    expect(totals.totalUsdc).toBe(20);
  });

  it('handles free tickets', () => {
    const totals = computeOrderTotals({ unitPriceUsdc: 0, quantity: 5 });
    expect(totals.subtotalUsdc).toBe(0);
    expect(totals.estimatedPlatformFeeUsdc).toBe(0);
    expect(totals.totalUsdc).toBe(0);
  });

  it('rounds away floating-point noise to two decimal places', () => {
    // 10/3 * 3 === 9.999999999999998 in IEEE754 double arithmetic.
    const totals = computeOrderTotals({ unitPriceUsdc: 10 / 3, quantity: 3 });
    expect(totals.subtotalUsdc).toBe(10);
  });

  it('exposes the documented default fee rate', () => {
    expect(DEFAULT_PLATFORM_FEE_BPS).toBe(250);
  });
});
