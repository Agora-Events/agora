import { useAuthStore } from '@/hooks/useAuth';
import {
  MarketplaceApiError,
  toKeyEnvelope,
  createListing,
  publishListing,
  fetchListings,
  fetchListing,
  withdrawListing,
  submitOffer,
  fetchOffers,
  uploadKeyEnvelope,
  fetchKeyEnvelope,
  registerPushToken,
} from '../marketplaceApi';

/**
 * Issue #1179 — marketplace API tests.
 *
 * Tests cover:
 *   - Request endpoints and body shape
 *   - Auth header inclusion when token exists
 *   - MarketplaceApiError with status and isNotFound
 *   - toKeyEnvelope pure function
 */

// ── Fixtures ──────────────────────────────────────────────────────────────────

const basePaymentId = 'pay-evt-1';
const baseEventId = 'evt-1';
const baseSellerWallet = 'GA7QYNF7SOWQ3GLR2BGMZEHXAVIRZA4KVWLTJJFC7MGXUA74P7UJVSGZ';
const baseBuyerWallet = 'GB7QYNF7SOWQ3GLR2BGMZEHXAVIRZA4KVWLTJJFC7MGXUA74P7UJVSGY';

function jsonResponse(status: number, body: unknown): Response {
  return {
    ok: status >= 200 && status < 300,
    status,
    json: async () => body,
    headers: new Headers({ 'Content-Type': 'application/json' }),
  } as Response;
}

// ── Setup ─────────────────────────────────────────────────────────────────────

beforeEach(() => {
  jest.restoreAllMocks();
  global.fetch = jest.fn();
  useAuthStore.setState({ token: null, user: null, isAuthenticated: false });
});

describe('MarketplaceApiError', () => {
  it('constructs with message and optional status', () => {
    const error = new MarketplaceApiError('Not found', 404);
    expect(error.message).toBe('Not found');
    expect(error.status).toBe(404);
    expect(error.name).toBe('MarketplaceApiError');
  });

  it('has isNotFound true when status is 404', () => {
    const error = new MarketplaceApiError('Not found', 404);
    expect(error.isNotFound).toBe(true);
  });

  it('has isNotFound false when status is not 404', () => {
    const error = new MarketplaceApiError('Bad request', 400);
    expect(error.isNotFound).toBe(false);
  });
});

describe('publishListing', () => {
  it('POSTs to /listings with correct payload and auth header', async () => {
    const mockToken = 'mock-jwt-token';
    useAuthStore.setState({ token: mockToken, user: null, isAuthenticated: false });

    const mockResponse = {
      payment_id: basePaymentId,
      event_id: baseEventId,
      seller_wallet: baseSellerWallet,
      price_stroops: 250000000,
      max_price_stroops: 300000000,
      royalty_bps: 250,
      status: 'active',
      buyer_wallet: null,
      listing_tx_hash: 'tx-abc123',
      sale_tx_hash: null,
      sold_at: null,
      created_at: '2026-09-25T10:00:00Z',
      updated_at: '2026-09-25T10:00:00Z',
      royalty_stroops: 62500000,
      seller_proceeds_stroops: 187500000,
      headroom_stroops: 50000000,
    };

    (global.fetch as jest.Mock).mockResolvedValueOnce(jsonResponse(200, mockResponse));

    const result = await publishListing({
      paymentId: basePaymentId,
      eventId: baseEventId,
      priceStroops: 250000000n,
      maxPriceStroops: 300000000n,
      royaltyBps: 250,
      listingTxHash: 'tx-abc123',
    });

    expect(global.fetch).toHaveBeenCalledTimes(1);
    const [url, init] = (global.fetch as jest.Mock).mock.calls[0];
    expect(url).toBe('http://localhost:8080/api/v1/marketplace/listings');
    expect(init.method).toBe('POST');
    expect(init.headers.Authorization).toBe(`Bearer ${mockToken}`);
    expect(JSON.parse(init.body)).toMatchObject({
      payment_id: basePaymentId,
      event_id: baseEventId,
      price_stroops: 250000000,
      max_price_stroops: 300000000,
      royalty_bps: 250,
      listing_tx_hash: 'tx-abc123',
    });
    expect(result).toEqual(mockResponse);
  });

  it('omits Authorization header when no auth token exists', async () => {
    const mockResponse = {
      payment_id: basePaymentId,
      event_id: baseEventId,
      price_stroops: 250000000,
    };
    (global.fetch as jest.Mock).mockResolvedValueOnce(jsonResponse(200, mockResponse));

    await publishListing({
      paymentId: basePaymentId,
      eventId: baseEventId,
      priceStroops: 250000000n,
      maxPriceStroops: 300000000n,
      royaltyBps: 250,
    });

    const [, init] = (global.fetch as jest.Mock).mock.calls[0];
    expect(init.headers.Authorization).toBeUndefined();
  });

  it('throws MarketplaceApiError on non-2xx response with status', async () => {
    useAuthStore.setState({ token: 'token', user: null, isAuthenticated: false });
    (global.fetch as jest.Mock).mockResolvedValueOnce(jsonResponse(400, { message: 'Invalid price' }));

    await expect(
      publishListing({
        paymentId: basePaymentId,
        eventId: baseEventId,
        priceStroops: 250000000n,
        maxPriceStroops: 300000000n,
        royaltyBps: 250,
      })
    ).rejects.toThrow('Invalid price');
  });

  it('throws MarketplaceApiError with status 404 for not found', async () => {
    useAuthStore.setState({ token: 'token', user: null, isAuthenticated: false });
    (global.fetch as jest.Mock).mockResolvedValueOnce(jsonResponse(404, { message: 'Not found' }));

    const error = await expect(
      publishListing({
        paymentId: basePaymentId,
        eventId: baseEventId,
        priceStroops: 250000000n,
        maxPriceStroops: 300000000n,
        royaltyBps: 250,
      })
    ).rejects;

    expect(error).toBeInstanceOf(MarketplaceApiError);
    expect((error as MarketplaceApiError).status).toBe(404);
  });
});

describe('fetchListings', () => {
  it('GETs from /listings with optional filters as query params', async () => {
    const mockToken = 'mock-jwt-token';
    useAuthStore.setState({ token: mockToken, user: null, isAuthenticated: false });

    const mockResponse = [
      {
        payment_id: 'pay-1',
        event_id: baseEventId,
        seller_wallet: baseSellerWallet,
        price_stroops: 250000000,
        status: 'active',
      },
      {
        payment_id: 'pay-2',
        event_id: baseEventId,
        seller_wallet: baseSellerWallet,
        price_stroops: 275000000,
        status: 'active',
      },
    ];

    (global.fetch as jest.Mock).mockResolvedValueOnce(jsonResponse(200, mockResponse));

    const result = await fetchListings({
      eventId: baseEventId,
      sellerWallet: baseSellerWallet,
      status: 'active' as const,
      limit: 10,
      offset: 0,
    });

    expect(global.fetch).toHaveBeenCalledTimes(1);
    const [url, init] = (global.fetch as jest.Mock).mock.calls[0];
    expect(url).toBe('http://localhost:8080/api/v1/marketplace/listings?event_id=evt-1&seller_wallet=GA7QYNF7SOWQ3GLR2BGMZEHXAVIRZA4KVWLTJJFC7MGXUA74P7UJVSGZ&status=active&limit=10&offset=0');
    expect(init.method).toBe('GET');
    expect(init.headers.Authorization).toBe(`Bearer ${mockToken}`);
    expect(result).toEqual(mockResponse);
  });

  it('uses default values when filter object is empty', async () => {
    useAuthStore.setState({ token: null, user: null, isAuthenticated: false });
    (global.fetch as jest.Mock).mockResolvedValueOnce(jsonResponse(200, []));

    await fetchListings({});

    const [url] = (global.fetch as jest.Mock).mock.calls[0];
    expect(url).toBe('http://localhost:8080/api/v1/marketplace/listings');
  });

  it('throws MarketplaceApiError on error response', async () => {
    useAuthStore.setState({ token: 'token', user: null, isAuthenticated: false });
    (global.fetch as jest.Mock).mockResolvedValueOnce(jsonResponse(500, { message: 'Server error' }));

    await expect(fetchListings({})).rejects.toThrow('Server error');
  });
});

describe('fetchListing', () => {
  it('GETs from /listings/:paymentId with URL encoding', async () => {
    useAuthStore.setState({ token: null, user: null, isAuthenticated: false });

    const mockResponse = {
      payment_id: 'pay-evt%201',
      event_id: baseEventId,
      price_stroops: 250000000,
      status: 'active',
    };
    (global.fetch as jest.Mock).mockResolvedValueOnce(jsonResponse(200, mockResponse));

    await fetchListing('pay-evt 1');

    const [url] = (global.fetch as jest.Mock).mock.calls[0];
    expect(url).toBe('http://localhost:8080/api/v1/marketplace/listings/pay-evt%201');
  });

  it('throws MarketplaceApiError when listing not found', async () => {
    useAuthStore.setState({ token: null, user: null, isAuthenticated: false });
    (global.fetch as jest.Mock).mockResolvedValueOnce(jsonResponse(404, { message: 'Listing not found' }));

    await expect(fetchListing(basePaymentId)).rejects.toThrow('Listing not found');
  });
});

describe('withdrawListing', () => {
  it('DELETEs from /listings/:paymentId with auth header', async () => {
    const mockToken = 'mock-jwt-token';
    useAuthStore.setState({ token: mockToken, user: null, isAuthenticated: false });

    const mockResponse = 'Listing withdrawn';
    (global.fetch as jest.Mock).mockResolvedValueOnce(jsonResponse(200, mockResponse));

    const result = await withdrawListing(basePaymentId);

    expect(global.fetch).toHaveBeenCalledTimes(1);
    const [url, init] = (global.fetch as jest.Mock).mock.calls[0];
    expect(url).toBe(`http://localhost:8080/api/v1/marketplace/listings/${basePaymentId}`);
    expect(init.method).toBe('DELETE');
    expect(init.headers.Authorization).toBe(`Bearer ${mockToken}`);
    expect(result).toBe(mockResponse);
  });

  it('throws MarketplaceApiError on withdrawal failure', async () => {
    useAuthStore.setState({ token: 'token', user: null, isAuthenticated: false });
    (global.fetch as jest.Mock).mockResolvedValueOnce(jsonResponse(403, { message: 'Not authorized' }));

    await expect(withdrawListing(basePaymentId)).rejects.toThrow('Not authorized');
  });
});

describe('submitOffer', () => {
  it('POSTs to /listings/:paymentId/offers with buyer public key and offer price', async () => {
    const mockToken = 'mock-jwt-token';
    useAuthStore.setState({ token: mockToken, user: null, isAuthenticated: false });

    const mockResponse = {
      id: 'offer-1',
      payment_id: basePaymentId,
      buyer_wallet: baseBuyerWallet,
      buyer_public_key: 'pub-base64-key',
      offer_price_stroops: 240000000,
      status: 'pending',
      created_at: '2026-09-25T11:00:00Z',
    };

    (global.fetch as jest.Mock).mockResolvedValueOnce(jsonResponse(200, mockResponse));

    const result = await submitOffer(
      basePaymentId,
      'pub-base64-key',
      240000000n
    );

    expect(global.fetch).toHaveBeenCalledTimes(1);
    const [url, init] = (global.fetch as jest.Mock).mock.calls[0];
    expect(url).toBe(`http://localhost:8080/api/v1/marketplace/listings/${basePaymentId}/offers`);
    expect(init.method).toBe('POST');
    expect(JSON.parse(init.body)).toMatchObject({
      buyer_public_key: 'pub-base64-key',
      offer_price_stroops: 240000000,
    });
    expect(result).toEqual(mockResponse);
  });

  it('throws MarketplaceApiError when offer submission fails', async () => {
    useAuthStore.setState({ token: 'token', user: null, isAuthenticated: false });
    (global.fetch as jest.Mock).mockResolvedValueOnce(jsonResponse(400, { message: 'Invalid price' }));

    await expect(submitOffer(basePaymentId, 'pub-key', 240000000n)).rejects.toThrow('Invalid price');
  });
});

describe('fetchOffers', () => {
  it('GETs from /listings/:paymentId/offers', async () => {
    useAuthStore.setState({ token: null, user: null, isAuthenticated: false });

    const mockResponse = [
      {
        id: 'offer-1',
        payment_id: basePaymentId,
        buyer_wallet: baseBuyerWallet,
        buyer_public_key: 'pub-base64-key',
        offer_price_stroops: 240000000,
        status: 'pending',
        created_at: '2026-09-25T11:00:00Z',
      },
    ];
    (global.fetch as jest.Mock).mockResolvedValueOnce(jsonResponse(200, mockResponse));

    const result = await fetchOffers(basePaymentId);

    expect(global.fetch).toHaveBeenCalledTimes(1);
    const [url] = (global.fetch as jest.Mock).mock.calls[0];
    expect(url).toBe(`http://localhost:8080/api/v1/marketplace/listings/${basePaymentId}/offers`);
    expect(result).toEqual(mockResponse);
  });
});

describe('uploadKeyEnvelope', () => {
  it('POSTs to /listings/:paymentId/key-envelope with envelope data', async () => {
    const mockToken = 'mock-jwt-token';
    useAuthStore.setState({ token: mockToken, user: null, isAuthenticated: false });

    const mockResponse = {
      payment_id: basePaymentId,
      buyer_wallet: baseBuyerWallet,
      ephemeral_public_key: 'eph-pub-base64',
      nonce: 'nonce-base64',
      ciphertext: 'cipher-base64',
      claimed_at: null,
      created_at: '2026-09-25T12:00:00Z',
    };

    const envelope = {
      ephemeralPublicKey: 'eph-pub-base64',
      nonce: 'nonce-base64',
      ciphertext: 'cipher-base64',
    };

    (global.fetch as jest.Mock).mockResolvedValueOnce(jsonResponse(200, mockResponse));

    const result = await uploadKeyEnvelope(basePaymentId, baseBuyerWallet, envelope, 'sale-tx-123');

    expect(global.fetch).toHaveBeenCalledTimes(1);
    const [url, init] = (global.fetch as jest.Mock).mock.calls[0];
    expect(url).toBe(`http://localhost:8080/api/v1/marketplace/listings/${basePaymentId}/key-envelope`);
    expect(init.method).toBe('POST');
    expect(JSON.parse(init.body)).toMatchObject({
      buyer_wallet: baseBuyerWallet,
      ephemeral_public_key: 'eph-pub-base64',
      nonce: 'nonce-base64',
      ciphertext: 'cipher-base64',
      sale_tx_hash: 'sale-tx-123',
    });
    expect(result).toEqual(mockResponse);
  });

  it('passes null for sale_tx_hash when not provided', async () => {
    useAuthStore.setState({ token: 'token', user: null, isAuthenticated: false });

    const envelope = {
      ephemeralPublicKey: 'eph-pub',
      nonce: 'nonce',
      ciphertext: 'cipher',
    };

    const mockResponse = {
      payment_id: basePaymentId,
      buyer_wallet: baseBuyerWallet,
      ephemeral_public_key: 'eph-pub',
      nonce: 'nonce',
      ciphertext: 'cipher',
      claimed_at: null,
      created_at: '2026-09-25T12:00:00Z',
    };

    (global.fetch as jest.Mock).mockResolvedValueOnce(jsonResponse(200, mockResponse));

    await uploadKeyEnvelope(basePaymentId, baseBuyerWallet, envelope);

    const [, init] = (global.fetch as jest.Mock).mock.calls[0];
    expect(JSON.parse(init.body).sale_tx_hash).toBeNull();
  });
});

describe('fetchKeyEnvelope', () => {
  it('GETs from /listings/:paymentId/key-envelope', async () => {
    useAuthStore.setState({ token: null, user: null, isAuthenticated: false });

    const mockResponse = {
      payment_id: basePaymentId,
      buyer_wallet: baseBuyerWallet,
      ephemeral_public_key: 'eph-pub',
      nonce: 'nonce',
      ciphertext: 'cipher',
      claimed_at: null,
      created_at: '2026-09-25T12:00:00Z',
    };
    (global.fetch as jest.Mock).mockResolvedValueOnce(jsonResponse(200, mockResponse));

    const result = await fetchKeyEnvelope(basePaymentId);

    expect(global.fetch).toHaveBeenCalledTimes(1);
    const [url] = (global.fetch as jest.Mock).mock.calls[0];
    expect(url).toBe(`http://localhost:8080/api/v1/marketplace/listings/${basePaymentId}/key-envelope`);
    expect(result).toEqual(mockResponse);
  });

  it('returns null on 404 instead of throwing', async () => {
    useAuthStore.setState({ token: null, user: null, isAuthenticated: false });
    (global.fetch as jest.Mock).mockResolvedValueOnce(jsonResponse(404, { message: 'No envelope yet' }));

    const result = await fetchKeyEnvelope(basePaymentId);

    expect(result).toBeNull();
  });

  it('throws on other errors (not 404)', async () => {
    useAuthStore.setState({ token: null, user: null, isAuthenticated: false });
    (global.fetch as jest.Mock).mockResolvedValueOnce(jsonResponse(500, { message: 'Server error' }));

    await expect(fetchKeyEnvelope(basePaymentId)).rejects.toThrow('Server error');
  });
});

describe('toKeyEnvelope', () => {
  it('maps stored envelope fields to KeyEnvelope shape', () => {
    const stored: any = {
      payment_id: basePaymentId,
      buyer_wallet: baseBuyerWallet,
      ephemeral_public_key: 'eph-pub-base64',
      nonce: 'nonce-base64',
      ciphertext: 'cipher-base64',
      claimed_at: null,
      created_at: '2026-09-25T12:00:00Z',
    };

    const result = toKeyEnvelope(stored);

    expect(result).toEqual({
      ephemeralPublicKey: 'eph-pub-base64',
      nonce: 'nonce-base64',
      ciphertext: 'cipher-base64',
    });
  });
});

describe('registerPushToken', () => {
  it('POSTs to /push-token with token and platform', async () => {
    const mockToken = 'mock-jwt-token';
    useAuthStore.setState({ token: mockToken, user: null, isAuthenticated: false });

    const mockResponse = 'Push token registered';
    (global.fetch as jest.Mock).mockResolvedValueOnce(jsonResponse(200, mockResponse));

    const result = await registerPushToken('device-token-123', 'ios');

    expect(global.fetch).toHaveBeenCalledTimes(1);
    const [url, init] = (global.fetch as jest.Mock).mock.calls[0];
    expect(url).toBe('http://localhost:8080/api/v1/marketplace/push-token');
    expect(init.method).toBe('POST');
    expect(JSON.parse(init.body)).toMatchObject({
      token: 'device-token-123',
      platform: 'ios',
    });
    expect(result).toBe(mockResponse);
  });

  it('throws MarketplaceApiError on registration failure', async () => {
    useAuthStore.setState({ token: 'token', user: null, isAuthenticated: false });
    (global.fetch as jest.Mock).mockResolvedValueOnce(jsonResponse(400, { message: 'Invalid platform' }));

    await expect(registerPushToken('token', 'invalid-platform' as any)).rejects.toThrow('Invalid platform');
  });
});
