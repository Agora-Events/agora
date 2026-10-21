/**
 * Soroban RPC client — Issue #1493
 *
 * Centralises all Soroban RPC Server instantiation so every part of the
 * codebase targets the same endpoint, timeout, and retry settings.
 *
 * Compatible with @stellar/stellar-sdk v13+ where the Soroban RPC module is
 * exposed as `rpc` (not the legacy `SorobanRpc` alias).
 */

import { rpc, Networks } from "@stellar/stellar-sdk";

// ---------------------------------------------------------------------------
// Environment configuration
// ---------------------------------------------------------------------------

const SOROBAN_RPC_URL =
  process.env.STELLAR_RPC_URL ?? "https://soroban-testnet.stellar.org";

/**
 * Stellar network passphrase configured from the environment or defaulting to Testnet.
 * Used for signing transactions and identifying the target network.
 *
 * @type {string}
 */
export const STELLAR_NETWORK_PASSPHRASE: string =
  process.env.STELLAR_NETWORK_PASSPHRASE ?? Networks.TESTNET;

// ---------------------------------------------------------------------------
// Server options
// ---------------------------------------------------------------------------

/**
 * Options passed to every `rpc.Server` instance.
 *
 * - `allowHttp`: must be `false` in production; the default endpoint is HTTPS.
 * - `timeout`: overrides the SDK default (60 s) with a tighter 30 s limit so
 *   simulations that hang don't block the UI indefinitely.
 * - Additional request headers can be appended here if your RPC provider
 *   requires an API key.
 */
const SERVER_OPTS: rpc.RpcServer.Options = {
  allowHttp: false,
  timeout: 30_000,
};

// ---------------------------------------------------------------------------
// Factory & singleton
// ---------------------------------------------------------------------------

/**
 * Creates and returns a new `rpc.Server` instance targeting the configured Soroban RPC endpoint.
 *
 * @param {string} [url=SOROBAN_RPC_URL] - The Soroban RPC endpoint URL. Defaults to the environment-configured URL or Testnet.
 * @returns {rpc.Server} A configured Soroban RPC Server instance with predefined timeout and security settings.
 *
 * @example
 * ```ts
 * const customServer = createSorobanRpcServer("https://soroban-testnet.stellar.org");
 * ```
 */
export function createSorobanRpcServer(url: string = SOROBAN_RPC_URL): rpc.Server {
  return new rpc.Server(url, SERVER_OPTS);
}

/**
 * Lazily-initialised module-level singleton so callers that import
 * `sorobanRpc` directly always share the same instance.
 */
let _server: rpc.Server | null = null;

/**
 * Retrieves the shared singleton `rpc.Server` instance, lazily instantiating it if necessary.
 *
 * @returns {rpc.Server} The shared Soroban RPC Server singleton instance.
 *
 * @example
 * ```ts
 * const server = getSorobanRpcServer();
 * const health = await server.getHealth();
 * ```
 */
export function getSorobanRpcServer(): rpc.Server {
  if (!_server) {
    _server = createSorobanRpcServer();
  }
  return _server;
}

/**
 * Resets the cached singleton `rpc.Server` instance to `null`.
 * Primarily used in test suites to force fresh instantiation between tests.
 *
 * @returns {void}
 *
 * @example
 * ```ts
 * beforeEach(() => {
 *   resetSorobanRpcServer();
 * });
 * ```
 */
export function resetSorobanRpcServer(): void {
  _server = null;
}

