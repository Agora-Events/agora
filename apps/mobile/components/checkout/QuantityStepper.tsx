import React from 'react';
import { View, Text, Pressable, StyleSheet } from 'react-native';
import Colors from '@/constants/Colors';

interface QuantityStepperProps {
  value: number;
  onChange: (next: number) => void;
  min?: number;
  max?: number;
  disabled?: boolean;
}

export default function QuantityStepper({
  value,
  onChange,
  min = 1,
  max = 10,
  disabled = false,
}: QuantityStepperProps) {
  const canDecrement = !disabled && value > min;
  const canIncrement = !disabled && value < max;

  return (
    <View style={styles.row}>
      <Pressable
        testID="quantity-decrement"
        accessibilityRole="button"
        accessibilityLabel="Decrease ticket quantity"
        disabled={!canDecrement}
        onPress={() => onChange(Math.max(min, value - 1))}
        style={({ pressed }) => [
          styles.button,
          !canDecrement && styles.buttonDisabled,
          pressed && canDecrement && styles.buttonPressed,
        ]}
      >
        <Text style={[styles.buttonText, !canDecrement && styles.buttonTextDisabled]}>−</Text>
      </Pressable>

      <Text testID="quantity-value" style={styles.value}>{value}</Text>

      <Pressable
        testID="quantity-increment"
        accessibilityRole="button"
        accessibilityLabel="Increase ticket quantity"
        disabled={!canIncrement}
        onPress={() => onChange(Math.min(max, value + 1))}
        style={({ pressed }) => [
          styles.button,
          !canIncrement && styles.buttonDisabled,
          pressed && canIncrement && styles.buttonPressed,
        ]}
      >
        <Text style={[styles.buttonText, !canIncrement && styles.buttonTextDisabled]}>+</Text>
      </Pressable>
    </View>
  );
}

const styles = StyleSheet.create({
  row: {
    flexDirection: 'row',
    alignItems: 'center',
    gap: 16,
  },
  button: {
    width: 36,
    height: 36,
    borderRadius: 18,
    backgroundColor: '#2C2C2E',
    justifyContent: 'center',
    alignItems: 'center',
  },
  buttonDisabled: {
    opacity: 0.35,
  },
  buttonPressed: {
    opacity: 0.7,
  },
  buttonText: {
    color: Colors.primaryText,
    fontSize: 20,
    fontWeight: '600',
    lineHeight: 22,
  },
  buttonTextDisabled: {
    color: Colors.secondaryText,
  },
  value: {
    color: Colors.primaryText,
    fontSize: 18,
    fontWeight: '700',
    minWidth: 24,
    textAlign: 'center',
  },
});
