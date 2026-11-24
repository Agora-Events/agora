import { useCallback, useEffect, useState } from 'react';
import {
  CurrencyCode,
  DEFAULT_CURRENCY,
  ExchangeRateResult,
  FALLBACK_RATES,
  convertFromUsd,
  fetchExchangeRates,
  formatCurrency,
  loadCurrencyPreference,
  saveCurrencyPreference,
} from '@/lib/currency';

// Keeps every mounted hook instance in sync when one screen changes the preference.
const listeners = new Set<(currency: CurrencyCode) => void>();

export function useCurrencyPreference() {
  const [currency, setCurrencyState] = useState<CurrencyCode>(DEFAULT_CURRENCY);
  const [exchange, setExchange] = useState<ExchangeRateResult>({
    rates: FALLBACK_RATES,
    source: 'fallback',
  });
  const [loading, setLoading] = useState(true);

  const refreshRates = useCallback(async () => {
    setExchange(await fetchExchangeRates());
  }, []);

  useEffect(() => {
    let active = true;
    let changedBeforeLoad = false;
    const listener = (next: CurrencyCode) => {
      changedBeforeLoad = true;
      setCurrencyState(next);
    };
    listeners.add(listener);

    Promise.all([loadCurrencyPreference(), fetchExchangeRates()]).then(([stored, result]) => {
      if (!active) return;
      // Don't let a slow storage read clobber a choice the user just made.
      if (!changedBeforeLoad) setCurrencyState(stored);
      setExchange(result);
      setLoading(false);
    });

    return () => {
      active = false;
      listeners.delete(listener);
    };
  }, []);

  const setCurrency = useCallback(async (next: CurrencyCode) => {
    listeners.forEach((listener) => listener(next));
    await saveCurrencyPreference(next);
  }, []);

  const formatFromUsd = useCallback(
    (amountUsd: number) => formatCurrency(convertFromUsd(amountUsd, currency, exchange.rates), currency),
    [currency, exchange.rates],
  );

  return {
    currency,
    setCurrency,
    rates: exchange.rates,
    ratesSource: exchange.source,
    loading,
    refreshRates,
    formatFromUsd,
  };
}
