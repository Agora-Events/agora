export interface CheckoutAttribution {
  utmSource?: string;
  utmMedium?: string;
  utmCampaign?: string;
}

const STORAGE_KEY = "agora_checkout_attribution";
const MAX_UTM_LENGTH = 255;

/**
 * How long a captured campaign keeps getting credit for checkouts.
 * Stored attribution older than this is ignored and removed.
 */
export const ATTRIBUTION_MAX_AGE_MS = 30 * 24 * 60 * 60 * 1000;

interface StoredAttribution extends CheckoutAttribution {
  /** `Date.now()` when the UTM values were captured. */
  capturedAt?: number;
}

function removeStored(): void {
  try {
    localStorage.removeItem(STORAGE_KEY);
  } catch {
    // Storage unavailable (e.g. private mode); nothing to clean up.
  }
}

function clean(value: string | null): string | undefined {
  const trimmed = value?.trim();
  return trimmed ? trimmed.slice(0, MAX_UTM_LENGTH) : undefined;
}

export function captureAttribution(searchParams: URLSearchParams): CheckoutAttribution | null {
  const attribution: CheckoutAttribution = {
    utmSource: clean(searchParams.get("utm_source")),
    utmMedium: clean(searchParams.get("utm_medium")),
    utmCampaign: clean(searchParams.get("utm_campaign")),
  };

  if (!attribution.utmSource && !attribution.utmMedium && !attribution.utmCampaign) {
    return null;
  }

  try {
    const stored: StoredAttribution = { ...attribution, capturedAt: Date.now() };
    localStorage.setItem(STORAGE_KEY, JSON.stringify(stored));
  } catch {
    return attribution;
  }
  return attribution;
}

/**
 * Attribution to send with a checkout, or `undefined` when nothing usable is stored.
 * Values older than {@link ATTRIBUTION_MAX_AGE_MS}, or stored without `capturedAt`,
 * are ignored and removed.
 */
export function getCheckoutAttribution(): CheckoutAttribution | undefined {
  try {
    const stored = JSON.parse(localStorage.getItem(STORAGE_KEY) ?? "null") as StoredAttribution | null;
    if (!stored || typeof stored !== "object") return undefined;
    // Entries written before `capturedAt` existed have no age, so they are dropped.
    // A timestamp in the future cannot be trusted either.
    const capturedAt = stored.capturedAt;
    const age = typeof capturedAt === "number" && Number.isFinite(capturedAt) ? Date.now() - capturedAt : NaN;
    if (!(age >= 0 && age <= ATTRIBUTION_MAX_AGE_MS)) {
      removeStored();
      return undefined;
    }
    const attribution = {
      utmSource: typeof stored.utmSource === "string" ? clean(stored.utmSource) : undefined,
      utmMedium: typeof stored.utmMedium === "string" ? clean(stored.utmMedium) : undefined,
      utmCampaign: typeof stored.utmCampaign === "string" ? clean(stored.utmCampaign) : undefined,
    };
    return attribution.utmSource || attribution.utmMedium || attribution.utmCampaign
      ? attribution
      : undefined;
  } catch {
    return undefined;
  }
}
