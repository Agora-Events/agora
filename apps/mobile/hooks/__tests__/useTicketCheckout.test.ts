import { act, renderHook, waitFor } from '@testing-library/react-native';
import { useTicketCheckout } from '../useTicketCheckout';
import { purchaseTickets } from '@/services/ticketPaymentContract';
import { recordTicketPurchase } from '@/services/paymentsApi';

/**
 * Issue #1005 — checkout state machine.
 *
 * `purchaseTickets` (Soroban build/sign/submit/poll) and `recordTicketPurchase`
 * (the `/api/payments/ticket` POST) are both mocked here: this test is only
 * responsible for the hook's own orchestration — step transitions, success/
 * error phases, and the receipt it hands back to the UI — not for Soroban RPC
 * behavior, which is covered in `services/__tests__/ticketPaymentContract.test.ts`.
 */

jest.mock('@/services/ticketPaymentContract', () => {
  const actual = jest.requireActual('@/services/ticketPaymentContract');
  return {
    ...actual,
    purchaseTickets: jest.fn(),
  };
});

jest.mock('@/services/paymentsApi', () => ({
  recordTicketPurchase: jest.fn(),
}));

jest.mock('expo-secure-store', () => ({
  setItemAsync: jest.fn().mockResolvedValue(undefined),
  getItemAsync: jest.fn().mockResolvedValue(null),
  deleteItemAsync: jest.fn().mockResolvedValue(undefined),
}));

const mockPurchaseTickets = purchaseTickets as jest.MockedFunction<typeof purchaseTickets>;
const mockRecordTicketPurchase = recordTicketPurchase as jest.MockedFunction<typeof recordTicketPurchase>;

const baseParams = {
  eventId: 'evt-1',
  eventTitle: 'Stellar Meridian 2026',
  tierId: 'tier-ga',
  tierName: 'General Admission',
  unitPriceUsdc: 25,
  quantity: 2,
  buyerPublicKey: 'GA7QYNF7SOWQ3GLR2BGMZEHXAVIRZA4KVWLTJJFC7MGXUA74P7UJVSGZ',
};

beforeEach(() => {
  jest.clearAllMocks();
});

describe('useTicketCheckout', () => {
  it('starts idle with every step pending', () => {
    const { result } = renderHook(() => useTicketCheckout());
    expect(result.current.phase).toBe('idle');
    expect(result.current.steps.every((step) => step.status === 'pending')).toBe(true);
    expect(result.current.receipt).toBeNull();
  });

  it('walks through progress stages, records the purchase, and reaches success with a receipt', async () => {
    mockPurchaseTickets.mockImplementation(async ({ onProgress }) => {
      onProgress?.({ stage: 'building-approval' });
      onProgress?.({ stage: 'signing-approval' });
      onProgress?.({ stage: 'submitting-approval' });
      onProgress?.({ stage: 'confirming-approval', attempt: 1, maxAttempts: 30 });
      onProgress?.({ stage: 'building-payment' });
      onProgress?.({ stage: 'signing-payment' });
      onProgress?.({ stage: 'submitting-payment' });
      onProgress?.({ stage: 'confirming-payment', attempt: 1, maxAttempts: 30 });
      return {
        paymentId: 'pay-evt-1-abc123',
        approvalTxHash: 'approval-hash-1',
        paymentTxHash: 'payment-hash-1',
        buyerPublicKey: baseParams.buyerPublicKey,
        totalPaidUsdc: 50,
        checkInSecret: new Uint8Array(32).fill(7),
      };
    });

    mockRecordTicketPurchase.mockResolvedValue({
      ticketId: 'ticket-999',
      paymentId: 'pay-evt-1-abc123',
      status: 'confirmed',
    });

    const { result } = renderHook(() => useTicketCheckout());

    await act(async () => {
      await result.current.startCheckout(baseParams);
    });

    await waitFor(() => expect(result.current.phase).toBe('success'));

    expect(result.current.receipt).toMatchObject({
      ticketId: 'ticket-999',
      paymentId: 'pay-evt-1-abc123',
      eventId: 'evt-1',
      tierName: 'General Admission',
      quantity: 2,
      unitPriceUsdc: 25,
      totalPaidUsdc: 50,
      approvalTxHash: 'approval-hash-1',
      paymentTxHash: 'payment-hash-1',
    });
    expect(result.current.steps.every((step) => step.status === 'done')).toBe(true);
    expect(mockRecordTicketPurchase).toHaveBeenCalledWith(
      expect.objectContaining({
        eventId: 'evt-1',
        tierId: 'tier-ga',
        quantity: 2,
        paymentId: 'pay-evt-1-abc123',
        approvalTxHash: 'approval-hash-1',
        paymentTxHash: 'payment-hash-1',
      })
    );
  });

  it('still reaches success with a receipt if only the backend recording step fails', async () => {
    mockPurchaseTickets.mockResolvedValue({
      paymentId: 'pay-evt-1-def456',
      approvalTxHash: 'approval-hash-2',
      paymentTxHash: 'payment-hash-2',
      buyerPublicKey: baseParams.buyerPublicKey,
      totalPaidUsdc: 50,
      checkInSecret: new Uint8Array(32).fill(1),
    });
    mockRecordTicketPurchase.mockRejectedValue(new Error('network blip'));

    const { result } = renderHook(() => useTicketCheckout());

    await act(async () => {
      await result.current.startCheckout(baseParams);
    });

    await waitFor(() => expect(result.current.phase).toBe('success'));
    // The payment already confirmed on-chain — a Postgres write failure must
    // not be reported to the buyer as a failed purchase.
    expect(result.current.receipt?.paymentId).toBe('pay-evt-1-def456');
    expect(result.current.receipt?.ticketId).toBe('pay-evt-1-def456'); // falls back to paymentId
  });

  it('moves to the error phase with a readable message when the chain call fails', async () => {
    mockPurchaseTickets.mockRejectedValue(new Error('The contract rejected the transaction (error code 12).'));

    const { result } = renderHook(() => useTicketCheckout());

    await act(async () => {
      await result.current.startCheckout(baseParams);
    });

    await waitFor(() => expect(result.current.phase).toBe('error'));
    expect(result.current.errorMessage).toMatch(/error code 12/);
    expect(result.current.steps.some((step) => step.status === 'error')).toBe(true);
    expect(mockRecordTicketPurchase).not.toHaveBeenCalled();
  });

  it('ignores a second startCheckout call while one is already in flight', async () => {
    let resolvePurchase: (value: any) => void = () => {};
    mockPurchaseTickets.mockImplementation(
      () =>
        new Promise((resolve) => {
          resolvePurchase = resolve;
        })
    );

    const { result } = renderHook(() => useTicketCheckout());

    let firstCall: Promise<void>;
    await act(async () => {
      firstCall = result.current.startCheckout(baseParams);
      await result.current.startCheckout(baseParams); // should be a no-op
    });

    expect(mockPurchaseTickets).toHaveBeenCalledTimes(1);

    await act(async () => {
      resolvePurchase({
        paymentId: 'pay-evt-1-ghi789',
        approvalTxHash: 'a',
        paymentTxHash: 'b',
        buyerPublicKey: baseParams.buyerPublicKey,
        totalPaidUsdc: 50,
        checkInSecret: new Uint8Array(32),
      });
      mockRecordTicketPurchase.mockResolvedValue({
        ticketId: 't',
        paymentId: 'pay-evt-1-ghi789',
        status: 'confirmed',
      });
      await firstCall!;
    });

    await waitFor(() => expect(result.current.phase).toBe('success'));
  });

  it('reset() returns the hook to idle with cleared steps and receipt', async () => {
    mockPurchaseTickets.mockRejectedValue(new Error('boom'));
    const { result } = renderHook(() => useTicketCheckout());

    await act(async () => {
      await result.current.startCheckout(baseParams);
    });
    await waitFor(() => expect(result.current.phase).toBe('error'));

    act(() => {
      result.current.reset();
    });

    expect(result.current.phase).toBe('idle');
    expect(result.current.errorMessage).toBeNull();
    expect(result.current.steps.every((step) => step.status === 'pending')).toBe(true);
  });
});
