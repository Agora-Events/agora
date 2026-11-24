import React, { useEffect, useRef, useState } from 'react';
import {
  Animated,
  StyleSheet,
  Text,
  View,
  TouchableOpacity,
  Platform,
} from 'react-native';
import NetInfo, { NetInfoState } from '@react-native-community/netinfo';
import { useSafeAreaInsets } from 'react-native-safe-area-context';
import { Ionicons } from '@expo/vector-icons';

export interface OfflineBannerProps {
  /** Optional custom message when offline */
  message?: string;
  /** Whether to allow dismissing the banner manually until next state change */
  dismissible?: boolean;
}

export function OfflineBanner({
  message = 'No Internet Connection. You are currently offline.',
  dismissible = false,
}: OfflineBannerProps) {
  const insets = useSafeAreaInsets();
  const [isOffline, setIsOffline] = useState(false);
  const [isDismissed, setIsDismissed] = useState(false);
  const slideAnim = useRef(new Animated.Value(-140)).current;

  useEffect(() => {
    // Initial fetch
    NetInfo.fetch().then((state: NetInfoState) => {
      const offline = state.isConnected === false || state.isInternetReachable === false;
      setIsOffline(offline);
    });

    // Subscribe to network connection state changes
    const unsubscribe = NetInfo.addEventListener((state: NetInfoState) => {
      const offline = state.isConnected === false || state.isInternetReachable === false;
      setIsOffline(offline);
      if (!offline) {
        setIsDismissed(false);
      }
    });

    return () => {
      unsubscribe();
    };
  }, []);

  const shouldShow = isOffline && !isDismissed;

  useEffect(() => {
    Animated.spring(slideAnim, {
      toValue: shouldShow ? 0 : -140,
      useNativeDriver: Platform.OS !== 'web',
      bounciness: shouldShow ? 4 : 0,
      speed: 12,
    }).start();
  }, [shouldShow, slideAnim]);

  return (
    <Animated.View
      pointerEvents={shouldShow ? 'auto' : 'none'}
      style={[
        styles.container,
        {
          paddingTop: Math.max(insets.top, 10),
          transform: [{ translateY: slideAnim }],
        },
      ]}
      accessibilityRole="alert"
      accessibilityLiveRegion="assertive"
      accessibilityLabel={message}
      testID="offline-banner"
    >
      <View style={styles.content}>
        <Ionicons name="cloud-offline-outline" size={20} color="#FFFFFF" style={styles.icon} />
        <Text style={styles.text} numberOfLines={2}>
          {message}
        </Text>
        {dismissible && (
          <TouchableOpacity
            onPress={() => setIsDismissed(true)}
            hitSlop={{ top: 10, bottom: 10, left: 10, right: 10 }}
            accessibilityRole="button"
            accessibilityLabel="Dismiss offline banner"
            style={styles.dismissButton}
          >
            <Ionicons name="close" size={18} color="#FFFFFF" />
          </TouchableOpacity>
        )}
      </View>
    </Animated.View>
  );
}

const styles = StyleSheet.create({
  container: {
    position: 'absolute',
    top: 0,
    left: 0,
    right: 0,
    zIndex: 99999,
    backgroundColor: '#DC2626', // Red warning
    paddingBottom: 10,
    paddingHorizontal: 16,
    shadowColor: '#000000',
    shadowOffset: { width: 0, height: 3 },
    shadowOpacity: 0.35,
    shadowRadius: 5,
    elevation: 10,
  },
  content: {
    flexDirection: 'row',
    alignItems: 'center',
    justifyContent: 'center',
  },
  icon: {
    marginRight: 8,
  },
  text: {
    color: '#FFFFFF',
    fontSize: 13,
    fontWeight: '600',
    flexShrink: 1,
    textAlign: 'center',
  },
  dismissButton: {
    marginLeft: 8,
    padding: 2,
  },
});

export default OfflineBanner;
