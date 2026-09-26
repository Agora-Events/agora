import React from 'react';
import { render, fireEvent } from '@testing-library/react-native';
import CheckoutCompleteScreen from '../complete';
import type { CheckoutReceipt } from '@/types/checkout';

/**
 * Issue #1005 acceptance criterion: "Upon success, the UI navigates to a
 * completion screen displaying the purchase receipt."
 */

jest.mock('expo-clipboard', () => ({
  setStringAsync: jest.fn().mockResolvedValue(true),
}));

let mockParams: Record<string, string | undefined> = {};
const mockReplace = jest.fn();

jest.mock('expo-router', () => ({
  useLocalSearchParams: () => mockParams,
  useRouter: () => ({ replace: mockReplace }),
}));

const receipt: CheckoutReceipt = {
  ticketId: 'ticket-abc123',
  paymentId: 'pay-evt-1-abc123',
  eventId: 'evt-1',
  eventTitle: 'Stellar Meridian 2026',
  tierName: 'General Admission',
  quantity: 2,
  unitPriceUsdc: 25,
  platformFeeUsdc: 1.25,
  totalPaidUsdc: 50,
  approvalTxHash: 'approval-hash',
  paymentTxHash: 'payment-hash',
  buyerPublicKey: 'GA7QYNF7SOWQ3GLR2BGMZEHXAVIRZA4KVWLTJJFC7MGXUA74P7UJVSGZ',
  completedAt: '2026-07-28T12:00:00.000Z',
};

beforeEach(() => {
  jest.clearAllMocks();
  mockParams = { receipt: JSON.stringify(receipt) };
});

describe('CheckoutCompleteScreen', () => {
  it('renders the receipt details and a plural ticket count message', () => {
    const { getByText, getAllByText } = render(<CheckoutCompleteScreen />);

    expect(getByText("You're going!")).toBeTruthy();
    expect(getByText('Your 2 tickets have been secured on-chain.')).toBeTruthy();
    expect(getAllByText('Stellar Meridian 2026').length).toBeGreaterThanOrEqual(1);
    expect(getByText('ticket-abc123')).toBeTruthy();
  });

  it('uses singular phrasing for a single-ticket purchase', () => {
    mockParams = { receipt: JSON.stringify({ ...receipt, quantity: 1 }) };
    const { getByText, queryByText } = render(<CheckoutCompleteScreen />);

    expect(getByText('Your ticket has been secured on-chain.')).toBeTruthy();
    expect(queryByText(/tickets have been secured/)).toBeNull();
  });

  it('navigates to /ticket/[id] with the receipt ticket id when "View My Tickets" is pressed', () => {
    const { getByTestId } = render(<CheckoutCompleteScreen />);
    fireEvent.press(getByTestId('checkout-complete-view-tickets'));

    expect(mockReplace).toHaveBeenCalledWith({
      pathname: '/ticket/[id]',
      params: { id: 'ticket-abc123' },
    });
  });

  it('navigates back to Discover when "Back to Events" is pressed', () => {
    const { getByTestId } = render(<CheckoutCompleteScreen />);
    fireEvent.press(getByTestId('checkout-complete-back-to-events'));

    expect(mockReplace).toHaveBeenCalledWith('/(tabs)/discover');
  });

  it('shows a fallback state instead of crashing when the receipt param is missing', () => {
    mockParams = {};
    const { getByText, queryByText } = render(<CheckoutCompleteScreen />);

    expect(getByText("We couldn't find your receipt.")).toBeTruthy();
    expect(queryByText("You're going!")).toBeNull();
  });

  it('shows the fallback state when the receipt param is malformed JSON', () => {
    mockParams = { receipt: '{not valid json' };
    const { getByText } = render(<CheckoutCompleteScreen />);
    expect(getByText("We couldn't find your receipt.")).toBeTruthy();
  });

  it('lets the buyer navigate away from the fallback state too', () => {
    mockParams = {};
    const { getByTestId } = render(<CheckoutCompleteScreen />);
    fireEvent.press(getByTestId('checkout-complete-back-to-events'));
    expect(mockReplace).toHaveBeenCalledWith('/(tabs)/discover');
  });
});
