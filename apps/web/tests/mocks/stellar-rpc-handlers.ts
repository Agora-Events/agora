import { http, HttpResponse } from "msw";

// ─── Stellar RPC Endpoint URLs ──────────────────────────────────────────────

const TESTNET_SOROBAN_RPC = "https://soroban-testnet.stellar.org";
const MAINNET_SOROBAN_RPC = "https://soroban.stellar.org";
const TESTNET_HORIZON = "https://horizon-testnet.stellar.org";
const MAINNET_HORIZON = "https://horizon.stellar.org";

// ─── Mock Data Fixtures ─────────────────────────────────────────────────────

/**
 * Mock XDR envelope for a successful transaction simulation.
 * Real XDR would be much longer; this is a simplified version for testing.
 */
const MOCK_SUCCESSFUL_TX_XDR =
  "AAAAAgAAAABmXD01AAAAZAAAAA4AAAAAAAAAAAAAAAA" +
  "BAAAAAAAAAAEAAAAEAAAAAAAAAASgAAAAAAAAAAA=";

/**
 * Mock XDR for a contract error simulation (e.g., Error(Contract, #101) - SoldOut).
 * This represents a HostFunctionError with an embedded contract error code.
 */
const MOCK_CONTRACT_ERROR_XDR =
  "AAAAAgAAAABmXD01AAAAZAAAAA4AAAAAAAAAAAAAAAA" +
  "BAAAAAAAAAAEAAAAEAAAAAAAAAASgAAAAAAAAAAE=";

/**
 * Mock data for a successful Soroban RPC simulateTransaction response.
 */
function mockSuccessfulSimulation(
  transactionEnvelope: string = MOCK_SUCCESSFUL_TX_XDR,
) {
  return {
    jsonrpc: "2.0",
    result: {
      transactionEnvelope,
      minResourceFee: "100",
      events: [
        {
          type: "contract",
          ledger: 1234567,
          txn_index: 0,
          operation_index: 0,
          topics: ["AAAADwAAAAVjb250cmFjdA=="],
          data: "AAAAAwAAAAA=",
        },
      ],
      error: null,
      latest_ledger: 1234567,
      latest_ledger_close_time: "1234567890",
      protocol_version: 21,
    },
    id: "1",
  };
}

/**
 * Mock data for a contract error simulation response.
 */
function mockContractErrorSimulation(errorCode: number = 101) {
  return {
    jsonrpc: "2.0",
    result: {
      transactionEnvelope: MOCK_CONTRACT_ERROR_XDR,
      minResourceFee: "100",
      events: [],
      error: `Error(Contract, #${errorCode})`,
      latest_ledger: 1234567,
      latest_ledger_close_time: "1234567890",
      protocol_version: 21,
    },
    id: "1",
  };
}

/**
 * Mock data for a successful sendTransaction response.
 */
function mockSendTransactionResponse() {
  const txHash = "0123456789abcdef0123456789abcdef01234567";
  return {
    jsonrpc: "2.0",
    result: {
      hash: txHash,
      status: "PENDING",
      latest_ledger: 1234567,
      latest_ledger_close_time: "1234567890",
    },
    id: "1",
  };
}

/**
 * Mock data for a transaction status query (via getTransaction).
 * Represents a successfully confirmed transaction on-chain.
 */
function mockGetTransactionSuccess() {
  return {
    jsonrpc: "2.0",
    result: {
      status: "SUCCESS",
      ledger: 1234580,
      createdAt: "2024-01-15T10:30:00Z",
      applicationOrder: 1,
      feePaid: "100",
      memoType: "text",
      memo: "ticket:event123",
    },
    id: "1",
  };
}

/**
 * Mock data for a transaction status query with failure status.
 */
function mockGetTransactionFailed(reason: string = "tx_failed") {
  return {
    jsonrpc: "2.0",
    result: {
      status: reason,
      ledger: 1234580,
      createdAt: "2024-01-15T10:30:00Z",
    },
    id: "1",
  };
}

/**
 * Mock Horizon account data with realistic XLM and USDC balances.
 * This represents a Stellar account with native XLM and USDC trustline.
 */
function mockHorizonAccount(publicKey: string) {
  return {
    id: publicKey,
    account_id: publicKey,
    balances: [
      {
        balance: "50.0000000",
        asset_type: "native",
      },
      {
        balance: "1000.0000000",
        asset_code: "USDC",
        asset_issuer: "GBBD47IF6LWK7P7MDEVSCWR7DPUWV3NY3DTQEVFL4NAT4AQH3ZLLFLA5",
        limit: "922337203685.4775807",
        buying_liabilities: "0.0000000",
        selling_liabilities: "0.0000000",
      },
    ],
    sequence: "123456789",
    subentry_count: 2,
    last_modified_ledger: 1234567,
    last_modified_time: "2024-01-15T10:30:00Z",
    thresholds: {
      low_threshold: 1,
      med_threshold: 1,
      high_threshold: 1,
    },
    flags: {
      auth_required: false,
      auth_revocable: false,
      auth_immutable: false,
      clawback_enabled: false,
    },
    signers: [
      {
        weight: 1,
        key: publicKey,
        type: "ed25519_public_key",
      },
    ],
    data: {},
    paging_token: "123456789",
  };
}

/**
 * Mock Horizon account data with no USDC trustline (e.g., new account).
 * Used to test trustline setup flows.
 */
function mockHorizonAccountNoTrustline(publicKey: string) {
  return {
    id: publicKey,
    account_id: publicKey,
    balances: [
      {
        balance: "10.0000000",
        asset_type: "native",
      },
    ],
    sequence: "1",
    subentry_count: 0,
    last_modified_ledger: 1234567,
    last_modified_time: "2024-01-15T10:30:00Z",
    thresholds: {
      low_threshold: 1,
      med_threshold: 1,
      high_threshold: 1,
    },
    flags: {
      auth_required: false,
      auth_revocable: false,
      auth_immutable: false,
      clawback_enabled: false,
    },
    signers: [
      {
        weight: 1,
        key: publicKey,
        type: "ed25519_public_key",
      },
    ],
    data: {},
    paging_token: "123456789",
  };
}

// ─── MSW Request Handlers ────────────────────────────────────────────────────

/**
 * MSW handlers for Stellar Soroban RPC and Horizon API endpoints.
 * Intercepts all Stellar network calls during tests, allowing offline execution
 * and deterministic error simulation.
 *
 * Usage in vitest.setup.ts:
 * ```ts
 * import { setupServer } from "msw/node";
 * import { stellarHandlers } from "@/tests/mocks/stellar-rpc-handlers";
 *
 * export const server = setupServer(...stellarHandlers);
 * ```
 */
export const stellarHandlers = [
  // ─── Soroban simulateTransaction (Testnet) ──────────────────────────────

  http.post(`${TESTNET_SOROBAN_RPC}`, async ({ request }) => {
    const body = (await request.json()) as {
      jsonrpc: string;
      method: string;
      params?: string[];
      id?: string;
    };

    if (body.method === "simulateTransaction") {
      // Simulate contract error by default; override in tests as needed
      // Check for a marker in transaction envelope to determine success/failure
      const txEnvelope = body.params?.[0] || "";

      // If envelope contains "ERROR" marker, return contract error
      if (txEnvelope.includes("ERROR")) {
        return HttpResponse.json(mockContractErrorSimulation(101));
      }

      // Otherwise return success
      return HttpResponse.json(mockSuccessfulSimulation(txEnvelope));
    }

    if (body.method === "sendTransaction") {
      return HttpResponse.json(mockSendTransactionResponse());
    }

    if (body.method === "getTransaction") {
      const txHash = body.params?.[0] || "unknown";

      // If hash contains "FAILED", simulate transaction failure
      if (txHash.includes("FAILED")) {
        return HttpResponse.json(mockGetTransactionFailed("tx_failed"));
      }

      // If hash contains "BAD_AUTH", simulate auth failure
      if (txHash.includes("BAD_AUTH")) {
        return HttpResponse.json(mockGetTransactionFailed("tx_bad_auth"));
      }

      // Otherwise return success
      return HttpResponse.json(mockGetTransactionSuccess());
    }

    // Unknown method — return JSON-RPC error
    return HttpResponse.json(
      {
        jsonrpc: "2.0",
        error: {
          code: -32601,
          message: "Method not found",
        },
        id: body.id || null,
      },
      { status: 400 },
    );
  }),

  // ─── Soroban simulateTransaction (Mainnet) ──────────────────────────────

  http.post(`${MAINNET_SOROBAN_RPC}`, async ({ request }) => {
    const body = (await request.json()) as {
      jsonrpc: string;
      method: string;
      params?: string[];
      id?: string;
    };

    if (body.method === "simulateTransaction") {
      return HttpResponse.json(mockSuccessfulSimulation());
    }

    if (body.method === "sendTransaction") {
      return HttpResponse.json(mockSendTransactionResponse());
    }

    if (body.method === "getTransaction") {
      return HttpResponse.json(mockGetTransactionSuccess());
    }

    return HttpResponse.json(
      {
        jsonrpc: "2.0",
        error: {
          code: -32601,
          message: "Method not found",
        },
        id: body.id || null,
      },
      { status: 400 },
    );
  }),

  // ─── Horizon GET /accounts/{accountId} (Testnet) ──────────────────────

  http.get(`${TESTNET_HORIZON}/accounts/:accountId`, ({ params }) => {
    const { accountId } = params;

    // Mock a new account with no USDC trustline
    if (typeof accountId === "string" && accountId.includes("NOTRUST")) {
      return HttpResponse.json(mockHorizonAccountNoTrustline(accountId));
    }

    // Default: account with XLM and USDC balances
    return HttpResponse.json(mockHorizonAccount(accountId as string));
  }),

  // ─── Horizon GET /accounts/{accountId} (Mainnet) ──────────────────────

  http.get(`${MAINNET_HORIZON}/accounts/:accountId`, ({ params }) => {
    const { accountId } = params;

    if (typeof accountId === "string" && accountId.includes("NOTRUST")) {
      return HttpResponse.json(mockHorizonAccountNoTrustline(accountId));
    }

    return HttpResponse.json(mockHorizonAccount(accountId as string));
  }),

  // ─── Horizon 404 for missing accounts ────────────────────────────────────

  http.get(`${TESTNET_HORIZON}/accounts/*`, () => {
    return HttpResponse.json(
      {
        type: "https://stellar.org/horizon-errors/not_found",
        title: "Resource Missing",
        status: 404,
        detail: "The resource at the url requested was not found.",
        instance: "some-request-id",
      },
      { status: 404 },
    );
  }),

  http.get(`${MAINNET_HORIZON}/accounts/*`, () => {
    return HttpResponse.json(
      {
        type: "https://stellar.org/horizon-errors/not_found",
        title: "Resource Missing",
        status: 404,
        detail: "The resource at the url requested was not found.",
        instance: "some-request-id",
      },
      { status: 404 },
    );
  }),
];

// ─── Re-export mock data helpers for test customization ──────────────────────

/**
 * Helpers for tests to generate custom mock responses.
 *
 * @example
 * ```ts
 * import { mockContractErrorSimulation } from "@/tests/mocks/stellar-rpc-handlers";
 *
 * // In your test, override the default handler:
 * server.use(
 *   http.post(TESTNET_SOROBAN_RPC, () => {
 *     return HttpResponse.json(mockContractErrorSimulation(102));
 *   })
 * );
 * ```
 */
export {
  mockSuccessfulSimulation,
  mockContractErrorSimulation,
  mockSendTransactionResponse,
  mockGetTransactionSuccess,
  mockGetTransactionFailed,
  mockHorizonAccount,
  mockHorizonAccountNoTrustline,
};
