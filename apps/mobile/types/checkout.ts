/**
 * Shared types for the ticket checkout flow (issue #1005).
 */

export interface TicketTierOption {
  id: string;
  name: string;
  description?: string;
  /** Decimal USDC price for a single ticket, e.g. `25` or `0` for free tiers. */
  priceUsdc: number;
  /** Tickets remaining in this tier, if known. */
  remaining?: number;
}

export interface CheckoutEventSummary {
  id: string;
  title: string;
  dateLabel: string;
  venueLabel: string;
}

/**
 * Ordered list of steps rendered by `CheckoutProgressModal`. Order matches
 * the sequence `purchaseTickets()` in `services/ticketPaymentContract.ts`
 * actually executes.
 */
export type CheckoutStepId =
  | 'build-approval'
  | 'sign-approval'
  | 'submit-approval'
  | 'confirm-approval'
  | 'build-payment'
  | 'sign-payment'
  | 'submit-payment'
  | 'confirm-payment'
  | 'record-purchase';

export type CheckoutStepStatus = 'pending' | 'active' | 'done' | 'error';

export interface CheckoutStep {
  id: CheckoutStepId;
  label: string;
  status: CheckoutStepStatus;
  /** Populated for the currently-confirming step while polling the ledger. */
  detail?: string;
}

export const CHECKOUT_STEP_LABELS: Record<CheckoutStepId, string> = {
  'build-approval': 'Generating USDC approval transaction...',
  'sign-approval': 'Signing approval transaction...',
  'submit-approval': 'Submitting approval to Soroban RPC...',
  'confirm-approval': 'Waiting for approval confirmation...',
  'build-payment': 'Generating ticket purchase transaction...',
  'sign-payment': 'Signing purchase transaction...',
  'submit-payment': 'Submitting purchase to Soroban RPC...',
  'confirm-payment': 'Waiting for ledger confirmation...',
  'record-purchase': 'Recording your ticket...',
};

export const CHECKOUT_STEP_ORDER: CheckoutStepId[] = [
  'build-approval',
  'sign-approval',
  'submit-approval',
  'confirm-approval',
  'build-payment',
  'sign-payment',
  'submit-payment',
  'confirm-payment',
  'record-purchase',
];

export interface CheckoutReceipt {
  ticketId: string;
  paymentId: string;
  eventId: string;
  eventTitle: string;
  tierName: string;
  quantity: number;
  unitPriceUsdc: number;
  platformFeeUsdc: number;
  totalPaidUsdc: number;
  approvalTxHash: string;
  paymentTxHash: string;
  buyerPublicKey: string;
  completedAt: string; // ISO timestamp
}
