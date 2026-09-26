/**
 * resaleContract.ts
 *
 * Soroban calls for the capped secondary market in `ticket_payment`
 * (`contract/contracts/ticket_payment/src/resale.rs`):
 *
 *   pub fn list_for_resale(payment_id: String, price_usdc: i128) -> ResaleListing
 *   pub fn cancel_resale_listing(payment_id: String)
 *   pub fn purchase_resale_ticket(payment_id: String, buyer: Address) -> ResaleListing
 *   pub fn get_max_resale_price(payment_id: String) -> i128
 *   pub fn get_resale_listing(payment_id: String) -> Option<ResaleListing>
 *
 * Buying is two signed transactions, the same approve-then-call shape as
 * primary checkout: `purchase_resale_ticket` pulls the price with
 * `transfer_from`, so the buyer must approve the contract first or the call
 * fails with `InsufficientAllowance` (error 12).
 *
 * Selling is one transaction. Listing moves no money — the price ceiling is
 * checked and the listing recorded, nothing more.
 */

import { Address, Contract, Keypair, nativeToScVal, scValToNative, xdr } from '@stellar/stellar-sdk';
import { StellarWalletManager } from './stellar';
import {
  buildAndSimulateContractCall,
  getLatestLedgerSequence,
  loadSorobanAccount,
  pollTransactionUntilFinal,
  signTransaction,
  submitTransaction,
} from './sorobanClient';
import {
  TICKET_PAYMENT_CONTRACT_ID,
  getUsdcTokenContractId,
  usdcToStroops,
} from './ticketPaymentContract';

/** Ledgers of validity granted to the resale allowance (~1 hour at 5s/ledger). */
const APPROVAL_LEDGER_WINDOW = 720;

/** Mirrors `resale::ResaleStatus`. */
export type ResaleStatus = 'Active' | 'Cancelled' | 'Sold';

/** Mirrors `resale::ResaleListing`, with i128 fields as bigint. */
export interface OnChainResaleListing {
  paymentId: string;
  eventId: string;
  seller: string;
  price: bigint;
  tokenAddress: string;
  maxPrice: bigint;
  royaltyBps: number;
  status: ResaleStatus;
  createdAt: bigint;
  updatedAt: bigint;
}

/**
 * Converts the contract's `ResaleListing` return value into the shape above.
 *
 * `scValToNative` renders a `#[contracttype]` struct as a snake_case object and
 * a fieldless enum variant as a single-element array (`['Active']`), so both
 * are normalised here rather than at every call site.
 */
function parseListing(raw: any): OnChainResaleListing {
  const status = Array.isArray(raw?.status) ? raw.status[0] : raw?.status;

  return {
    paymentId: raw.payment_id,
    eventId: raw.event_id,
    seller: raw.seller,
    price: BigInt(raw.price),
    tokenAddress: raw.token_address,
    maxPrice: BigInt(raw.max_price),
    royaltyBps: Number(raw.royalty_bps),
    status: status as ResaleStatus,
    createdAt: BigInt(raw.created_at),
    updatedAt: BigInt(raw.updated_at),
  };
}

/**
 * Simulates a contract call without submitting it, for read-only entry points.
 * Uses the caller's own account purely as a simulation source — no signature
 * is produced and nothing reaches the ledger.
 */
async function simulateRead(
  sourcePublicKey: string,
  method: string,
  args: xdr.ScVal[]
): Promise<any> {
  const account = await loadSorobanAccount(sourcePublicKey);
  const contract = new Contract(TICKET_PAYMENT_CONTRACT_ID);
  const { simulation } = await buildAndSimulateContractCall({
    sourceAccount: account,
    operation: contract.call(method, ...args),
  });

  const retval = simulation.result?.retval;
  return retval ? scValToNative(retval) : null;
}

async function loadKeypair(): Promise<Keypair> {
  const secretKey = await StellarWalletManager.getSecretKey();
  if (!secretKey) {
    throw new Error(
      'No Stellar wallet found on this device. Import or create a wallet before using the resale market.'
    );
  }
  return Keypair.fromSecret(secretKey);
}

// ── Reads ───────────────────────────────────────────────────────────────────

/**
 * Highest price this ticket may legally be listed at, in stroops.
 *
 * Worth reading before showing the listing form: it lets the UI show the
 * ceiling up front instead of letting the seller discover it by having a
 * transaction rejected.
 */
export async function fetchMaxResalePrice(paymentId: string): Promise<bigint> {
  const keypair = await loadKeypair();
  const result = await simulateRead(keypair.publicKey(), 'get_max_resale_price', [
    nativeToScVal(paymentId, { type: 'string' }),
  ]);
  return BigInt(result ?? 0);
}

/** Reads a listing from the ledger, or `null` if the ticket was never listed. */
export async function fetchOnChainListing(
  paymentId: string
): Promise<OnChainResaleListing | null> {
  const keypair = await loadKeypair();
  const result = await simulateRead(keypair.publicKey(), 'get_resale_listing', [
    nativeToScVal(paymentId, { type: 'string' }),
  ]);
  return result ? parseListing(result) : null;
}

// ── Listing ─────────────────────────────────────────────────────────────────

export interface ListForResaleResult {
  hash: string;
  listing: OnChainResaleListing;
}

/**
 * Lists a ticket at `priceUsdc`. Rejected on-chain with
 * `ResalePriceExceedsCap` (error 26) if the price is above the ceiling, so
 * callers should validate against `fetchMaxResalePrice` first for a better
 * error than a failed transaction.
 */
export async function submitListForResale(
  paymentId: string,
  priceUsdc: number
): Promise<ListForResaleResult> {
  const keypair = await loadKeypair();
  const account = await loadSorobanAccount(keypair.publicKey());

  const contract = new Contract(TICKET_PAYMENT_CONTRACT_ID);
  const operation = contract.call(
    'list_for_resale',
    nativeToScVal(paymentId, { type: 'string' }),
    nativeToScVal(usdcToStroops(priceUsdc), { type: 'i128' })
  );

  const { transaction, simulation } = await buildAndSimulateContractCall({
    sourceAccount: account,
    operation,
  });

  const signed = signTransaction(transaction, keypair);
  const { hash } = await submitTransaction(signed);
  await pollTransactionUntilFinal(hash);

  const retval = simulation.result?.retval;
  if (!retval) {
    throw new Error('The contract did not return the created listing.');
  }

  return { hash, listing: parseListing(scValToNative(retval)) };
}

/** Withdraws a listing. Only the seller who created it can cancel. */
export async function submitCancelResaleListing(paymentId: string): Promise<string> {
  const keypair = await loadKeypair();
  const account = await loadSorobanAccount(keypair.publicKey());

  const contract = new Contract(TICKET_PAYMENT_CONTRACT_ID);
  const { transaction } = await buildAndSimulateContractCall({
    sourceAccount: account,
    operation: contract.call(
      'cancel_resale_listing',
      nativeToScVal(paymentId, { type: 'string' })
    ),
  });

  const signed = signTransaction(transaction, keypair);
  const { hash } = await submitTransaction(signed);
  await pollTransactionUntilFinal(hash);
  return hash;
}

// ── Buying ──────────────────────────────────────────────────────────────────

export type ResalePurchaseStage =
  | 'loading-wallet'
  | 'approving'
  | 'confirming-approval'
  | 'buying'
  | 'confirming-purchase';

export interface PurchaseResaleParams {
  paymentId: string;
  /** Listed price in stroops, read from the listing rather than user input. */
  priceStroops: bigint;
  onProgress?: (stage: ResalePurchaseStage) => void;
}

export interface PurchaseResaleResult {
  approvalTxHash: string;
  purchaseTxHash: string;
  buyerPublicKey: string;
  listing: OnChainResaleListing;
}

/**
 * Runs the full buy flow: approve the listing price, wait for it to confirm,
 * then call `purchase_resale_ticket`.
 *
 * The contract settles payment, the organizer royalty and the ownership
 * transfer in that second transaction, so there is no state in which the buyer
 * has paid without receiving the ticket. What they still need afterwards is
 * the check-in secret, which arrives out-of-band via the key envelope — see
 * `marketplaceApi.fetchKeyEnvelope`.
 */
export async function purchaseResaleTicket(
  params: PurchaseResaleParams
): Promise<PurchaseResaleResult> {
  const { paymentId, priceStroops, onProgress } = params;
  const emit = (stage: ResalePurchaseStage) => onProgress?.(stage);

  emit('loading-wallet');
  const keypair = await loadKeypair();
  const buyerPublicKey = keypair.publicKey();

  emit('approving');
  const [account, latestLedger] = await Promise.all([
    loadSorobanAccount(buyerPublicKey),
    getLatestLedgerSequence(),
  ]);

  const tokenContract = new Contract(getUsdcTokenContractId());
  const approvalOperation = tokenContract.call(
    'approve',
    new Address(buyerPublicKey).toScVal(),
    new Address(TICKET_PAYMENT_CONTRACT_ID).toScVal(),
    nativeToScVal(priceStroops, { type: 'i128' }),
    nativeToScVal(latestLedger + APPROVAL_LEDGER_WINDOW, { type: 'u32' })
  );

  const approval = await buildAndSimulateContractCall({
    sourceAccount: account,
    operation: approvalOperation,
  });
  const approvalSubmission = await submitTransaction(
    signTransaction(approval.transaction, keypair)
  );

  emit('confirming-approval');
  await pollTransactionUntilFinal(approvalSubmission.hash);

  emit('buying');
  // Reload the account: its sequence number advanced with the approval.
  const purchaseAccount = await loadSorobanAccount(buyerPublicKey);
  const contract = new Contract(TICKET_PAYMENT_CONTRACT_ID);
  const purchase = await buildAndSimulateContractCall({
    sourceAccount: purchaseAccount,
    operation: contract.call(
      'purchase_resale_ticket',
      nativeToScVal(paymentId, { type: 'string' }),
      new Address(buyerPublicKey).toScVal()
    ),
  });

  const purchaseSubmission = await submitTransaction(
    signTransaction(purchase.transaction, keypair)
  );

  emit('confirming-purchase');
  await pollTransactionUntilFinal(purchaseSubmission.hash);

  const retval = purchase.simulation.result?.retval;
  if (!retval) {
    throw new Error('The contract did not return the settled listing.');
  }

  return {
    approvalTxHash: approvalSubmission.hash,
    purchaseTxHash: purchaseSubmission.hash,
    buyerPublicKey,
    listing: parseListing(scValToNative(retval)),
  };
}
