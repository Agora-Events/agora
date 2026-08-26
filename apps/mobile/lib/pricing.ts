/**
 * Client-side order-total math for the checkout summary screen.
 *
 * Important: on `ticket_payment`'s `process_payment`, the platform fee is
 * NOT added on top of the ticket price — the buyer transfers exactly
 * `amount * quantity`, and the fee is carved out of that transfer before the
 * organizer's share is credited (see `total_organizer_amount = effective_total
 * - total_platform_fee` in `contract/contracts/ticket_payment/src/contract.rs`).
 * So `totalUsdc` below always equals `subtotalUsdc` — the fee line is shown
 * purely for transparency about how the organizer's payout is split.
 *
 * The *actual* fee bps is computed on-chain from the event's
 * `platform_fee_percent` / `custom_fee_bps`, and can be 0 for organizers with
 * a Pro subscription. The mobile client has no read access to that value
 * before submitting the transaction, so this is a best-effort estimate shown
 * as "Est. Platform Fee (from organizer payout)" — not a charge to the buyer.
 */

export const DEFAULT_PLATFORM_FEE_BPS = 250; // 2.5%, matches the contract's common default fee.
const MAX_BPS = 10_000;

export interface OrderTotals {
  subtotalUsdc: number;
  estimatedPlatformFeeUsdc: number;
  /** What the buyer actually transfers — always equal to `subtotalUsdc`. */
  totalUsdc: number;
}

export function computeOrderTotals(params: {
  unitPriceUsdc: number;
  quantity: number;
  platformFeeBps?: number;
}): OrderTotals {
  const { unitPriceUsdc, quantity, platformFeeBps = DEFAULT_PLATFORM_FEE_BPS } = params;

  const subtotalUsdc = round2(unitPriceUsdc * quantity);
  const estimatedPlatformFeeUsdc = round2((subtotalUsdc * platformFeeBps) / MAX_BPS);

  return { subtotalUsdc, estimatedPlatformFeeUsdc, totalUsdc: subtotalUsdc };
}

function round2(value: number): number {
  return Math.round(value * 100) / 100;
}
