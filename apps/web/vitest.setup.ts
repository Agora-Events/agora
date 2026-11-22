import '@testing-library/jest-dom';
import { cleanup } from '@testing-library/react';
import { afterEach, beforeAll, afterAll } from 'vitest';
import { http, HttpResponse } from 'msw';
import { setupServer } from 'msw/node';
import { stellarHandlers } from './tests/mocks/stellar-rpc-handlers';

// ─── MSW Server Setup ────────────────────────────────────────────────────────
// All Stellar RPC and Horizon calls are intercepted by MSW, allowing tests
// to run fully offline without depending on live Testnet availability.

const server = setupServer(
  ...stellarHandlers,
  http.get('http://localhost:3001/api/v1/marketplace/listings', ({ request }) => {
    const { searchParams } = new URL(request.url);
    const hasListing = searchParams.get('event_id') === '1' && searchParams.get('status') === 'active';

    return HttpResponse.json({
      data: hasListing
        ? [{
            payment_id: 'resale-payment-1',
            seller_wallet: 'GTESTSELLER',
            price_stroops: 25_000_000,
            created_at: '2026-01-01T00:00:00Z',
          }]
        : [],
    });
  }),
);

// Start MSW server before all tests
beforeAll(() => {
  server.listen({ onUnhandledRequest: 'error' });
});

// Reset handlers and cleanup after each test
afterEach(() => {
  server.resetHandlers();
  cleanup();
});

// Stop MSW server after all tests complete
afterAll(() => {
  server.close();
});
