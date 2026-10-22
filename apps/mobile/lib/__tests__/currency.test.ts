import AsyncStorage from '@react-native-async-storage/async-storage';
import {
  CURRENCY_STORAGE_KEY,
  DEFAULT_CURRENCY,
  FALLBACK_RATES,
  SUPPORTED_CURRENCIES,
  convertFromUsd,
  fetchExchangeRates,
  formatCurrency,
  isSupportedCurrency,
  loadCurrencyPreference,
  saveCurrencyPreference,
} from '../currency';

/**
 * Issue #1649 — currency preference + conversion helpers.
 */

jest.mock('@react-native-async-storage/async-storage', () => {
  const store = new Map<string, string>();
  return {
    __esModule: true,
    default: {
      getItem: jest.fn(async (key: string) => store.get(key) ?? null),
      setItem: jest.fn(async (key: string, value: string) => {
        store.set(key, value);
      }),
      clear: jest.fn(async () => store.clear()),
    },
  };
});

function jsonResponse(status: number, body: unknown) {
  return { ok: status >= 200 && status < 300, status, json: async () => body };
}

beforeEach(async () => {
  await AsyncStorage.clear();
  jest.clearAllMocks();
  global.fetch = jest.fn();
});

describe('currency preference persistence', () => {
  it('supports exactly USD, EUR, XLM and NGN', () => {
    expect(SUPPORTED_CURRENCIES).toEqual(['USD', 'EUR', 'XLM', 'NGN']);
    expect(isSupportedCurrency('NGN')).toBe(true);
    expect(isSupportedCurrency('GBP')).toBe(false);
  });

  it('defaults to USD when nothing is stored', async () => {
    expect(DEFAULT_CURRENCY).toBe('USD');
    await expect(loadCurrencyPreference()).resolves.toBe('USD');
  });

  it.each(['USD', 'EUR', 'XLM', 'NGN'] as const)('persists %s to AsyncStorage', async (code) => {
    await saveCurrencyPreference(code);
    expect(AsyncStorage.setItem).toHaveBeenCalledWith(CURRENCY_STORAGE_KEY, code);
    await expect(loadCurrencyPreference()).resolves.toBe(code);
  });

  it('ignores an unsupported stored value', async () => {
    await AsyncStorage.setItem(CURRENCY_STORAGE_KEY, 'GBP');
    await expect(loadCurrencyPreference()).resolves.toBe('USD');
  });

  it('falls back to USD when AsyncStorage throws', async () => {
    (AsyncStorage.getItem as jest.Mock).mockRejectedValueOnce(new Error('disk'));
    await expect(loadCurrencyPreference()).resolves.toBe('USD');
  });
});

describe('fetchExchangeRates', () => {
  it('derives USD-based rates from the live XLM quote', async () => {
    (global.fetch as jest.Mock).mockResolvedValueOnce(
      jsonResponse(200, { stellar: { usd: 0.5, eur: 0.45, ngn: 800 } }),
    );

    const result = await fetchExchangeRates();

    expect(global.fetch).toHaveBeenCalledTimes(1);
    expect(result.source).toBe('live');
    expect(result.rates.USD).toBe(1);
    expect(result.rates.EUR).toBeCloseTo(0.9);
    expect(result.rates.XLM).toBeCloseTo(2);
    expect(result.rates.NGN).toBeCloseTo(1600);
  });

  it('returns fallback rates when the request fails', async () => {
    (global.fetch as jest.Mock).mockRejectedValueOnce(new Error('offline'));
    await expect(fetchExchangeRates()).resolves.toEqual({
      rates: FALLBACK_RATES,
      source: 'fallback',
    });
  });

  it('returns fallback rates on a non-OK response', async () => {
    (global.fetch as jest.Mock).mockResolvedValueOnce(jsonResponse(429, {}));
    await expect(fetchExchangeRates()).resolves.toMatchObject({ source: 'fallback' });
  });

  it('returns fallback rates when the payload is malformed', async () => {
    (global.fetch as jest.Mock).mockResolvedValueOnce(
      jsonResponse(200, { stellar: { usd: 0, eur: 'x' } }),
    );
    await expect(fetchExchangeRates()).resolves.toMatchObject({ source: 'fallback' });
  });
});

describe('convertFromUsd', () => {
  const rates = { USD: 1, EUR: 0.9, XLM: 2, NGN: 1600 };

  it.each([
    ['USD', 10, 10],
    ['EUR', 10, 9],
    ['XLM', 10, 20],
    ['NGN', 10, 16000],
  ] as const)('converts to %s', (code, amount, expected) => {
    expect(convertFromUsd(amount, code, rates)).toBeCloseTo(expected);
  });

  it('uses fallback rates when none are given', () => {
    expect(convertFromUsd(1, 'EUR')).toBe(FALLBACK_RATES.EUR);
  });

  it('handles zero and non-finite amounts', () => {
    expect(convertFromUsd(0, 'NGN', rates)).toBe(0);
    expect(convertFromUsd(Number.NaN, 'EUR', rates)).toBe(0);
  });
});

describe('formatCurrency', () => {
  it('formats fiat currencies with their symbol and 2 decimals', () => {
    expect(formatCurrency(1234.5, 'USD')).toBe('$1,234.50');
    expect(formatCurrency(9, 'EUR')).toBe('€9.00');
    expect(formatCurrency(16000, 'NGN')).toBe('₦16,000.00');
  });

  it('formats XLM with a suffix and up to 7 decimals', () => {
    expect(formatCurrency(20, 'XLM')).toBe('20 XLM');
    expect(formatCurrency(1.23456789, 'XLM')).toBe('1.2345679 XLM');
  });

  it('formats negative and non-finite values safely', () => {
    expect(formatCurrency(-5, 'USD')).toBe('-$5.00');
    expect(formatCurrency(Number.NaN, 'EUR')).toBe('€0.00');
  });
});
