import React, { useCallback, useEffect, useState } from 'react';
import { StyleSheet, Text, View } from 'react-native';
import NetInfo, { NetInfoState } from '@react-native-community/netinfo';
import { Ionicons } from '@expo/vector-icons';
import { useSafeAreaInsets } from 'react-native-safe-area-context';
import Colors from '../constants/Colors';
import Button from './ui/Button';

/**
 * Full-screen fallback shown whenever the device has no network connection.
 *
 * Without this, screens that depend on network requests (events list, ticket
 * purchase, wallet sync, etc.) just hang indefinitely with a spinner and no
 * way for the user to know what's wrong or recover. This takes over the
 * screen with a clear "No Internet Connection" message and a manual retry
 * action, and disappears automatically as soon as connectivity returns.
 */
export function OfflineFallbackScreen() {
  const insets = useSafeAreaInsets();
  const [isOffline, setIsOffline] = useState(false);
  const [isRetrying, setIsRetrying] = useState(false);

  const evaluate = useCallback((state: NetInfoState) => {
    const offline = state.isConnected === false || state.isInternetReachable === false;
    setIsOffline(offline);
    return offline;
  }, []);

  useEffect(() => {
    NetInfo.fetch().then(evaluate);

    const unsubscribe = NetInfo.addEventListener(evaluate);

    return () => {
      unsubscribe();
    };
  }, [evaluate]);

  const handleRetry = useCallback(async () => {
    setIsRetrying(true);
    try {
      const state = await NetInfo.refresh();
      evaluate(state);
    } catch {
      // NetInfo.refresh() isn't available on every platform; fall back to fetch().
      const state = await NetInfo.fetch();
      evaluate(state);
    } finally {
      setIsRetrying(false);
    }
  }, [evaluate]);

  if (!isOffline) {
    return null;
  }

  return (
    <View
      style={[styles.container, { paddingTop: insets.top, paddingBottom: insets.bottom }]}
      accessibilityViewIsModal
      accessibilityRole="alert"
      accessibilityLiveRegion="assertive"
      testID="offline-fallback-screen"
    >
      <View style={styles.content}>
        <Ionicons name="cloud-offline-outline" size={64} color={Colors.secondaryText} />
        <Text style={styles.title}>No Internet Connection</Text>
        <Text style={styles.message}>
          You&apos;re offline. Check your connection and try again.
        </Text>
        <Button
          title="Try Again"
          onPress={handleRetry}
          loading={isRetrying}
          style={styles.button}
          testID="offline-fallback-retry"
          accessibilityLabel="Retry connection"
        />
      </View>
    </View>
  );
}

const styles = StyleSheet.create({
  container: {
    position: 'absolute',
    top: 0,
    left: 0,
    right: 0,
    bottom: 0,
    zIndex: 99998,
    backgroundColor: Colors.darkBackground,
  },
  content: {
    flex: 1,
    justifyContent: 'center',
    alignItems: 'center',
    paddingHorizontal: 32,
  },
  title: {
    fontSize: 22,
    fontWeight: 'bold',
    color: Colors.primaryText,
    marginTop: 20,
    marginBottom: 8,
    textAlign: 'center',
  },
  message: {
    fontSize: 15,
    color: Colors.secondaryText,
    marginBottom: 28,
    textAlign: 'center',
  },
  button: {
    maxWidth: 220,
  },
});

export default OfflineFallbackScreen;
