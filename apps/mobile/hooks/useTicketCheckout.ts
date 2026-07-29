import { useCallback, useMemo, useReducer, useRef } from 'react';
import * as SecureStore from 'expo-secure-store';
import {
  describePurchaseFailure,
  purchaseTickets,
  type PurchaseProgressStage,
} from '@/services/ticketPaymentContract';
import { recordTicketPurchase } from '@/services/paymentsApi';
import { computeOrderTotals } from '@/lib/pricing';
import {
  CHECKOUT_STEP_LABELS,
  CHECKOUT_STEP_ORDER,
  type CheckoutReceipt,
  type CheckoutStep,
  type CheckoutStepId,
} from '@/types/checkout';

/**
 * Maps a low-level progress stage reported by `purchaseTickets()` onto the
 * UI-facing step id it should mark active. `confirm-approval`/`confirm-payment`
 * additionally carry a `detail` string built from the poll attempt counters.
 */
function stageToStepId(stage: PurchaseProgressStage): CheckoutStepId {
  switch (stage) {
    case 'loading-wallet':
    case 'building-approval':
      return 'build-approval';
    case 'signing-approval':
      return 'sign-approval';
    case 'submitting-approval':
      return 'submit-approval';
    case 'confirming-approval':
      return 'confirm-approval';
    case 'building-payment':
      return 'build-payment';
    case 'signing-payment':
      return 'sign-payment';
    case 'submitting-payment':
      return 'submit-payment';
    case 'confirming-payment':
      return 'confirm-payment';
    default:
      return 'build-approval';
  }
}

// ── State machine ────────────────────────────────────────────────────────────

export type CheckoutPhase = 'idle' | 'in-progress' | 'success' | 'error';

interface CheckoutState {
  phase: CheckoutPhase;
  steps: CheckoutStep[];
  errorMessage: string | null;
  receipt: CheckoutReceipt | null;
}

type CheckoutAction =
  | { type: 'RESET' }
  | { type: 'START' }
  | { type: 'STEP_ACTIVE'; stepId: CheckoutStepId; detail?: string }
  | { type: 'STEP_DONE'; stepId: CheckoutStepId }
  | { type: 'FAILURE'; stepId: CheckoutStepId | null; message: string }
  | { type: 'SUCCESS'; receipt: CheckoutReceipt };

function buildInitialSteps(): CheckoutStep[] {
  return CHECKOUT_STEP_ORDER.map((id) => ({
    id,
    label: CHECKOUT_STEP_LABELS[id],
    status: 'pending' as const,
  }));
}

function checkoutReducer(state: CheckoutState, action: CheckoutAction): CheckoutState {
  switch (action.type) {
    case 'RESET':
      return { phase: 'idle', steps: buildInitialSteps(), errorMessage: null, receipt: null };

    case 'START':
      return { phase: 'in-progress', steps: buildInitialSteps(), errorMessage: null, receipt: null };

    case 'STEP_ACTIVE':
      return {
        ...state,
        steps: state.steps.map((step) =>
          step.id === action.stepId
            ? { ...step, status: 'active', detail: action.detail }
            : step.status === 'active'
              ? { ...step, status: 'done' } // a later step became active; the previous active one finished
              : step
        ),
      };

    case 'STEP_DONE':
      return {
        ...state,
        steps: state.steps.map((step) =>
          step.id === action.stepId ? { ...step, status: 'done', detail: undefined } : step
        ),
      };

    case 'FAILURE': {
      // A failure can land before any step goes active — the very first call
      // throwing, for instance. Without a fallback the modal would show an
      // error phase with every step still pending and no marker for where it
      // broke, so the first unfinished step is flagged instead.
      const hasExplicitTarget = state.steps.some(
        (step) =>
          (action.stepId != null && step.id === action.stepId) || step.status === 'active'
      );
      const firstUnfinishedIndex = state.steps.findIndex((step) => step.status !== 'done');

      return {
        ...state,
        phase: 'error',
        errorMessage: action.message,
        steps: state.steps.map((step, index) => {
          if (action.stepId && step.id === action.stepId) return { ...step, status: 'error' };
          if (step.status === 'active') return { ...step, status: 'error' };
          if (!hasExplicitTarget && index === firstUnfinishedIndex) {
            return { ...step, status: 'error' };
          }
          return step;
        }),
      };
    }

    case 'SUCCESS':
      return {
        ...state,
        phase: 'success',
        receipt: action.receipt,
        steps: state.steps.map((step) => ({ ...step, status: 'done' as const, detail: undefined })),
      };

    default:
      return state;
  }
}

// ── Hook ─────────────────────────────────────────────────────────────────────

export interface StartCheckoutParams {
  eventId: string;
  eventTitle: string;
  tierId: string;
  tierName: string;
  unitPriceUsdc: number;
  quantity: number;
  buyerPublicKey: string;
  recipientAddress?: string | null;
}

export interface UseTicketCheckoutResult {
  phase: CheckoutPhase;
  steps: CheckoutStep[];
  errorMessage: string | null;
  receipt: CheckoutReceipt | null;
  isSubmitting: boolean;
  startCheckout: (params: StartCheckoutParams) => Promise<void>;
  reset: () => void;
}

const CHECK_IN_SECRET_STORAGE_PREFIX = 'agora_ticket_checkin_secret_';

/**
 * Drives the full checkout flow described in issue #1005:
 *
 *   build → sign → submit → poll for ledger confirmation → record in Postgres
 *
 * UI components only need `steps` (for the progress list), `phase` /
 * `errorMessage` (for success/error UI), and `receipt` (for the completion
 * screen). All Soroban/backend orchestration lives in
 * `services/ticketPaymentContract.ts` and `services/paymentsApi.ts` — this
 * hook is purely the state machine gluing them to React.
 */
export function useTicketCheckout(): UseTicketCheckoutResult {
  const [state, dispatch] = useReducer(checkoutReducer, undefined, () => ({
    phase: 'idle' as const,
    steps: buildInitialSteps(),
    errorMessage: null,
    receipt: null,
  }));

  // Guards against a second concurrent checkout attempt (e.g. rage-tapping
  // the confirm button) racing the same hook instance.
  const inFlightRef = useRef(false);

  const startCheckout = useCallback(async (params: StartCheckoutParams) => {
    if (inFlightRef.current) return;
    inFlightRef.current = true;
    dispatch({ type: 'START' });

    try {
      const purchaseResult = await purchaseTickets({
        eventId: params.eventId,
        tierId: params.tierId,
        quantity: params.quantity,
        unitPriceUsdc: params.unitPriceUsdc,
        recipientAddress: params.recipientAddress,
        onProgress: (event) => {
          const stepId = stageToStepId(event.stage);
          const detail =
            event.attempt && event.maxAttempts
              ? `Attempt ${event.attempt} of ${event.maxAttempts}...`
              : undefined;
          dispatch({ type: 'STEP_ACTIVE', stepId, detail });
        },
      });

      // Best-effort local persistence of the check-in secret; a failure here
      // must not fail the whole checkout since the payment already confirmed
      // on-chain — the buyer can still view their ticket without a working
      // dynamic QR fallback.
      try {
        await SecureStore.setItemAsync(
          `${CHECK_IN_SECRET_STORAGE_PREFIX}${purchaseResult.paymentId}`,
          Buffer.from(purchaseResult.checkInSecret).toString('base64'),
          { keychainService: 'agora_stellar_wallet' }
        );
      } catch {
        // Ignore — see comment above.
      }

      dispatch({ type: 'STEP_DONE', stepId: 'confirm-payment' });
      dispatch({ type: 'STEP_ACTIVE', stepId: 'record-purchase' });

      const totals = computeOrderTotals({
        unitPriceUsdc: params.unitPriceUsdc,
        quantity: params.quantity,
      });

      let ticketId = purchaseResult.paymentId;
      try {
        const recorded = await recordTicketPurchase({
          eventId: params.eventId,
          tierId: params.tierId,
          quantity: params.quantity,
          buyerWallet: purchaseResult.buyerPublicKey,
          recipientWallet: params.recipientAddress ?? undefined,
          unitPriceUsdc: params.unitPriceUsdc,
          totalPaidUsdc: purchaseResult.totalPaidUsdc,
          paymentId: purchaseResult.paymentId,
          approvalTxHash: purchaseResult.approvalTxHash,
          paymentTxHash: purchaseResult.paymentTxHash,
        });
        ticketId = recorded.ticketId;
      } catch (_recordError) {
        // The purchase is fully confirmed on-chain at this point — losing the
        // backend write is a data-sync problem, not a failed purchase. Surface
        // a receipt anyway so the buyer isn't told their money vanished, but
        // note the sync issue in the step detail for support/debugging.
        dispatch({
          type: 'STEP_ACTIVE',
          stepId: 'record-purchase',
          detail: 'Payment confirmed on-chain, but the receipt could not be saved yet. It will sync automatically.',
        });
      }

      dispatch({
        type: 'SUCCESS',
        receipt: {
          ticketId,
          paymentId: purchaseResult.paymentId,
          eventId: params.eventId,
          eventTitle: params.eventTitle,
          tierName: params.tierName,
          quantity: params.quantity,
          unitPriceUsdc: params.unitPriceUsdc,
          platformFeeUsdc: totals.estimatedPlatformFeeUsdc,
          totalPaidUsdc: purchaseResult.totalPaidUsdc,
          approvalTxHash: purchaseResult.approvalTxHash,
          paymentTxHash: purchaseResult.paymentTxHash,
          buyerPublicKey: purchaseResult.buyerPublicKey,
          completedAt: new Date().toISOString(),
        },
      });
    } catch (error) {
      const message = describePurchaseFailure(error);
      const stepId = (error as any)?.stage ? stageFromErrorStage((error as any).stage) : null;
      dispatch({ type: 'FAILURE', stepId, message });
    } finally {
      inFlightRef.current = false;
    }
  }, []);

  const reset = useCallback(() => dispatch({ type: 'RESET' }), []);

  return useMemo(
    () => ({
      phase: state.phase,
      steps: state.steps,
      errorMessage: state.errorMessage,
      receipt: state.receipt,
      isSubmitting: state.phase === 'in-progress',
      startCheckout,
      reset,
    }),
    [state, startCheckout, reset]
  );
}

/** Best-effort mapping from a `SorobanTransactionError.stage` back to the step that failed. */
function stageFromErrorStage(stage: string): CheckoutStepId | null {
  switch (stage) {
    case 'load-account':
    case 'simulate':
      return 'build-approval';
    case 'sign':
      return 'sign-approval';
    case 'submit':
      return 'submit-approval';
    case 'confirm':
    case 'timeout':
      return 'confirm-approval';
    default:
      return null;
  }
}
