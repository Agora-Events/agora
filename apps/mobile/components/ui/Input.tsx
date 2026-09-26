import React, { useState } from 'react';
import {
  View,
  TextInput,
  Text,
  StyleSheet,
  TextInputProps,
  StyleProp,
  ViewStyle,
} from 'react-native';
import Colors from '../../constants/Colors';
import { useTheme } from '../../hooks/useTheme';

interface InputProps extends TextInputProps {
  label?: string;
  error?: string;
  containerStyle?: StyleProp<ViewStyle>;
}

export const Input: React.FC<InputProps> = ({
  label,
  error,
  containerStyle,
  onFocus,
  onBlur,
  value = '',
  maxLength,
  ...props
}) => {
  const [isFocused, setIsFocused] = useState(false);
  const { theme, palette } = useTheme();

  const handleFocus = (e: any) => {
    setIsFocused(true);
    if (onFocus) onFocus(e);
  };

  const handleBlur = (e: any) => {
    setIsFocused(false);
    if (onBlur) onBlur(e);
  };

  // Show the counter only when maxLength is explicitly set.
  const showCounter = maxLength !== undefined;
  const currentLength = (value as string).length;
  // Warn when within the last 10% of the limit (minimum 1 char threshold).
  const warningThreshold = Math.max(1, Math.floor(maxLength! * 0.9));
  const isNearLimit = showCounter && currentLength >= warningThreshold;
  const isAtLimit = showCounter && currentLength >= maxLength!;

  const counterColor = isAtLimit
    ? Colors.accentRed
    : isNearLimit
    ? theme.counterWarning
    : theme.icon;

  return (
    <View style={[styles.container, containerStyle]}>
      {label && <Text style={[styles.label, { color: theme.text }]}>{label}</Text>}
      <View
        style={[
          styles.inputContainer,
          { backgroundColor: theme.cardBackground, borderColor: theme.border },
          isFocused && { borderColor: palette.primaryYellow },
          !!error && { borderColor: Colors.accentRed },
        ]}
      >
        <TextInput
          placeholderTextColor={Colors.secondaryText}
          style={[styles.input, { color: theme.text }]}
          onFocus={handleFocus}
          onBlur={handleBlur}
          value={value}
          maxLength={maxLength}
          {...props}
        />
      </View>

      {/* Counter and/or error row */}
      <View style={styles.footer}>
        {error ? (
          <Text style={[styles.errorText, { color: Colors.accentRed }]}>{error}</Text>
        ) : (
          // Reserve space so layout doesn't jump when error appears/disappears
          <View />
        )}
        {showCounter && (
          <Text style={[styles.counter, { color: counterColor }]}>
            {currentLength}/{maxLength}
          </Text>
        )}
      </View>
    </View>
  );
};

const styles = StyleSheet.create({
  container: {
    marginBottom: 16,
    width: '100%',
  },
  label: {
    fontSize: 14,
    fontWeight: '500',
    marginBottom: 6,
  },
  inputContainer: {
    borderWidth: 1,
    borderRadius: 8,
    height: 50,
    justifyContent: 'center',
    paddingHorizontal: 12,
  },
  input: {
    fontSize: 16,
    height: '100%',
  },
  footer: {
    flexDirection: 'row',
    justifyContent: 'space-between',
    alignItems: 'center',
    marginTop: 4,
    minHeight: 16,
  },
  errorText: {
    fontSize: 12,
    flex: 1,
  },
  counter: {
    fontSize: 11,
    marginLeft: 8,
    flexShrink: 0,
  },
});

export default Input;
