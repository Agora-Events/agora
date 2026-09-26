import '@testing-library/jest-dom';
import { cleanup } from '@testing-library/react';
import { afterEach, beforeAll, afterAll } from 'vitest';
import { setupServer } from 'msw/node';
import { stellarHandlers } from './tests/mocks/stellar-rpc-handlers';

// ─── MSW Server Setup ────────────────────────────────────────────────────────
// All Stellar RPC and Horizon calls are intercepted by MSW, allowing tests
// to run fully offline without depending on live Testnet availability.

const server = setupServer(...stellarHandlers);

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
