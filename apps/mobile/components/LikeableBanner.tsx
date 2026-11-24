import React from 'react';
import { Pressable, StyleProp, StyleSheet, ViewStyle } from 'react-native';
import { Gesture, GestureDetector } from 'react-native-gesture-handler';
import Animated, {
  Easing,
  useAnimatedStyle,
  useSharedValue,
  withSequence,
  withTiming,
} from 'react-native-reanimated';
import Ionicons from '@expo/vector-icons/Ionicons';
import Colors from '@/constants/Colors';
import { triggerSelectionChange } from '@/services/feedback';

export interface LikeableBannerProps {
  /** Current liked/saved status of the event this banner belongs to. */
  liked: boolean;
  /** Called with the new liked value whenever it should change. */
  onChange: (liked: boolean) => void;
  /** Banner content (hero image or placeholder) to render underneath the overlay. */
  children: React.ReactNode;
  style?: StyleProp<ViewStyle>;
  /** Base testID; the heart button gets `${testID}-like-button`. */
  testID?: string;
}

/**
 * Issue #1552: Instagram-style double-tap-to-like on the main event image
 * banner.
 *
 * Wraps the banner in a `GestureDetector` that recognizes a double tap and
 * "likes" the event (mirrors Instagram: double-tapping always *likes*, it
 * never unlikes, and always re-plays the heart-burst animation as feedback
 * even if the event was already liked). A small heart button in the top
 * corner mirrors the current state and lets a user toggle it (like *or*
 * unlike) with a single tap — this also doubles as the reliable,
 * RNTL-testable entry point into the same `onChange` callback, since
 * simulating a real two-finger-independent double-tap gesture through
 * `react-native-gesture-handler`'s native recognizer isn't something
 * `@testing-library/react-native` can drive directly.
 */
export default function LikeableBanner({
  liked,
  onChange,
  children,
  style,
  testID,
}: LikeableBannerProps) {
  const burstOpacity = useSharedValue(0);
  const burstScale = useSharedValue(0.6);

  const playBurst = () => {
    burstOpacity.value = withSequence(
      withTiming(1, { duration: 120, easing: Easing.out(Easing.quad) }),
      withTiming(0, { duration: 350, easing: Easing.in(Easing.quad) })
    );
    burstScale.value = withSequence(
      withTiming(1.15, { duration: 160, easing: Easing.out(Easing.back(2)) }),
      withTiming(1, { duration: 140 })
    );
  };

  const handleDoubleTap = () => {
    if (!liked) {
      onChange(true);
    }
    void triggerSelectionChange();
    playBurst();
  };

  const handleHeartPress = () => {
    const next = !liked;
    onChange(next);
    void triggerSelectionChange();
    if (next) {
      playBurst();
    }
  };

  const doubleTap = Gesture.Tap()
    .numberOfTaps(2)
    .onEnd(() => {
      handleDoubleTap();
    });

  const burstStyle = useAnimatedStyle(() => ({
    opacity: burstOpacity.value,
    transform: [{ scale: burstScale.value }],
  }));

  const likeButtonTestID = testID ? `${testID}-like-button` : 'event-banner-like-button';

  return (
    <GestureDetector gesture={doubleTap}>
      <Animated.View style={[styles.container, style]} testID={testID}>
        {children}

        <Animated.View style={[styles.burstOverlay, burstStyle]} pointerEvents="none">
          <Ionicons name="heart" size={96} color="#FFFFFF" />
        </Animated.View>

        <Pressable
          onPress={handleHeartPress}
          style={styles.likeButton}
          testID={likeButtonTestID}
          accessibilityRole="button"
          accessibilityLabel={liked ? 'Unlike event' : 'Like event'}
          hitSlop={8}
        >
          <Ionicons
            name={liked ? 'heart' : 'heart-outline'}
            size={22}
            color={liked ? Colors.accentRed : Colors.primaryText}
          />
        </Pressable>
      </Animated.View>
    </GestureDetector>
  );
}

const styles = StyleSheet.create({
  container: {
    position: 'relative',
    overflow: 'hidden',
  },
  burstOverlay: {
    position: 'absolute',
    top: 0,
    left: 0,
    right: 0,
    bottom: 0,
    justifyContent: 'center',
    alignItems: 'center',
  },
  likeButton: {
    position: 'absolute',
    top: 12,
    right: 12,
    width: 36,
    height: 36,
    borderRadius: 18,
    backgroundColor: 'rgba(0,0,0,0.35)',
    justifyContent: 'center',
    alignItems: 'center',
  },
});
