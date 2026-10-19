import { recordTicketPurchase, RecordPaymentError } from '../paymentsApi';
import { useAuthStore } from '@/hooks/useAuth';

/**
 * Issue #1005 — after a Soroban purchase confirms on-chain, the checkout hook
 * POSTs the transaction hash to `/api/payments/ticket` so the backend can
 * persist the ticket. These tests cover the request shape, auth header,
 * retry-on-5xx behavior, and error surfacing — all pure `fetch` mocking, no
 * real network access.
 */

const basePayload = {
  eventId: 'evt-1',
  tierId: 'tier-ga',
  quantity: 2,
  buyerWallet: 'GA7QYNF7SOWQ3GLR2BGMZEHXAVIRZA4KVWLTJJFC7MGXUA74P7UJVSGZ',
  unitPriceUsdc: 25,
  totalPaidUsdc: 50,
  paymentId: 'pay-evt-1-abc123',
  approvalTxHash: 'approval-hash',
  paymentTxHash: 'payment-hash',
};

function jsonResponse(status: number, body: unknown) {
  return {
    ok: status >= 200 && status < 300,
    status,
    json: async () => body,
  };
}

beforeEach(() => {
  jest.restoreAllMocks();
  global.fetch = jest.fn();
  useAuthStore.setState({ token: 'mock-jwt-token-agora', user: null, isAuthenticated: false });
});

describe('recordTicketPurchase', () => {
  it('POSTs to /api/payments/ticket with the payload and a bearer auth header', async () => {
    (global.fetch as jest.Mock).mockResolvedValueOnce(
      jsonResponse(200, { ticketId: 'ticket-1', paymentId: basePayload.paymentId, status: 'confirmed' })
    );

    const result = await recordTicketPurchase(basePayload);

    expect(global.fetch).toHaveBeenCalledTimes(1);
    const [url, init] = (global.fetch as jest.Mock).mock.calls[0];
    expect(url).toMatch(/\/api\/payments\/ticket$/);
    expect(init.method).toBe('POST');
    expect(init.headers.Authorization).toBe('Bearer mock-jwt-token-agora');
    expect(JSON.parse(init.body)).toMatchObject({ eventId: 'evt-1', quantity: 2 });
    expect(result).toEqual({ ticketId: 'ticket-1', paymentId: basePayload.paymentId, status: 'confirmed' });
  });

  it('omits the Authorization header when there is no auth token', async () => {
    useAuthStore.setState({ token: null, user: null, isAuthenticated: false });
    (global.fetch as jest.Mock).mockResolvedValueOnce(jsonResponse(200, { ticketId: 't' }));

    await recordTicketPurchase(basePayload);

    const [, init] = (global.fetch as jest.Mock).mock.calls[0];
    expect(init.headers.Authorization).toBeUndefined();
  });

  it('falls back to the local paymentId/ticketId when the backend omits them', async () => {
    (global.fetch as jest.Mock).mockResolvedValueOnce(jsonResponse(200, {}));
    const result = await recordTicketPurchase(basePayload);
    expect(result.ticketId).toBe(basePayload.paymentId);
    expect(result.paymentId).toBe(basePayload.paymentId);
    expect(result.status).toBe('confirmed');
  });

  it('retries once on a 500 and succeeds on the second attempt', async () => {
    (global.fetch as jest.Mock)
      .mockResolvedValueOnce(jsonResponse(500, { error: 'db unavailable' }))
      .mockResolvedValueOnce(jsonResponse(200, { ticketId: 'ticket-2' }));

    const result = await recordTicketPurchase(basePayload);

    expect(global.fetch).toHaveBeenCalledTimes(2);
    expect(result.ticketId).toBe('ticket-2');
  });

  it('does not retry on a 4xx — surfaces the error immediately', async () => {
    (global.fetch as jest.Mock).mockResolvedValueOnce(jsonResponse(400, { error: 'Invalid quantity' }));

    await expect(recordTicketPurchase(basePayload)).rejects.toThrow('Invalid quantity');
    expect(global.fetch).toHaveBeenCalledTimes(1);
  });

  it('gives up after exhausting retries on repeated 5xx responses', async () => {
    (global.fetch as jest.Mock).mockResolvedValue(jsonResponse(503, { error: 'still down' }));

    await expect(recordTicketPurchase(basePayload)).rejects.toBeInstanceOf(RecordPaymentError);
    // initial attempt + RECORD_PAYMENT_MAX_RETRIES(2) retries = 3 calls total
    expect(global.fetch).toHaveBeenCalledTimes(3);
  }, 15000);
});
