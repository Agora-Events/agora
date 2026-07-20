import React, { useEffect } from 'react';
import { ThemeProvider, DarkTheme } from '@react-navigation/native';
import { useFonts } from 'expo-font';
import { Stack, useRouter, useSegments } from 'expo-router';
import * as SplashScreen from 'expo-splash-screen';
import 'react-native-reanimated';

import { useAuth } from '@/hooks/useAuth';
import ErrorBoundary from '@/components/ui/ErrorBoundary';
import Colors from '@/constants/Colors';

// Prevent the splash screen from auto-hiding before asset loading is complete.
SplashScreen.preventAutoHideAsync();

const AgoraNavigationTheme = {
  ...DarkTheme,
  colors: {
    ...DarkTheme.colors,
    background: Colors.darkBackground,
    primary: Colors.primaryYellow,
    card: Colors.darkBackground,
    text: Colors.primaryText,
    border: '#1E1E20',
  },
};

function AppNavigation() {
  const { isAuthenticated } = useAuth();
  const segments = useSegments();
  const router = useRouter();

  useEffect(() => {
    // Check if the user is in the auth route group
    const inAuthGroup = segments[0] === 'auth';

    if (!isAuthenticated && !inAuthGroup) {
      // Redirect to the auth landing screen if not authenticated
      router.replace('/auth');
    } else if (isAuthenticated && inAuthGroup) {
      // Redirect to discover tab once authenticated
      router.replace('/(tabs)/discover');
    }
  }, [isAuthenticated, segments, router]);

  return (
    <Stack>
      <Stack.Screen name="(tabs)" options={{ headerShown: false }} />
      <Stack.Screen name="auth" options={{ headerShown: false }} />
      <Stack.Screen
        name="checkout"
        options={{
          presentation: 'modal',
          title: 'Ticket Checkout',
          headerStyle: { backgroundColor: Colors.darkBackground },
          headerTintColor: Colors.primaryText,
          headerShadowVisible: false,
        }}
      />
      <Stack.Screen
        name="event/[id]"
        options={{
          presentation: 'modal',
          title: 'Event Details',
          headerStyle: { backgroundColor: Colors.darkBackground },
          headerTintColor: Colors.primaryText,
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
          headerStyle: { backgroundColor: Colors.darkBackground },
          headerTintColor: Colors.primaryText,
          headerShadowVisible: false,
        }}
      />
      <Stack.Screen
        name="create-event/step-2-location"
        options={{
          title: 'Create Event',
          headerStyle: { backgroundColor: Colors.darkBackground },
          headerTintColor: Colors.primaryText,
          headerShadowVisible: false,
        }}
      />
      <Stack.Screen
        name="create-event/step-3-tickets"
        options={{
          title: 'Create Event',
          headerStyle: { backgroundColor: Colors.darkBackground },
          headerTintColor: Colors.primaryText,
          headerShadowVisible: false,
        }}
      />
      <Stack.Screen
        name="create-event/step-4-review"
        options={{
          title: 'Create Event',
          headerStyle: { backgroundColor: Colors.darkBackground },
          headerTintColor: Colors.primaryText,
          headerShadowVisible: false,
        }}
      />
      <Stack.Screen name="+not-found" options={{ title: 'Not Found' }} />
    </Stack>
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
      <ThemeProvider value={AgoraNavigationTheme}>
        <AppNavigation />
      </ThemeProvider>
    </ErrorBoundary>
  );
}
