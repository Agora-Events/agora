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
 * Creates a new `rpc.Server` instance targeting the configured Soroban RPC
 * endpoint.  Pass a custom `url` to override the default (useful in tests).
 */
export function createSorobanRpcServer(url: string = SOROBAN_RPC_URL): rpc.Server {
  return new rpc.Server(url, SERVER_OPTS);
}

/**
 * Lazily-initialised module-level singleton so callers that import
 * `sorobanRpc` directly always share the same instance.
 */
let _server: rpc.Server | null = null;

export function getSorobanRpcServer(): rpc.Server {
  if (!_server) {
    _server = createSorobanRpcServer();
  }
  return _server;
}

/**
 * Reset the singleton — used in tests to force fresh instantiation.
 */
export function resetSorobanRpcServer(): void {
  _server = null;
}
