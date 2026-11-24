import React from 'react';
import { StyleSheet, Text, TouchableOpacity, View } from 'react-native';
import Colors from '@/constants/Colors';
import { useCurrencyPreference } from '@/hooks/useCurrencyPreference';
import {
  CURRENCY_LABELS,
  SUPPORTED_CURRENCIES,
  convertFromUsd,
  formatCurrency,
} from '@/lib/currency';

/** Issue #1649 — pick the display currency (USD, EUR, XLM, NGN). */
export default function CurrencySelector() {
  const { currency, setCurrency, rates, ratesSource } = useCurrencyPreference();

  return (
    <View>
      {SUPPORTED_CURRENCIES.map((code) => {
        const selected = code === currency;
        return (
          <TouchableOpacity
            key={code}
            testID={`currency-option-${code}`}
            accessibilityRole="radio"
            accessibilityState={{ selected }}
            onPress={() => setCurrency(code)}
            style={[styles.option, selected && styles.optionSelected]}
          >
            <View>
              <Text style={styles.code}>{code}</Text>
              <Text style={styles.label}>{CURRENCY_LABELS[code]}</Text>
            </View>
            <Text style={styles.rate}>
              {formatCurrency(convertFromUsd(1, code, rates), code)}
            </Text>
          </TouchableOpacity>
        );
      })}
      <Text testID="currency-rates-source" style={styles.footnote}>
        {ratesSource === 'live'
          ? 'Rates per 1 USD, updated live.'
          : 'Offline: showing approximate rates per 1 USD.'}
      </Text>
    </View>
  );
}

const styles = StyleSheet.create({
  option: {
    flexDirection: 'row',
    justifyContent: 'space-between',
    alignItems: 'center',
    paddingVertical: 12,
    paddingHorizontal: 12,
    borderRadius: 8,
    borderWidth: 1,
    borderColor: Colors.dark.border,
    marginTop: 8,
  },
  optionSelected: {
    borderColor: Colors.primaryYellow,
  },
  code: {
    fontSize: 15,
    fontWeight: 'bold',
    color: Colors.primaryText,
  },
  label: {
    fontSize: 12,
    color: Colors.secondaryText,
  },
  rate: {
    fontSize: 13,
    color: Colors.primaryYellow,
    fontFamily: 'SpaceMono',
  },
  footnote: {
    fontSize: 12,
    color: Colors.secondaryText,
    marginTop: 12,
  },
});
