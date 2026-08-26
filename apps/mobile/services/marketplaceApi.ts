/**
 * marketplaceApi.ts
 *
 * Client for the secondary-market endpoints under `/api/v1/marketplace`
 * (`server/src/handlers/marketplace.rs`).
 *
 * Two distinct jobs live behind these endpoints:
 *
 *   1. **Discovery.** On-chain listings are keyed by `payment_id` and cannot be
 *      iterated, so the server keeps a browsable mirror. Nothing here is
 *      authoritative — the contract is. A listing that disagrees with chain
 *      state simply fails to settle.
 *   2. **Key handover.** The seller uploads the ticket's check-in secret
 *      sealed to the buyer's X25519 key. The payloads crossing this boundary
 *      are ciphertext produced by `resaleCrypto.ts`; the server cannot read
 *      them, and neither can anything in this file.
 *
 * Prices are carried as integer stroop strings/numbers end to end. Formatting
 * to decimal USDC happens only at the display edge.
 */

import { useAuthStore } from '@/hooks/useAuth';
import type { KeyEnvelope } from './resaleCrypto';

const API_BASE_URL = process.env.EXPO_PUBLIC_API_URL ?? 'http://localhost:8080';
const REQUEST_TIMEOUT_MS = 15_000;

export class MarketplaceApiError extends Error {
  readonly status: number | null;

  constructor(message: string, status: number | null = null) {
    super(message);
    this.name = 'MarketplaceApiError';
    this.status = status;
  }

  /** True when the server has nothing for us yet, as opposed to a real failure. */
  get isNotFound(): boolean {
    return this.status === 404;
  }
}

// ── Wire types ──────────────────────────────────────────────────────────────

export type ResaleListingStatus = 'active' | 'cancelled' | 'sold';

export interface ResaleListing {
  payment_id: string;
  event_id: string;
  seller_wallet: string;
  price_stroops: number;
  max_price_stroops: number;
  royalty_bps: number;
  status: ResaleListingStatus;
  buyer_wallet: string | null;
  listing_tx_hash: string | null;
  sale_tx_hash: string | null;
  sold_at: string | null;
  created_at: string;
  updated_at: string;
  /** Server-derived: organizer's cut of `price_stroops`. */
  royalty_stroops: number;
  /** Server-derived: `price_stroops - royalty_stroops`. */
  seller_proceeds_stroops: number;
  /** Server-derived: how much room is left under the cap. */
  headroom_stroops: number;
}

export interface ResaleOffer {
  id: string;
  payment_id: string;
  buyer_wallet: string;
  /** Base64 X25519 key this buyer's envelope must be sealed to. */
  buyer_public_key: string;
  offer_price_stroops: number;
  status: 'pending' | 'accepted' | 'withdrawn' | 'declined';
  created_at: string;
}

export interface StoredKeyEnvelope {
  payment_id: string;
  buyer_wallet: string;
  ephemeral_public_key: string;
  nonce: string;
  ciphertext: string;
  claimed_at: string | null;
  created_at: string;
}

// ── Transport ───────────────────────────────────────────────────────────────

/**
 * Issues an authenticated JSON request and unwraps the server's response
 * envelope.
 *
 * Successful reads come back as `{ success, data, message }` while creates
 * return the resource directly (201), so this normalises both to the resource.
 */
async function request<T>(
  path: string,
  init: { method: string; body?: unknown } = { method: 'GET' }
): Promise<T> {
  const controller = new AbortController();
  const timeout = setTimeout(() => controller.abort(), REQUEST_TIMEOUT_MS);

  try {
    const token = useAuthStore.getState().token;
    const response = await fetch(`${API_BASE_URL}/api/v1/marketplace${path}`, {
      method: init.method,
      headers: {
        'Content-Type': 'application/json',
        ...(token ? { Authorization: `Bearer ${token}` } : {}),
      },
      body: init.body === undefined ? undefined : JSON.stringify(init.body),
      signal: controller.signal,
    });

    let body: any = null;
    try {
      body = await response.json();
    } catch {
      body = null;
    }

    if (!response.ok) {
      throw new MarketplaceApiError(
        body?.message || `Marketplace request failed (${response.status}).`,
        response.status
      );
    }

    return (body?.data ?? body) as T;
  } catch (error) {
    if (error instanceof MarketplaceApiError) throw error;
    if ((error as any)?.name === 'AbortError') {
      throw new MarketplaceApiError('The marketplace request timed out.');
    }
    throw new MarketplaceApiError(
      error instanceof Error ? error.message : 'Marketplace request failed.'
    );
  } finally {
    clearTimeout(timeout);
  }
}

// ── Listings ────────────────────────────────────────────────────────────────

export interface CreateListingInput {
  paymentId: string;
  eventId: string;
  priceStroops: bigint;
  /** Cap read back from the `ResaleListing` the contract returned. */
  maxPriceStroops: bigint;
  royaltyBps: number;
  listingTxHash?: string | null;
}

/**
 * Publishes a listing that already exists on-chain. Call this only after the
 * `list_for_resale` transaction confirms — advertising a listing the contract
 * has not accepted would show buyers something that cannot be bought.
 */
export function publishListing(input: CreateListingInput): Promise<ResaleListing> {
  return request<ResaleListing>('/listings', {
    method: 'POST',
    body: {
      payment_id: input.paymentId,
      event_id: input.eventId,
      price_stroops: Number(input.priceStroops),
      max_price_stroops: Number(input.maxPriceStroops),
      royalty_bps: input.royaltyBps,
      listing_tx_hash: input.listingTxHash ?? null,
    },
  });
}

export interface ListListingsFilter {
  eventId?: string;
  sellerWallet?: string;
  status?: ResaleListingStatus;
  limit?: number;
  offset?: number;
}

export function fetchListings(filter: ListListingsFilter = {}): Promise<ResaleListing[]> {
  const params = new URLSearchParams();
  if (filter.eventId) params.set('event_id', filter.eventId);
  if (filter.sellerWallet) params.set('seller_wallet', filter.sellerWallet);
  if (filter.status) params.set('status', filter.status);
  if (filter.limit != null) params.set('limit', String(filter.limit));
  if (filter.offset != null) params.set('offset', String(filter.offset));

  const query = params.toString();
  return request<ResaleListing[]>(`/listings${query ? `?${query}` : ''}`);
}

export function fetchListing(paymentId: string): Promise<ResaleListing> {
  return request<ResaleListing>(`/listings/${encodeURIComponent(paymentId)}`);
}

export function withdrawListing(paymentId: string): Promise<string> {
  return request<string>(`/listings/${encodeURIComponent(paymentId)}`, {
    method: 'DELETE',
  });
}

// ── Offers ──────────────────────────────────────────────────────────────────

/**
 * Registers interest in a listing and publishes the X25519 key the seller
 * should seal the ticket secret to. Re-calling updates the standing offer,
 * which is how a buyer rotates their encryption key.
 */
export function submitOffer(
  paymentId: string,
  buyerPublicKey: string,
  offerPriceStroops: bigint
): Promise<ResaleOffer> {
  return request<ResaleOffer>(`/listings/${encodeURIComponent(paymentId)}/offers`, {
    method: 'POST',
    body: {
      buyer_public_key: buyerPublicKey,
      offer_price_stroops: Number(offerPriceStroops),
    },
  });
}

/** Seller-only: the offer book for one of your listings, best offer first. */
export function fetchOffers(paymentId: string): Promise<ResaleOffer[]> {
  return request<ResaleOffer[]>(`/listings/${encodeURIComponent(paymentId)}/offers`);
}

// ── Key envelope ────────────────────────────────────────────────────────────

/**
 * Uploads the sealed ticket secret for `buyerWallet` and flips the listing to
 * sold. The envelope must come from `sealTicketSecret` — this function will
 * happily post anything, but only a correctly sealed box is openable by the
 * buyer.
 */
export function uploadKeyEnvelope(
  paymentId: string,
  buyerWallet: string,
  envelope: KeyEnvelope,
  saleTxHash?: string | null
): Promise<StoredKeyEnvelope> {
  return request<StoredKeyEnvelope>(
    `/listings/${encodeURIComponent(paymentId)}/key-envelope`,
    {
      method: 'POST',
      body: {
        buyer_wallet: buyerWallet,
        ephemeral_public_key: envelope.ephemeralPublicKey,
        nonce: envelope.nonce,
        ciphertext: envelope.ciphertext,
        sale_tx_hash: saleTxHash ?? null,
      },
    }
  );
}

/**
 * Fetches the envelope sealed for the authenticated buyer.
 *
 * Returns `null` rather than throwing when nothing is waiting yet — a buyer
 * polling right after settlement will legitimately see a 404 until the seller
 * uploads.
 */
export async function fetchKeyEnvelope(
  paymentId: string
): Promise<StoredKeyEnvelope | null> {
  try {
    return await request<StoredKeyEnvelope>(
      `/listings/${encodeURIComponent(paymentId)}/key-envelope`
    );
  } catch (error) {
    if (error instanceof MarketplaceApiError && error.isNotFound) return null;
    throw error;
  }
}

/** Converts a stored envelope back into the shape `openTicketSecret` expects. */
export function toKeyEnvelope(stored: StoredKeyEnvelope): KeyEnvelope {
  return {
    ephemeralPublicKey: stored.ephemeral_public_key,
    nonce: stored.nonce,
    ciphertext: stored.ciphertext,
  };
}

// ── Push registration ───────────────────────────────────────────────────────

/** Registers this device so the seller is alerted when their ticket sells. */
export function registerPushToken(
  token: string,
  platform: 'ios' | 'android' | 'web'
): Promise<string> {
  return request<string>('/push-token', {
    method: 'POST',
    body: { token, platform },
  });
}
