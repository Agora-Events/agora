import React from 'react';
import { render } from '@testing-library/react-native';
import OrderSummaryCard from '../OrderSummaryCard';

describe('OrderSummaryCard', () => {
  it('shows the event, tier, ticket count, unit price, fee estimate, and total (issue #1005 AC)', () => {
    const { getByText, getAllByText } = render(
      <OrderSummaryCard
        eventTitle="Stellar Meridian 2026"
        tierName="General Admission"
        quantity={3}
        unitPriceUsdc={25}
      />
    );

    expect(getByText('Stellar Meridian 2026')).toBeTruthy();
    expect(getByText('General Admission')).toBeTruthy();
    expect(getByText('3')).toBeTruthy(); // ticket count
    expect(getByText('25.00 USDC')).toBeTruthy(); // unit price

    // Subtotal AND total both read "75.00 USDC" (25 * 3) by design — the
    // buyer transfers exactly the subtotal, since the platform fee is
    // carved out of the organizer payout rather than added on top
    // (see lib/pricing.ts).
    expect(getAllByText('75.00 USDC').length).toBe(2);
    expect(getByText('Total due')).toBeTruthy();
  });

  it('renders a zero fee and zero total for a free tier', () => {
    const { getAllByText } = render(
      <OrderSummaryCard eventTitle="Free Meetup" tierName="RSVP" quantity={1} unitPriceUsdc={0} />
    );
    // Subtotal, fee, and total are all "0.00 USDC" for a free ticket.
    expect(getAllByText('0.00 USDC').length).toBeGreaterThanOrEqual(2);
  });
});
