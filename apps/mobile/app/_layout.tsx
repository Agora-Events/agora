import React, { useEffect, useMemo } from 'react';
import { ThemeProvider as NavThemeProvider, DarkTheme, DefaultTheme } from '@react-navigation/native';
import { useFonts } from 'expo-font';
import { Stack, useRouter, useSegments } from 'expo-router';
import * as SplashScreen from 'expo-splash-screen';
import 'react-native-reanimated';

import { ThemeProvider, useThemeContext } from '@/context/ThemeContext';
import { useAuth } from '@/hooks/useAuth';
import ErrorBoundary from '@/components/ui/ErrorBoundary';
import { Colors } from '@/constants/Colors';

// Prevent the splash screen from auto-hiding before asset loading is complete.
SplashScreen.preventAutoHideAsync();

/**
 * Builds a react-navigation theme from the current Agora theme tokens.
 * Re-runs whenever colorScheme changes so the navigator updates instantly.
 */
function useAgoraNavTheme() {
  const { colorScheme } = useThemeContext();

  return useMemo(() => {
    const base = colorScheme === 'dark' ? DarkTheme : DefaultTheme;
    return {
      ...base,
      colors: {
        ...base.colors,
        background: Colors[colorScheme].background,
        primary: Colors.primaryYellow,
        card: Colors[colorScheme].background,
        text: Colors[colorScheme].text,
        border: colorScheme === 'dark' ? '#1E1E20' : '#E5E5EA',
        notification: Colors.primaryYellow,
      },
    };
  }, [colorScheme]);
}

function AppNavigation() {
  const { isAuthenticated } = useAuth();
  const segments = useSegments();
  const router = useRouter();
  const navTheme = useAgoraNavTheme();
  const { colorScheme } = useThemeContext();

  useEffect(() => {
    const inAuthGroup = segments[0] === 'auth';

    if (!isAuthenticated && !inAuthGroup) {
      router.replace('/auth');
    } else if (isAuthenticated && inAuthGroup) {
      router.replace('/(tabs)/discover');
    }
  }, [isAuthenticated, segments, router]);

  const headerStyle = useMemo(
    () => ({
      backgroundColor: Colors[colorScheme].background,
    }),
    [colorScheme]
  );

  const headerTintColor = Colors[colorScheme].text;

  return (
    <NavThemeProvider value={navTheme}>
      <Stack>
        <Stack.Screen name="(tabs)" options={{ headerShown: false }} />
        <Stack.Screen name="auth" options={{ headerShown: false }} />
        <Stack.Screen
          name="checkout/index"
          options={{
            presentation: 'modal',
            title: 'Ticket Checkout',
            headerStyle,
            headerTintColor,
            headerShadowVisible: false,
          }}
        />
        <Stack.Screen
          name="checkout/complete"
          options={{
            presentation: 'modal',
            title: 'Purchase Complete',
            headerStyle,
            headerTintColor,
            headerShadowVisible: false,
            gestureEnabled: false,
          }}
        />
        <Stack.Screen
          name="event/[id]"
          options={{
            presentation: 'modal',
            title: 'Event Details',
            headerStyle,
            headerTintColor,
            headerShadowVisible: false,
          }}
        />
        <Stack.Screen
          name="create-event/index"
          options={{ headerShown: false }}
        />
        <Stack.Screen
          name="create-event/step-1-basics"
          options={{
            title: 'Create Event',
            headerStyle,
            headerTintColor,
            headerShadowVisible: false,
          }}
        />
        <Stack.Screen
          name="create-event/step-2-location"
          options={{
            title: 'Create Event',
            headerStyle,
            headerTintColor,
            headerShadowVisible: false,
          }}
        />
        <Stack.Screen
          name="create-event/step-3-tickets"
          options={{
            title: 'Create Event',
            headerStyle,
            headerTintColor,
            headerShadowVisible: false,
          }}
        />
        <Stack.Screen
          name="create-event/step-4-review"
          options={{
            title: 'Create Event',
            headerStyle,
            headerTintColor,
            headerShadowVisible: false,
          }}
        />
        <Stack.Screen
          name="organizer/staking"
          options={{
            title: 'Organizer Staking',
            headerStyle,
            headerTintColor,
            headerShadowVisible: false,
          }}
        />
        <Stack.Screen
          name="organizer/dashboard"
          options={{
            title: 'Organizer Dashboard',
            headerStyle,
            headerTintColor,
            headerShadowVisible: false,
          }}
        />
        <Stack.Screen name="+not-found" options={{ title: 'Not Found' }} />
      </Stack>
    </NavThemeProvider>
  );
}

export default function RootLayout() {
  const [loaded] = useFonts({
    /* eslint-disable-next-line @typescript-eslint/no-require-imports */
    SpaceMono: require('../assets/fonts/SpaceMono-Regular.ttf'),
  });

  useEffect(() => {
    if (loaded) {
      SplashScreen.hideAsync();
    }
  }, [loaded]);

  if (!loaded) {
    return null;
  }

  return (
    <ErrorBoundary>
      <ThemeProvider>
        <AppNavigation />
      </ThemeProvider>
    </ErrorBoundary>
  );
}
