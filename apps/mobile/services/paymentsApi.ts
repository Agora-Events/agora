/**
 * paymentsApi.ts
 *
 * Records a confirmed on-chain ticket purchase in the backend's Postgres
 * database. Called only after the Soroban transaction has reached SUCCESS —
 * this endpoint never initiates a payment itself, it just persists the
 * receipt so the ticket shows up in the user's profile / organizer
 * dashboards. See `apps/web/public/openapi.yaml` (`POST /payments/ticket`).
 */

import { useAuthStore } from '@/hooks/useAuth';

const API_BASE_URL = process.env.EXPO_PUBLIC_API_URL ?? 'http://localhost:8080';

const RECORD_PAYMENT_TIMEOUT_MS = 15_000;
const RECORD_PAYMENT_MAX_RETRIES = 2;
const RECORD_PAYMENT_RETRY_DELAY_MS = 1500;

export interface RecordTicketPaymentRequest {
  eventId: string;
  tierId: string;
  quantity: number;
  buyerWallet: string;
  recipientWallet?: string;
  unitPriceUsdc: number;
  totalPaidUsdc: number;
  paymentId: string;
  approvalTxHash: string;
  paymentTxHash: string;
}

export interface RecordTicketPaymentResponse {
  ticketId: string;
  paymentId: string;
  status: string;
}

export class RecordPaymentError extends Error {
  readonly status: number | null;
  constructor(message: string, status: number | null = null) {
    super(message);
    this.name = 'RecordPaymentError';
    this.status = status;
  }
}

function sleep(ms: number): Promise<void> {
  return new Promise((resolve) => setTimeout(resolve, ms));
}

/**
 * POSTs the confirmed transaction hash to `/api/payments/ticket` so the
 * backend can verify it on-chain and persist the ticket row. Retries a
 * couple of times on network failure / 5xx — the money has already moved at
 * this point, so we should not give up on recording it after one blip.
 */
export async function recordTicketPurchase(
  payload: RecordTicketPaymentRequest
): Promise<RecordTicketPaymentResponse> {
  let lastError: unknown = null;

  for (let attempt = 0; attempt <= RECORD_PAYMENT_MAX_RETRIES; attempt++) {
    try {
      return await postTicketPayment(payload);
    } catch (error) {
      lastError = error;
      const isLastAttempt = attempt === RECORD_PAYMENT_MAX_RETRIES;
      const isRetryable = error instanceof RecordPaymentError ? isRetryableStatus(error.status) : true;
      if (isLastAttempt || !isRetryable) {
        throw error;
      }
      await sleep(RECORD_PAYMENT_RETRY_DELAY_MS * (attempt + 1));
    }
  }

  // Unreachable, but keeps TypeScript happy about the return type.
  throw lastError instanceof Error ? lastError : new RecordPaymentError('Failed to record payment.');
}

function isRetryableStatus(status: number | null): boolean {
  if (status == null) return true; // network error / timeout
  return status >= 500;
}

async function postTicketPayment(
  payload: RecordTicketPaymentRequest
): Promise<RecordTicketPaymentResponse> {
  const controller = new AbortController();
  const timeout = setTimeout(() => controller.abort(), RECORD_PAYMENT_TIMEOUT_MS);

  try {
    const token = useAuthStore.getState().token;
    const response = await fetch(`${API_BASE_URL}/api/payments/ticket`, {
      method: 'POST',
      headers: {
        'Content-Type': 'application/json',
        ...(token ? { Authorization: `Bearer ${token}` } : {}),
      },
      body: JSON.stringify(payload),
      signal: controller.signal,
    });

    let body: any = null;
    try {
      body = await response.json();
    } catch {
      body = null;
    }

    if (!response.ok) {
      throw new RecordPaymentError(
        body?.error || `Failed to record ticket purchase (${response.status}).`,
        response.status
      );
    }

    return {
      ticketId: body?.ticketId ?? body?.id ?? payload.paymentId,
      paymentId: body?.paymentId ?? payload.paymentId,
      status: body?.status ?? 'confirmed',
    };
  } catch (error) {
    if (error instanceof RecordPaymentError) throw error;
    if ((error as any)?.name === 'AbortError') {
      throw new RecordPaymentError('Timed out recording your ticket purchase. Your payment was still successful on-chain.');
    }
    throw new RecordPaymentError(
      error instanceof Error ? error.message : 'Failed to record ticket purchase.'
    );
  } finally {
    clearTimeout(timeout);
  }
}
