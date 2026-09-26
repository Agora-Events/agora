/**
 * ticketPaymentContract.ts
 *
 * Everything specific to the on-chain `ticket_payment` Soroban contract:
 * argument encoding for `process_payment`, the USDC allowance step required
 * before it, the contract's `#[contracterror]` table translated into
 * human-readable messages, and a single high-level `purchaseTickets()`
 * orchestrator that the mobile checkout screen drives.
 *
 * Contract reference: contract/contracts/ticket_payment/src/contract.rs
 *
 *   pub fn process_payment(
 *       env: Env,
 *       payment_id: String,
 *       event_id: String,
 *       ticket_tier_id: String,
 *       buyer_address: Address,
 *       recipient_address: Option<Address>,
 *       token_address: Address,
 *       amount: i128,          // price for ONE ticket, in token stroops
 *       quantity: u32,
 *       options: PurchaseOptions,
 *       validation_hash: BytesN<32>,
 *   ) -> Result<String, TicketPaymentError>
 *
 * Note on naming: issue #1005 refers to this entry point as
 * `process_purchase`. The function actually deployed on the contract is
 * `process_payment` — this module targets the real contract.
 *
 * `process_payment` requires the buyer to have already granted the contract
 * a USDC allowance covering the full order (see `InsufficientAllowance`,
 * error code 12), so a full purchase is really two signed transactions:
 *
 *   1. `approve` on the USDC Stellar Asset Contract (spender = ticket_payment)
 *   2. `process_payment` on the ticket_payment contract
 */

import {
  Address,
  Asset,
  Contract,
  Keypair,
  hash as sha256,
  nativeToScVal,
  scValToNative,
  xdr,
} from '@stellar/stellar-sdk';
import { StellarWalletManager } from './stellar';
import {
  SorobanTransactionError,
  SOROBAN_NETWORK_PASSPHRASE,
  buildAndSimulateContractCall,
  getLatestLedgerSequence,
  loadSorobanAccount,
  pollTransactionUntilFinal,
  signTransaction,
  submitTransaction,
} from './sorobanClient';

// ── Contract addresses ───────────────────────────────────────────────────────

/**
 * `ticket_payment` contract ID on Stellar Testnet. Placeholder value —
 * update once the production deployment ID is published, following the same
 * convention as `EVENT_REGISTRY_CONTRACT` in `services/staking.ts`.
 */
export const TICKET_PAYMENT_CONTRACT_ID =
  process.env.EXPO_PUBLIC_TICKET_PAYMENT_CONTRACT_ID ??
  'CBQHNAXSI55GX2GN6D67GK7BHVPSLJUGZQEU7WJ5LKR5PNUCGLIMAO4K';

/** Classic USDC issuer used elsewhere in the app (`services/stellar.ts`). */
const USDC_ASSET_CODE = 'USDC';
const USDC_ISSUER = 'GBBD47IF6LWK7P7MABDHYZCYYZ277OEX22PQ6CHYA67BDN6C7CR27STN';

/** USDC has 7 decimal places on Stellar, same as XLM stroops. */
export const USDC_DECIMALS = 7;
const USDC_STROOP_FACTOR = 10_000_000; // 10^7

/**
 * The Soroban Asset Contract (SAC) ID that wraps the classic USDC asset.
 * Derived deterministically from the asset + network rather than hardcoded,
 * so it always matches whichever network passphrase the app is pointed at.
 */
export function getUsdcTokenContractId(): string {
  return new Asset(USDC_ASSET_CODE, USDC_ISSUER).contractId(SOROBAN_NETWORK_PASSPHRASE);
}

/** Ledgers of validity granted to the USDC allowance (~1 hour at 5s/ledger). */
const APPROVAL_LEDGER_WINDOW = 720;

// ── Amount helpers ───────────────────────────────────────────────────────────

/** Converts a decimal USDC amount (e.g. `12.5`) into base-unit stroops. */
export function usdcToStroops(amountUsdc: number): bigint {
  if (!Number.isFinite(amountUsdc) || amountUsdc < 0) {
    throw new Error(`Invalid USDC amount: ${amountUsdc}`);
  }
  return BigInt(Math.round(amountUsdc * USDC_STROOP_FACTOR));
}

/** Converts base-unit stroops back into a decimal USDC amount. */
export function stroopsToUsdc(stroops: bigint): number {
  return Number(stroops) / USDC_STROOP_FACTOR;
}

/** Formats a decimal USDC amount for display, e.g. `12.50`. */
export function formatUsdc(amountUsdc: number): string {
  return amountUsdc.toFixed(2);
}

// ── PurchaseOptions ScVal encoding ──────────────────────────────────────────

export interface PurchaseOptionsInput {
  /** SHA-256 preimage for the legacy global discount code system. Rarely used by mobile. */
  codePreimage?: Uint8Array | null;
  /** Referrer's Stellar public key, if this purchase should attribute a referral reward. */
  referrerAddress?: string | null;
  /** Per-event limited-time discount code string. */
  discountCode?: string | null;
}

/**
 * Builds the XDR `ScVal` for the contract's `PurchaseOptions` struct.
 *
 * `#[contracttype]` structs are encoded by soroban-sdk as an `ScVal::Map`
 * whose entries are `ScVal::Symbol(field_name) -> ScVal`, sorted
 * lexicographically by field name. For `PurchaseOptions` that order is:
 * `code_preimage`, `discount_code`, `referrer`.
 *
 * `Option<T>::None` is encoded as `ScVal::Void`; `Some(x)` is encoded as
 * `x`'s own ScVal directly (Option is not a separate wrapper type in XDR).
 */
export function buildPurchaseOptionsScVal(options: PurchaseOptionsInput): xdr.ScVal {
  const entries: xdr.ScMapEntry[] = [
    new xdr.ScMapEntry({
      key: xdr.ScVal.scvSymbol('code_preimage'),
      val: options.codePreimage
        ? nativeToScVal(Buffer.from(options.codePreimage), { type: 'bytes' })
        : xdr.ScVal.scvVoid(),
    }),
    new xdr.ScMapEntry({
      key: xdr.ScVal.scvSymbol('discount_code'),
      val: options.discountCode
        ? nativeToScVal(options.discountCode, { type: 'string' })
        : xdr.ScVal.scvVoid(),
    }),
    new xdr.ScMapEntry({
      key: xdr.ScVal.scvSymbol('referrer'),
      val: options.referrerAddress ? new Address(options.referrerAddress).toScVal() : xdr.ScVal.scvVoid(),
    }),
  ];

  return xdr.ScVal.scvMap(entries);
}

// ── Validation secret / hash ─────────────────────────────────────────────────

export interface PurchaseSecret {
  /** Raw 32-byte secret; kept client-side and later used to prove ticket ownership at check-in. */
  secretBytes: Uint8Array;
  /** SHA-256 of `secretBytes`, sent on-chain as `validation_hash`. */
  hash: Buffer;
}

/**
 * Generates the random secret + its SHA-256 digest submitted as the
 * contract's `validation_hash` argument. `storage::verify_secret` (see
 * `contract/contracts/ticket_payment/src/storage.rs`) later checks a raw
 * secret against this stored hash during ticket check-in, so the raw secret
 * must be persisted locally (never sent to the contract) for that flow.
 */
export function generatePurchaseSecret(): PurchaseSecret {
  const secretBytes = Keypair.random().rawSecretKey(); // 32 cryptographically-random bytes
  const digest = sha256(Buffer.from(secretBytes));
  return { secretBytes, hash: digest };
}

// ── Payment ID ────────────────────────────────────────────────────────────────

/**
 * Generates a payment id unique enough for a single buyer session. The
 * contract stores one `Payment` record per ticket in a batch purchase
 * (splitting `payment_id` into `p-0`, `p-1`, ... internally when
 * `quantity > 1`), so this only needs to be unique per checkout attempt.
 */
export function generatePaymentId(eventId: string, buyerPublicKey: string): string {
  const shortBuyer = buyerPublicKey.slice(0, 6);
  const timestamp = Date.now().toString(36);
  const random = Math.random().toString(36).slice(2, 8);
  return `pay-${eventId}-${shortBuyer}-${timestamp}-${random}`.slice(0, 64);
}

// ── Contract error table ────────────────────────────────────────────────────

/**
 * Mirrors `contract/contracts/ticket_payment/src/error.rs` `TicketPaymentError`.
 * Keep in sync with that file when the contract's error enum changes.
 */
export const TICKET_PAYMENT_ERROR_MESSAGES: Record<number, string> = {
  1: 'This contract has already been initialized.',
  2: 'One of the addresses provided is not a valid Stellar address.',
  3: 'The ticket payment contract has not been initialized yet.',
  4: 'This event could not be found.',
  5: 'This event is not currently active.',
  6: 'USDC is not accepted for this event right now. Please try again shortly.',
  7: 'This ticket tier is sold out.',
  8: 'We could not find a record of this payment.',
  9: 'This payment is not in a state that allows this action.',
  10: 'This ticket is not eligible for a refund.',
  11: 'The selected ticket tier no longer exists — the event may have changed its pricing.',
  12: 'Your USDC spending approval has not gone through yet. Please try again in a moment.',
  13: 'We could not verify your payment reached the contract. No funds were taken — please retry.',
  14: 'A math overflow occurred while calculating the order total. Please try a smaller quantity.',
  15: 'You cannot refer yourself for this purchase.',
  16: "The event's price changed. Please refresh and try again.",
  17: 'The ticket price has changed since you loaded this screen. Please refresh and try again.',
  18: 'That discount code is not valid.',
  19: 'That discount code has already been used.',
  20: 'You are not authorized to perform this action.',
  21: 'This event has not completed yet.',
  22: 'There are no funds available to withdraw for this event.',
  23: 'The refund window for this ticket has passed.',
  24: 'This withdrawal would exceed the organizer’s daily withdrawal cap.',
  25: 'There are insufficient platform fees available for this action.',
  26: 'The resale price exceeds the anti-scalping cap set for this ticket.',
  27: 'Ticket sales are temporarily paused. Please try again later.',
  35: 'This event has been cancelled.',
  36: 'This event is currently under dispute and cannot process payments.',
  37: 'This scanner is not authorized to check tickets in for this event.',
  38: 'This ticket has already been used.',
  39: 'This event did not meet its funding goal.',
  40: 'Pricing oracle is not configured for this token.',
  41: "The token's live price is currently unavailable. Please try again shortly.",
  42: 'The token price moved outside the allowed range while your order was being built. Please retry.',
  43: 'The configured price slippage tolerance is invalid.',
  44: 'This ticket tier is not currently up for auction.',
  45: 'Your bid is lower than the current highest bid.',
  46: 'This auction has already ended.',
  47: 'This auction has not ended yet.',
  48: 'This ticket tier is not an auction tier.',
  49: 'You are not authorized to perform this governance action.',
  50: 'This governance proposal could not be found.',
  51: 'This governance proposal is not active.',
  52: 'You have already voted on this proposal.',
  53: 'The voting period for this proposal has not elapsed yet.',
  54: 'This proposal does not have enough votes yet.',
  55: 'This governance proposal has expired.',
  56: "The token's price feed is stale. Please try again shortly.",
  57: 'Cannot remove the last remaining governor.',
  58: 'The fee percentage provided is invalid.',
  59: 'This event has already ended.',
  60: 'This ticket cannot be transferred.',
  61: 'The secret provided does not match this ticket.',
  62: 'That discount code has expired.',
  63: 'That discount code has reached its maximum number of uses.',
  64: 'This ticket is not listed for resale.',
  65: 'This listing is no longer active — it may have been cancelled or already sold.',
  66: 'This ticket is already listed for resale. Cancel the existing listing first.',
  67: 'The royalty percentage provided is invalid.',
};

export const INSUFFICIENT_ALLOWANCE_ERROR_CODE = 12;

export function describeContractError(code: number | null): string {
  if (code == null) return 'The contract rejected the transaction for an unknown reason.';
  return (
    TICKET_PAYMENT_ERROR_MESSAGES[code] ??
    `The contract rejected the transaction (error code ${code}).`
  );
}

/**
 * Turns any error thrown by this module (or by the lower-level
 * `sorobanClient`) into a single readable string safe to show in the UI.
 */
export function describePurchaseFailure(error: unknown): string {
  if (error instanceof SorobanTransactionError) {
    if (error.contractErrorCode != null) {
      return describeContractError(error.contractErrorCode);
    }
    return error.message;
  }
  if (error instanceof Error) return error.message;
  return 'Something went wrong while processing your purchase.';
}

// ── Approve (USDC allowance) ─────────────────────────────────────────────────

export interface ApproveAllowanceParams {
  buyerKeypair: Keypair;
  totalAmountStroops: bigint;
}

export interface SubmittedTx {
  hash: string;
}

/**
 * Grants the ticket_payment contract an allowance to pull `totalAmountStroops`
 * of USDC from the buyer. Required before `process_payment` — see
 * `InsufficientAllowance` in `contract.rs`.
 */
export async function submitUsdcApproval(
  params: ApproveAllowanceParams
): Promise<SubmittedTx> {
  const { buyerKeypair, totalAmountStroops } = params;
  const buyerPublicKey = buyerKeypair.publicKey();

  const [account, latestLedger] = await Promise.all([
    loadSorobanAccount(buyerPublicKey),
    getLatestLedgerSequence(),
  ]);

  const tokenContract = new Contract(getUsdcTokenContractId());
  const fromAddress = new Address(buyerPublicKey);
  const spenderAddress = new Address(TICKET_PAYMENT_CONTRACT_ID);
  const expirationLedger = latestLedger + APPROVAL_LEDGER_WINDOW;

  const operation = tokenContract.call(
    'approve',
    fromAddress.toScVal(),
    spenderAddress.toScVal(),
    nativeToScVal(totalAmountStroops, { type: 'i128' }),
    nativeToScVal(expirationLedger, { type: 'u32' })
  );

  const { transaction } = await buildAndSimulateContractCall({
    sourceAccount: account,
    operation,
  });

  const signed = signTransaction(transaction, buyerKeypair);
  const sendResponse = await submitTransaction(signed);
  return { hash: sendResponse.hash };
}

// ── process_payment ──────────────────────────────────────────────────────────

export interface ProcessPaymentParams {
  buyerKeypair: Keypair;
  eventId: string;
  tierId: string;
  quantity: number;
  /** Price of a SINGLE ticket in decimal USDC, e.g. `25.00`. */
  unitPriceUsdc: number;
  /** Optional recipient (gifting) — defaults to the buyer when omitted. */
  recipientAddress?: string | null;
  paymentId: string;
  validationHash: Buffer;
  options?: PurchaseOptionsInput;
}

export interface ProcessPaymentResult {
  hash: string;
  /** Contract's return value: the `payment_id` echoed back on success. */
  returnedPaymentId: string | null;
}

export async function submitProcessPayment(
  params: ProcessPaymentParams
): Promise<ProcessPaymentResult> {
  const {
    buyerKeypair,
    eventId,
    tierId,
    quantity,
    unitPriceUsdc,
    recipientAddress,
    paymentId,
    validationHash,
    options = {},
  } = params;

  const buyerPublicKey = buyerKeypair.publicKey();
  const account = await loadSorobanAccount(buyerPublicKey);

  const contract = new Contract(TICKET_PAYMENT_CONTRACT_ID);
  const buyerScAddress = new Address(buyerPublicKey);
  const tokenScAddress = new Address(getUsdcTokenContractId());
  const amountStroops = usdcToStroops(unitPriceUsdc);

  const args: xdr.ScVal[] = [
    nativeToScVal(paymentId, { type: 'string' }),
    nativeToScVal(eventId, { type: 'string' }),
    nativeToScVal(tierId, { type: 'string' }),
    buyerScAddress.toScVal(),
    recipientAddress ? new Address(recipientAddress).toScVal() : xdr.ScVal.scvVoid(),
    tokenScAddress.toScVal(),
    nativeToScVal(amountStroops, { type: 'i128' }),
    nativeToScVal(quantity, { type: 'u32' }),
    buildPurchaseOptionsScVal(options),
    nativeToScVal(validationHash, { type: 'bytes' }),
  ];

  const operation = contract.call('process_payment', ...args);

  const { transaction, simulation } = await buildAndSimulateContractCall({
    sourceAccount: account,
    operation,
  });

  const signed = signTransaction(transaction, buyerKeypair);
  const sendResponse = await submitTransaction(signed);

  let returnedPaymentId: string | null = null;
  try {
    if (simulation.result?.retval) {
      returnedPaymentId = scValToNative(simulation.result.retval) as string;
    }
  } catch {
    returnedPaymentId = null;
  }

  return { hash: sendResponse.hash, returnedPaymentId };
}

// ── Full orchestration ──────────────────────────────────────────────────────

export type PurchaseProgressStage =
  | 'loading-wallet'
  | 'building-approval'
  | 'signing-approval'
  | 'submitting-approval'
  | 'confirming-approval'
  | 'building-payment'
  | 'signing-payment'
  | 'submitting-payment'
  | 'confirming-payment';

export interface PurchaseProgressEvent {
  stage: PurchaseProgressStage;
  attempt?: number;
  maxAttempts?: number;
}

export interface PurchaseTicketsParams {
  eventId: string;
  tierId: string;
  quantity: number;
  unitPriceUsdc: number;
  recipientAddress?: string | null;
  options?: PurchaseOptionsInput;
  onProgress?: (event: PurchaseProgressEvent) => void;
  pollIntervalMs?: number;
  pollMaxAttempts?: number;
}

export interface PurchaseTicketsResult {
  paymentId: string;
  approvalTxHash: string;
  paymentTxHash: string;
  buyerPublicKey: string;
  totalPaidUsdc: number;
  /** Raw 32-byte check-in secret; caller should persist it locally (e.g. SecureStore) for QR generation. */
  checkInSecret: Uint8Array;
}

/**
 * Runs the full two-transaction purchase flow:
 *   1. Load the buyer's stored secret key.
 *   2. Approve the ticket_payment contract to pull the total USDC cost.
 *   3. Wait for the approval to confirm on-ledger.
 *   4. Call `process_payment` for the requested tier/quantity.
 *   5. Wait for that to confirm on-ledger.
 *
 * `onProgress` is invoked at each stage transition so the UI can render a
 * step-by-step status list.
 */
export async function purchaseTickets(
  params: PurchaseTicketsParams
): Promise<PurchaseTicketsResult> {
  const {
    eventId,
    tierId,
    quantity,
    unitPriceUsdc,
    recipientAddress,
    options,
    onProgress,
    pollIntervalMs,
    pollMaxAttempts,
  } = params;

  if (quantity <= 0) {
    throw new Error('Quantity must be at least 1.');
  }
  if (unitPriceUsdc < 0) {
    throw new Error('Unit price cannot be negative.');
  }

  const emit = (stage: PurchaseProgressStage) => onProgress?.({ stage });

  emit('loading-wallet');
  const secretKey = await StellarWalletManager.getSecretKey();
  if (!secretKey) {
    throw new Error(
      'No Stellar wallet found on this device. Import or create a wallet before checking out.'
    );
  }
  const buyerKeypair = Keypair.fromSecret(secretKey);
  const totalAmountStroops = usdcToStroops(unitPriceUsdc * quantity);

  emit('building-approval');
  emit('signing-approval');
  emit('submitting-approval');
  const approval = await submitUsdcApproval({ buyerKeypair, totalAmountStroops });

  emit('confirming-approval');
  await pollTransactionUntilFinal(approval.hash, {
    intervalMs: pollIntervalMs,
    maxAttempts: pollMaxAttempts,
    onAttempt: (attempt, maxAttempts) =>
      onProgress?.({ stage: 'confirming-approval', attempt, maxAttempts }),
  });

  const checkInSecret = generatePurchaseSecret();
  const paymentId = generatePaymentId(eventId, buyerKeypair.publicKey());

  emit('building-payment');
  emit('signing-payment');
  emit('submitting-payment');
  const payment = await submitProcessPayment({
    buyerKeypair,
    eventId,
    tierId,
    quantity,
    unitPriceUsdc,
    recipientAddress,
    paymentId,
    validationHash: checkInSecret.hash,
    options,
  });

  emit('confirming-payment');
  await pollTransactionUntilFinal(payment.hash, {
    intervalMs: pollIntervalMs,
    maxAttempts: pollMaxAttempts,
    onAttempt: (attempt, maxAttempts) =>
      onProgress?.({ stage: 'confirming-payment', attempt, maxAttempts }),
  });

  return {
    paymentId: payment.returnedPaymentId ?? paymentId,
    approvalTxHash: approval.hash,
    paymentTxHash: payment.hash,
    buyerPublicKey: buyerKeypair.publicKey(),
    totalPaidUsdc: unitPriceUsdc * quantity,
    checkInSecret: checkInSecret.secretBytes,
  };
}
