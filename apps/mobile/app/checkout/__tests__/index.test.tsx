import React from 'react';
import { Alert } from 'react-native';
import { render, fireEvent } from '@testing-library/react-native';
import CheckoutScreen from '../index';
import { useTicketCheckout } from '@/hooks/useTicketCheckout';
import { useAuth } from '@/hooks/useAuth';

/**
 * Issue #1005 acceptance criteria covered here:
 *   "The user is shown a detailed summary screen with ticket count, unit
 *   price, platform fees (if applicable), and total USDC cost before
 *   finalizing."
 *
 * The Soroban / backend orchestration itself is mocked via `useTicketCheckout`
 * (already covered in `hooks/__tests__/useTicketCheckout.test.ts`) — this
 * suite is only responsible for the screen wiring: tier/quantity selection,
 * the summary it produces, and the guard rails before `startCheckout` fires.
 */

let mockParams: Record<string, string> = { eventId: '1', eventTitle: 'Stellar Meridian 2026' };
const mockPush = jest.fn();
const mockReplace = jest.fn();

jest.mock('expo-router', () => ({
  useLocalSearchParams: () => mockParams,
  useRouter: () => ({ push: mockPush, replace: mockReplace }),
}));

jest.mock('@/hooks/useAuth');
jest.mock('@/hooks/useTicketCheckout');

const mockUseAuth = useAuth as jest.MockedFunction<typeof useAuth>;
const mockUseTicketCheckout = useTicketCheckout as jest.MockedFunction<typeof useTicketCheckout>;

const startCheckout = jest.fn();
const reset = jest.fn();

function idleCheckoutState() {
  return {
    phase: 'idle' as const,
    steps: [],
    errorMessage: null,
    receipt: null,
    isSubmitting: false,
    startCheckout,
    reset,
  };
}

beforeEach(() => {
  jest.clearAllMocks();
  mockParams = { eventId: '1', eventTitle: 'Stellar Meridian 2026' };
  mockUseTicketCheckout.mockReturnValue(idleCheckoutState());
  mockUseAuth.mockReturnValue({
    token: 'mock-jwt-token-agora',
    user: { name: 'Agora User', email: 'user@example.com', walletAddress: 'GBUYERWALLETADDRESSTESTNETXXXXXXXXXXXXXXXXXXXXXXXXXXXX' },
    isAuthenticated: true,
    login: jest.fn(),
    logout: jest.fn(),
    updateWalletAddress: jest.fn(),
  } as any);
});

describe('CheckoutScreen', () => {
  it('defaults to the first available tier and shows a matching order summary', () => {
    const { getByText, getAllByText } = render(<CheckoutScreen />);

    // Event 1's mock catalogue: General Admission (150 USDC, first / default), VIP (450 USDC).
    // "General Admission" appears twice: once as the selected TierSelector
    // card, once as the OrderSummaryCard's tier name line.
    expect(getByText('Stellar Meridian 2026')).toBeTruthy();
    expect(getAllByText('General Admission').length).toBe(2);
    // quantity defaults to 1, so unit price === subtotal === total === "150.00 USDC"
    expect(getAllByText('150.00 USDC').length).toBeGreaterThanOrEqual(2);
  });

  it('updates the order summary when the quantity is incremented', () => {
    const { getByTestId, getAllByText } = render(<CheckoutScreen />);

    fireEvent.press(getByTestId('quantity-increment'));
    fireEvent.press(getByTestId('quantity-increment'));

    // quantity now 3 → subtotal/total = 450.00 USDC
    expect(getAllByText('450.00 USDC').length).toBeGreaterThanOrEqual(2);
  });

  it('switches tiers and resets quantity back to 1', () => {
    const { getByTestId, getAllByText } = render(<CheckoutScreen />);

    fireEvent.press(getByTestId('quantity-increment')); // quantity -> 2
    fireEvent.press(getByTestId('tier-card-tier-vip'));

    // "VIP" now appears twice: TierSelector card + OrderSummaryCard tier name.
    expect(getAllByText('VIP').length).toBe(2);
    expect(getByTestId('quantity-value').props.children).toBe(1); // reset on tier change
    expect(getAllByText('450.00 USDC').length).toBeGreaterThanOrEqual(2); // back to qty 1 at VIP price
  });

  it('warns and does not start checkout when no wallet is configured', () => {
    mockUseAuth.mockReturnValue({
      token: null,
      user: { name: 'Agora User', email: '', walletAddress: 'GDAGORA...' },
      isAuthenticated: true,
      login: jest.fn(),
      logout: jest.fn(),
      updateWalletAddress: jest.fn(),
    } as any);
    const alertSpy = jest.spyOn(Alert, 'alert').mockImplementation(() => {});

    const { getByTestId } = render(<CheckoutScreen />);
    fireEvent.press(getByTestId('checkout-confirm-button'));

    expect(alertSpy).toHaveBeenCalledWith('Wallet required', expect.any(String));
    expect(startCheckout).not.toHaveBeenCalled();
  });

  it('calls startCheckout with the selected tier, quantity, and buyer wallet on confirm', () => {
    const { getByTestId } = render(<CheckoutScreen />);
    fireEvent.press(getByTestId('quantity-increment')); // quantity -> 2

    fireEvent.press(getByTestId('checkout-confirm-button'));

    expect(startCheckout).toHaveBeenCalledWith(
      expect.objectContaining({
        eventId: '1',
        eventTitle: 'Stellar Meridian 2026',
        tierId: 'tier-ga',
        tierName: 'General Admission',
        unitPriceUsdc: 150,
        quantity: 2,
        buyerPublicKey: 'GBUYERWALLETADDRESSTESTNETXXXXXXXXXXXXXXXXXXXXXXXXXXXX',
      })
    );
  });

  it('disables the confirm button and hides its label behind a spinner while submitting', () => {
    mockUseTicketCheckout.mockReturnValue({ ...idleCheckoutState(), phase: 'in-progress', isSubmitting: true });

    const { getByTestId, queryByText } = render(<CheckoutScreen />);

    // Button.tsx swaps the title for an ActivityIndicator while `loading` is true.
    expect(queryByText('Confirm & Pay with USDC')).toBeNull();
    expect(getByTestId('checkout-confirm-button').props.accessibilityState?.disabled).toBe(true);
  });
});
