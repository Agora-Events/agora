import AsyncStorage from '@react-native-async-storage/async-storage';

/**
 * Display-currency preference and USD-based conversion (issue #1649).
 *
 * Prices are stored and charged in USD(C); this module only converts them for
 * display. Live rates come from CoinGecko's XLM quote in USD/EUR/NGN, which
 * yields every USD→X rate from a single keyless request. When offline or the
 * request fails, the approximate `FALLBACK_RATES` below are used instead.
 */

export const SUPPORTED_CURRENCIES = ['USD', 'EUR', 'XLM', 'NGN'] as const;
export type CurrencyCode = (typeof SUPPORTED_CURRENCIES)[number];

export const DEFAULT_CURRENCY: CurrencyCode = 'USD';
export const CURRENCY_STORAGE_KEY = '@agora/currency_preference';

/** Units of each currency per 1 USD. */
export type ExchangeRates = Record<CurrencyCode, number>;

/** Approximate offline rates; only used when live rates are unavailable. */
export const FALLBACK_RATES: ExchangeRates = {
  USD: 1,
  EUR: 0.92,
  XLM: 3.5,
  NGN: 1550,
};

export const EXCHANGE_RATE_URL =
  'https://api.coingecko.com/api/v3/simple/price?ids=stellar&vs_currencies=usd,eur,ngn';

export const CURRENCY_LABELS: Record<CurrencyCode, string> = {
  USD: 'US Dollar',
  EUR: 'Euro',
  XLM: 'Stellar Lumens',
  NGN: 'Nigerian Naira',
};

const CURRENCY_SYMBOLS: Record<Exclude<CurrencyCode, 'XLM'>, string> = {
  USD: '$',
  EUR: '€',
  NGN: '₦',
};

export function isSupportedCurrency(value: unknown): value is CurrencyCode {
  return typeof value === 'string' && (SUPPORTED_CURRENCIES as readonly string[]).includes(value);
}

export async function loadCurrencyPreference(): Promise<CurrencyCode> {
  try {
    const stored = await AsyncStorage.getItem(CURRENCY_STORAGE_KEY);
    return isSupportedCurrency(stored) ? stored : DEFAULT_CURRENCY;
  } catch {
    return DEFAULT_CURRENCY;
  }
}

export async function saveCurrencyPreference(currency: CurrencyCode): Promise<void> {
  await AsyncStorage.setItem(CURRENCY_STORAGE_KEY, currency);
}

export interface ExchangeRateResult {
  rates: ExchangeRates;
  source: 'live' | 'fallback';
}

export async function fetchExchangeRates(): Promise<ExchangeRateResult> {
  try {
    const response = await fetch(EXCHANGE_RATE_URL);
    if (!response.ok) throw new Error(`Rate request failed: ${response.status}`);

    const body = await response.json();
    const { usd, eur, ngn } = body?.stellar ?? {};
    if (![usd, eur, ngn].every(isPositiveNumber)) throw new Error('Malformed rate payload');

    return {
      rates: { USD: 1, EUR: eur / usd, XLM: 1 / usd, NGN: ngn / usd },
      source: 'live',
    };
  } catch {
    return { rates: FALLBACK_RATES, source: 'fallback' };
  }
}

export function convertFromUsd(
  amountUsd: number,
  currency: CurrencyCode,
  rates: ExchangeRates = FALLBACK_RATES,
): number {
  if (!Number.isFinite(amountUsd)) return 0;
  return amountUsd * rates[currency];
}

export function formatCurrency(amount: number, currency: CurrencyCode): string {
  const value = Number.isFinite(amount) ? amount : 0;
  const sign = value < 0 ? '-' : '';
  const abs = Math.abs(value);

  if (currency === 'XLM') {
    // Stellar amounts have 7 decimal places; trim trailing zeros.
    const [whole, fraction = ''] = abs.toFixed(7).split('.');
    const trimmed = fraction.replace(/0+$/, '');
    return `${sign}${groupThousands(whole)}${trimmed ? `.${trimmed}` : ''} XLM`;
  }

  const [whole, fraction] = abs.toFixed(2).split('.');
  return `${sign}${CURRENCY_SYMBOLS[currency]}${groupThousands(whole)}.${fraction}`;
}

function groupThousands(digits: string): string {
  return digits.replace(/\B(?=(\d{3})+(?!\d))/g, ',');
}

function isPositiveNumber(value: unknown): value is number {
  return typeof value === 'number' && Number.isFinite(value) && value > 0;
}
