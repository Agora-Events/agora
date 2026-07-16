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
  ...props
}) => {
  const [isFocused, setIsFocused] = useState(false);

  const handleFocus = (e: any) => {
    setIsFocused(true);
    if (onFocus) onFocus(e);
  };

  const handleBlur = (e: any) => {
    setIsFocused(false);
    if (onBlur) onBlur(e);
  };

  return (
    <View style={[styles.container, containerStyle]}>
      {label && <Text style={styles.label}>{label}</Text>}
      <View
        style={[
          styles.inputContainer,
          isFocused && styles.focused,
          !!error && styles.errorBorder,
        ]}
      >
        <TextInput
          placeholderTextColor={Colors.secondaryText}
          style={styles.input}
          onFocus={handleFocus}
          onBlur={handleBlur}
          {...props}
        />
      </View>
      {error ? <Text style={styles.errorText}>{error}</Text> : null}
    </View>
  );
};

const styles = StyleSheet.create({
  container: {
    marginBottom: 16,
    width: '100%',
  },
  label: {
    color: Colors.primaryText,
    fontSize: 14,
    fontWeight: '500',
    marginBottom: 6,
  },
  inputContainer: {
    backgroundColor: '#1E1E20',
    borderWidth: 1,
    borderColor: '#2C2C2E',
    borderRadius: 8,
    height: 50,
    justifyContent: 'center',
    paddingHorizontal: 12,
  },
  input: {
    color: Colors.primaryText,
    fontSize: 16,
    height: '100%',
  },
  focused: {
    borderColor: Colors.primaryYellow,
  },
  errorBorder: {
    borderColor: Colors.accentRed,
  },
  errorText: {
    color: Colors.accentRed,
    fontSize: 12,
    marginTop: 4,
  },
});

export default Input;
