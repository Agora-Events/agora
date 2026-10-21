import React from 'react';
import { fireEvent, render, waitFor } from '@testing-library/react-native';
import CurrencySelector from '../CurrencySelector';
import {
  fetchExchangeRates,
  loadCurrencyPreference,
  saveCurrencyPreference,
} from '@/lib/currency';

/** Issue #1649 — currency preference selector. */

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

beforeEach(() => {
  jest.clearAllMocks();
  mockLoad.mockResolvedValue('USD');
  mockSave.mockResolvedValue(undefined);
  mockFetch.mockResolvedValue({ rates: { USD: 1, EUR: 0.9, XLM: 2, NGN: 1600 }, source: 'live' });
});

describe('CurrencySelector', () => {
  it('renders an option with its live rate for every supported currency', async () => {
    const { getByTestId, getByText } = render(<CurrencySelector />);

    await waitFor(() => expect(getByText('₦1,600.00')).toBeTruthy());
    for (const code of ['USD', 'EUR', 'XLM', 'NGN']) {
      expect(getByTestId(`currency-option-${code}`)).toBeTruthy();
    }
    expect(getByText('2 XLM')).toBeTruthy();
    expect(getByTestId('currency-rates-source')).toHaveTextContent('Rates per 1 USD, updated live.');
  });

  it('marks the stored preference as selected', async () => {
    mockLoad.mockResolvedValue('XLM');
    const { getByTestId } = render(<CurrencySelector />);

    await waitFor(() =>
      expect(getByTestId('currency-option-XLM').props.accessibilityState.selected).toBe(true),
    );
    expect(getByTestId('currency-option-USD').props.accessibilityState.selected).toBe(false);
  });

  it('persists and selects a currency when pressed', async () => {
    const { getByTestId } = render(<CurrencySelector />);
    await waitFor(() => expect(mockFetch).toHaveBeenCalled());

    fireEvent.press(getByTestId('currency-option-EUR'));

    await waitFor(() => expect(mockSave).toHaveBeenCalledWith('EUR'));
    expect(getByTestId('currency-option-EUR').props.accessibilityState.selected).toBe(true);
  });

  it('flags approximate rates when offline', async () => {
    const actual = jest.requireActual('@/lib/currency');
    mockFetch.mockResolvedValue({ rates: actual.FALLBACK_RATES, source: 'fallback' });
    const { getByTestId } = render(<CurrencySelector />);

    await waitFor(() =>
      expect(getByTestId('currency-rates-source')).toHaveTextContent(
        'Offline: showing approximate rates per 1 USD.',
      ),
    );
  });
});
