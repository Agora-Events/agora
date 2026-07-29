/**
 * sorobanClient.ts
 *
 * Thin, dependency-light wrapper around `@stellar/stellar-sdk`'s Soroban RPC
 * surface (`rpc.Server`). This module intentionally knows nothing about the
 * `ticket_payment` contract itself — it only knows how to:
 *
 *   1. Load a source account from the network.
 *   2. Build + simulate + "assemble" a transaction that invokes a contract.
 *   3. Sign a transaction with a `Keypair`.
 *   4. Submit a signed transaction to the RPC server.
 *   5. Poll the RPC server until a submitted transaction reaches a terminal
 *      status (SUCCESS / FAILED), with bounded retries.
 *   6. Best-effort decode a Soroban contract error (`Error(Contract, #N)`)
 *      out of whatever shape of error/response the SDK handed back.
 *
 * Contract-specific concerns (which function to call, how to encode its
 * arguments, what the error codes mean) live in `ticketPaymentContract.ts`.
 */

import {
  Account,
  Keypair,
  Networks,
  Operation,
  Transaction,
  TransactionBuilder,
  BASE_FEE,
  rpc,
} from '@stellar/stellar-sdk';

// ── Network configuration ───────────────────────────────────────────────────

/**
 * Soroban RPC endpoint. Overridable via `EXPO_PUBLIC_SOROBAN_RPC_URL` so the
 * app can be pointed at a local `stellar quickstart` node or futurenet
 * without a code change.
 */
export const SOROBAN_RPC_URL =
  process.env.EXPO_PUBLIC_SOROBAN_RPC_URL ?? 'https://soroban-testnet.stellar.org';

export const SOROBAN_NETWORK_PASSPHRASE = Networks.TESTNET;

/** Lazily-constructed singleton so screens/tests don't each open a new server. */
let rpcServerSingleton: rpc.Server | null = null;

export function getSorobanServer(): rpc.Server {
  if (!rpcServerSingleton) {
    rpcServerSingleton = new rpc.Server(SOROBAN_RPC_URL, { allowHttp: false });
  }
  return rpcServerSingleton;
}

/** Test-only escape hatch to reset the singleton between test files. */
export function __resetSorobanServerForTests(): void {
  rpcServerSingleton = null;
}

// ── Errors ───────────────────────────────────────────────────────────────────

export type SorobanFailureStage =
  | 'load-account'
  | 'simulate'
  | 'sign'
  | 'submit'
  | 'confirm'
  | 'timeout'
  | 'unknown';

/**
 * Normalized error type raised anywhere in the build → sign → submit → poll
 * pipeline. UI code should only ever need to read `.stage`, `.contractErrorCode`
 * and `.message` — never SDK-internal shapes.
 */
export class SorobanTransactionError extends Error {
  readonly stage: SorobanFailureStage;
  /** Numeric Soroban contract error code, if one could be decoded. */
  readonly contractErrorCode: number | null;
  /** The raw underlying error/response, kept for logging/debugging only. */
  readonly cause: unknown;
  /** Transaction hash, if a hash was assigned before the failure occurred. */
  readonly txHash: string | null;

  constructor(params: {
    stage: SorobanFailureStage;
    message: string;
    contractErrorCode?: number | null;
    cause?: unknown;
    txHash?: string | null;
  }) {
    super(params.message);
    this.name = 'SorobanTransactionError';
    this.stage = params.stage;
    this.contractErrorCode = params.contractErrorCode ?? null;
    this.cause = params.cause ?? null;
    this.txHash = params.txHash ?? null;
  }
}

// ── Helpers ──────────────────────────────────────────────────────────────────

export function delay(ms: number): Promise<void> {
  return new Promise((resolve) => setTimeout(resolve, ms));
}

/**
 * Soroban (and Horizon) errors surface a contract's `panic_with_error!` /
 * `#[contracterror]` code in a human-readable diagnostic string of the shape
 * `Error(Contract, #6)`. The exact container (thrown exception message,
 * `simulateTransaction().error`, a `getTransaction()` result's diagnostic
 * events, etc.) varies by SDK version and failure path, so rather than
 * pattern-matching a dozen possible object shapes we flatten whatever we were
 * given into a single string and regex out the first contract error code.
 *
 * This is deliberately best-effort: if no code is found we return null and
 * callers fall back to a generic message.
 */
export function extractContractErrorCode(source: unknown): number | null {
  const text = flattenErrorToString(source);
  if (!text) return null;

  // Standard soroban-sdk panic format: "Error(Contract, #6)"
  const contractMatch = text.match(/Error\(Contract,\s*#(\d+)\)/i);
  if (contractMatch) {
    return parseInt(contractMatch[1], 10);
  }

  // Some RPC responses instead surface a bare "HostError: ... status Ok(ContractError(6))"
  const hostErrorMatch = text.match(/ContractError\((\d+)\)/i);
  if (hostErrorMatch) {
    return parseInt(hostErrorMatch[1], 10);
  }

  return null;
}

function flattenErrorToString(source: unknown): string {
  if (source == null) return '';
  if (typeof source === 'string') return source;
  if (source instanceof Error) {
    return `${source.name}: ${source.message}\n${(source as any).stack ?? ''}`;
  }

  try {
    // `simulateTransaction` error responses and `getTransaction` FAILED
    // responses are plain objects with fields like `.error`, `.resultXdr`,
    // `.diagnosticEvents` (an array of xdr.DiagnosticEvent, which have a
    // `.toXDR('base64')`/`.toString()` when stringified via JSON as buffers).
    return JSON.stringify(source, (_key, value) => {
      if (value && typeof value === 'object' && typeof value.toXDR === 'function') {
        try {
          return value.toXDR('base64');
        } catch {
          return String(value);
        }
      }
      if (typeof value === 'bigint') return value.toString();
      return value;
    });
  } catch {
    return String(source);
  }
}

// ── Account loading ──────────────────────────────────────────────────────────

export async function loadSorobanAccount(publicKey: string): Promise<Account> {
  try {
    return await getSorobanServer().getAccount(publicKey);
  } catch (error) {
    throw new SorobanTransactionError({
      stage: 'load-account',
      message:
        'Could not load your Stellar account from the network. Make sure the account exists ' +
        'and is funded on Testnet.',
      cause: error,
    });
  }
}

export async function getLatestLedgerSequence(): Promise<number> {
  const latest = await getSorobanServer().getLatestLedger();
  return latest.sequence;
}

// ── Build + simulate + assemble ─────────────────────────────────────────────

export interface PreparedTransaction {
  /** Fully-assembled transaction, ready to sign. Soroban resource fees/footprint applied. */
  transaction: Transaction;
  /** Simulation result, kept around for diagnostics (e.g. min resource fee). */
  simulation: rpc.Api.SimulateTransactionSuccessResponse;
}

/**
 * Builds a single-operation transaction invoking a contract, simulates it
 * against the current ledger, and assembles the simulation's footprint /
 * resource fees back onto the transaction so it's ready to sign.
 */
export async function buildAndSimulateContractCall(params: {
  sourceAccount: Account;
  operation: ReturnType<typeof Operation.invokeHostFunction>;
  timeoutSeconds?: number;
  baseFee?: string;
}): Promise<PreparedTransaction> {
  const { sourceAccount, operation, timeoutSeconds = 60, baseFee = BASE_FEE } = params;

  const builtTx = new TransactionBuilder(sourceAccount, {
    fee: baseFee,
    networkPassphrase: SOROBAN_NETWORK_PASSPHRASE,
  })
    .addOperation(operation)
    .setTimeout(timeoutSeconds)
    .build();

  const server = getSorobanServer();
  let simulation: rpc.Api.SimulateTransactionResponse;
  try {
    simulation = await server.simulateTransaction(builtTx);
  } catch (error) {
    throw new SorobanTransactionError({
      stage: 'simulate',
      message: 'Failed to simulate the transaction against the Soroban network.',
      contractErrorCode: extractContractErrorCode(error),
      cause: error,
    });
  }

  if (rpc.Api.isSimulationError(simulation)) {
    throw new SorobanTransactionError({
      stage: 'simulate',
      message: simulation.error || 'The contract rejected this transaction during simulation.',
      contractErrorCode: extractContractErrorCode(simulation),
      cause: simulation,
    });
  }

  if (rpc.Api.isSimulationRestore(simulation)) {
    throw new SorobanTransactionError({
      stage: 'simulate',
      message:
        'This transaction requires restoring expired ledger entries first. Please try again ' +
        '— the wallet will attempt to restore automatically on the next run.',
      cause: simulation,
    });
  }

  let assembled: Transaction;
  try {
    assembled = rpc.assembleTransaction(builtTx, simulation).build();
  } catch (error) {
    throw new SorobanTransactionError({
      stage: 'simulate',
      message: 'Failed to assemble the simulated transaction.',
      cause: error,
    });
  }

  return { transaction: assembled, simulation };
}

// ── Sign ─────────────────────────────────────────────────────────────────────

export function signTransaction(transaction: Transaction, keypair: Keypair): Transaction {
  try {
    transaction.sign(keypair);
    return transaction;
  } catch (error) {
    throw new SorobanTransactionError({
      stage: 'sign',
      message: 'Failed to sign the transaction with your stored wallet key.',
      cause: error,
    });
  }
}

// ── Submit ───────────────────────────────────────────────────────────────────

export async function submitTransaction(
  transaction: Transaction
): Promise<rpc.Api.SendTransactionResponse> {
  const server = getSorobanServer();
  let response: rpc.Api.SendTransactionResponse;
  try {
    response = await server.sendTransaction(transaction);
  } catch (error) {
    throw new SorobanTransactionError({
      stage: 'submit',
      message: 'Failed to submit the transaction to the Soroban network.',
      contractErrorCode: extractContractErrorCode(error),
      cause: error,
    });
  }

  if (response.status === 'ERROR') {
    throw new SorobanTransactionError({
      stage: 'submit',
      message: 'The network rejected the transaction.',
      contractErrorCode: extractContractErrorCode(response.errorResult ?? response),
      cause: response,
      txHash: response.hash ?? null,
    });
  }

  // 'DUPLICATE' means an identical envelope is already in flight for this
  // hash — that's fine, we can still poll on the same hash.
  return response;
}

// ── Poll for finality ────────────────────────────────────────────────────────

export interface PollOptions {
  /** Milliseconds between polls. Default 2000ms. */
  intervalMs?: number;
  /** Maximum number of polls before giving up. Default 30 (~60s at 2s interval). */
  maxAttempts?: number;
  /** Invoked once per poll attempt with the attempt number (1-indexed). */
  onAttempt?: (attempt: number, maxAttempts: number) => void;
}

/**
 * Polls `getTransaction` until the transaction leaves the `NOT_FOUND` state,
 * i.e. until the ledger has either applied it (SUCCESS) or rejected it
 * (FAILED). Throws `SorobanTransactionError` for both a FAILED result and a
 * polling timeout so callers have one error type to handle.
 */
export async function pollTransactionUntilFinal(
  hash: string,
  options: PollOptions = {}
): Promise<rpc.Api.GetSuccessfulTransactionResponse> {
  const { intervalMs = 2000, maxAttempts = 30, onAttempt } = options;
  const server = getSorobanServer();

  for (let attempt = 1; attempt <= maxAttempts; attempt++) {
    onAttempt?.(attempt, maxAttempts);

    let response: rpc.Api.GetTransactionResponse;
    try {
      response = await server.getTransaction(hash);
    } catch (_error) {
      // Transient network hiccup while polling — keep retrying rather than
      // failing the whole checkout on one dropped request.
      await delay(intervalMs);
      continue;
    }

    if (response.status === rpc.Api.GetTransactionStatus.SUCCESS) {
      return response as rpc.Api.GetSuccessfulTransactionResponse;
    }

    if (response.status === rpc.Api.GetTransactionStatus.FAILED) {
      throw new SorobanTransactionError({
        stage: 'confirm',
        message: 'The transaction was included in the ledger but failed.',
        contractErrorCode: extractContractErrorCode(response),
        cause: response,
        txHash: hash,
      });
    }

    // NOT_FOUND — keep waiting.
    await delay(intervalMs);
  }

  throw new SorobanTransactionError({
    stage: 'timeout',
    message:
      'Timed out waiting for ledger confirmation. Your transaction may still confirm — check ' +
      'the transaction hash on Stellar Expert before retrying.',
    txHash: hash,
  });
}
