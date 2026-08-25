import React, { useState } from 'react';
import { View, Text, StyleSheet, SafeAreaView, KeyboardAvoidingView, Platform, ScrollView } from 'react-native';
import Colors from '@/constants/Colors';
import Input from '@/components/ui/Input';
import Button from '@/components/ui/Button';
import { useAuth } from '@/hooks/useAuth';
import ErrorBoundary from '@/components/ui/ErrorBoundary';

export default function AuthScreen() {
  const { login } = useAuth();
  
  const [email, setEmail] = useState('');
  const [password, setPassword] = useState('');
  const [emailError, setEmailError] = useState('');
  const [passwordError, setPasswordError] = useState('');
  const [loading, setLoading] = useState(false);

  const validate = () => {
    let isValid = true;
    
    // Email Validation
    if (!email) {
      setEmailError('Email is required');
      isValid = false;
    } else {
      const emailRegex = /^[^\s@]+@[^\s@]+\.[^\s@]+$/;
      if (!emailRegex.test(email)) {
        setEmailError('Invalid email format');
        isValid = false;
      } else {
        setEmailError('');
      }
    }

    // Password Validation
    if (!password) {
      setPasswordError('Password is required');
      isValid = false;
    } else if (password.length < 6) {
      setPasswordError('Password must be at least 6 characters');
      isValid = false;
    } else {
      setPasswordError('');
    }

    return isValid;
  };

  const handleLogin = () => {
    if (!validate()) return;

    setLoading(true);
    // Simulate API request
    setTimeout(() => {
      setLoading(false);
      login(email, 'Jane Doe', 'GDAGORA12345678901234567890123456789012345678901234567890XLM');
      // RootLayout will automatically redirect to discover tab due to the useEffect listener
    }, 1000);
  };

  const handleSocialLogin = (platform: 'Google' | 'Apple') => {
    setLoading(true);
    setTimeout(() => {
      setLoading(false);
      login(
        `${platform.toLowerCase()}user@agora.events`,
        `${platform} User`,
        `GDSOCIAL${platform.toUpperCase()}12345678901234567890XLM`
      );
    }, 1000);
  };

  return (
    <ErrorBoundary>
      <SafeAreaView style={styles.container}>
        <KeyboardAvoidingView
          behavior={Platform.OS === 'ios' ? 'padding' : 'height'}
          style={styles.keyboardView}
        >
          <ScrollView contentContainerStyle={styles.scrollContent} keyboardShouldPersistTaps="handled">
            <View style={styles.headerContainer}>
              <Text style={styles.logoText}>AGORA</Text>
              <Text style={styles.subtitleText}>Decentralized Event Ticketing</Text>
            </View>

            <View style={styles.formContainer}>
              <Input
                label="Email Address"
                value={email}
                onChangeText={(text) => {
                  setEmail(text);
                  if (emailError) setEmailError('');
                }}
                error={emailError}
                placeholder="Enter your email"
                keyboardType="email-address"
                autoCapitalize="none"
                autoCorrect={false}
              />

              <Input
                label="Password"
                value={password}
                onChangeText={(text) => {
                  setPassword(text);
                  if (passwordError) setPasswordError('');
                }}
                error={passwordError}
                placeholder="Enter your password"
                secureTextEntry
                autoCapitalize="none"
                autoCorrect={false}
              />

              <Button
                title={loading ? 'Logging in...' : 'Log In'}
                onPress={handleLogin}
                loading={loading}
                style={styles.loginButton}
              />
            </View>

            <View style={styles.dividerContainer}>
              <View style={styles.dividerLine} />
              <Text style={styles.dividerText}>or continue with</Text>
              <View style={styles.dividerLine} />
            </View>

            <View style={styles.socialContainer}>
              <Button
                title="Sign In with Google"
                onPress={() => handleSocialLogin('Google')}
                variant="outline"
                style={styles.socialButton}
              />
              <Button
                title="Sign In with Apple"
                onPress={() => handleSocialLogin('Apple')}
                variant="outline"
                style={styles.socialButton}
              />
            </View>
          </ScrollView>
        </KeyboardAvoidingView>
      </SafeAreaView>
    </ErrorBoundary>
  );
}

const styles = StyleSheet.create({
  container: {
    flex: 1,
    backgroundColor: Colors.darkBackground,
  },
  keyboardView: {
    flex: 1,
  },
  scrollContent: {
    flexGrow: 1,
    justifyContent: 'center',
    padding: 24,
  },
  headerContainer: {
    alignItems: 'center',
    marginBottom: 40,
  },
  logoText: {
    fontSize: 40,
    fontWeight: '900',
    color: Colors.primaryYellow,
    letterSpacing: 4,
  },
  subtitleText: {
    fontSize: 16,
    color: Colors.secondaryText,
    marginTop: 8,
  },
  formContainer: {
    width: '100%',
    marginBottom: 24,
  },
  loginButton: {
    marginTop: 8,
  },
  dividerContainer: {
    flexDirection: 'row',
    alignItems: 'center',
    marginVertical: 24,
  },
  dividerLine: {
    flex: 1,
    height: 1,
    backgroundColor: '#2C2C2E',
  },
  dividerText: {
    color: Colors.secondaryText,
    paddingHorizontal: 12,
    fontSize: 14,
  },
  socialContainer: {
    width: '100%',
    gap: 12,
  },
  socialButton: {
    borderColor: '#2C2C2E',
  },
});
