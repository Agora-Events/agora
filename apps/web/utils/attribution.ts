export interface CheckoutAttribution {
  utmSource?: string;
  utmMedium?: string;
  utmCampaign?: string;
}

const STORAGE_KEY = "agora_checkout_attribution";
const MAX_UTM_LENGTH = 255;

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
    localStorage.setItem(STORAGE_KEY, JSON.stringify(attribution));
  } catch {
    return attribution;
  }
  return attribution;
}

export function getCheckoutAttribution(): CheckoutAttribution | undefined {
  try {
    const stored = JSON.parse(localStorage.getItem(STORAGE_KEY) ?? "null") as CheckoutAttribution | null;
    if (!stored || typeof stored !== "object") return undefined;
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
