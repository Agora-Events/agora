import { act, renderHook, waitFor } from '@testing-library/react-native';
import { useCurrencyPreference } from '../useCurrencyPreference';
import {
  fetchExchangeRates,
  loadCurrencyPreference,
  saveCurrencyPreference,
} from '@/lib/currency';

/**
 * Issue #1649 — useCurrencyPreference.
 *
 * Storage and rate fetching are mocked; conversion/formatting use the real
 * helpers so the hook's `formatFromUsd` output is checked end to end.
 */

jest.mock('@react-native-async-storage/async-storage', () => ({
  __esModule: true,
  default: { getItem: jest.fn(), setItem: jest.fn() },
}));

jest.mock('@/lib/currency', () => ({
  ...jest.requireActual('@/lib/currency'),
  loadCurrencyPreference: jest.fn(),
  saveCurrencyPreference: jest.fn(),
  fetchExchangeRates: jest.fn(),
}));

const mockLoad = loadCurrencyPreference as jest.MockedFunction<typeof loadCurrencyPreference>;
const mockSave = saveCurrencyPreference as jest.MockedFunction<typeof saveCurrencyPreference>;
const mockFetch = fetchExchangeRates as jest.MockedFunction<typeof fetchExchangeRates>;

const liveRates = { USD: 1, EUR: 0.9, XLM: 2, NGN: 1600 };

beforeEach(() => {
  jest.clearAllMocks();
  mockLoad.mockResolvedValue('USD');
  mockSave.mockResolvedValue(undefined);
  mockFetch.mockResolvedValue({ rates: liveRates, source: 'live' });
});

it('restores the stored preference and live rates on mount', async () => {
  mockLoad.mockResolvedValue('NGN');
  const { result } = renderHook(() => useCurrencyPreference());

  await waitFor(() => expect(result.current.loading).toBe(false));

  expect(result.current.currency).toBe('NGN');
  expect(result.current.ratesSource).toBe('live');
  expect(result.current.formatFromUsd(10)).toBe('₦16,000.00');
});

it('switches currency, persists it, and syncs other instances', async () => {
  const first = renderHook(() => useCurrencyPreference());
  const second = renderHook(() => useCurrencyPreference());
  await waitFor(() => expect(first.result.current.loading).toBe(false));
  await waitFor(() => expect(second.result.current.loading).toBe(false));

  await act(async () => {
    await first.result.current.setCurrency('EUR');
  });

  expect(mockSave).toHaveBeenCalledWith('EUR');
  expect(first.result.current.currency).toBe('EUR');
  expect(second.result.current.currency).toBe('EUR');
  expect(second.result.current.formatFromUsd(10)).toBe('€9.00');

  await act(async () => {
    await first.result.current.setCurrency('XLM');
  });
  expect(first.result.current.formatFromUsd(10)).toBe('20 XLM');
});

it('exposes fallback rates when live rates are unavailable', async () => {
  const actual = jest.requireActual('@/lib/currency');
  mockFetch.mockResolvedValue({ rates: actual.FALLBACK_RATES, source: 'fallback' });

  const { result } = renderHook(() => useCurrencyPreference());
  await waitFor(() => expect(result.current.loading).toBe(false));

  expect(result.current.ratesSource).toBe('fallback');
  expect(result.current.rates).toEqual(actual.FALLBACK_RATES);
});
