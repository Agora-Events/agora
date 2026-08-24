import React, { useEffect, useRef } from "react";
import { Animated, StyleSheet, View } from "react-native";

/**
 * EventCardSkeleton — placeholder skeleton for the event list
 * (issue #1024).
 *
 * Mimics the layout of EventRow: a 64×64 thumbnail on the left and
 * three text lines on the right.  A looping pulse animation fades the
 * skeleton's opacity between 30 % and 70 % to communicate loading.
 */

const SKELETON_COLOR = "#27272A";
const PULSE_MIN = 0.3;
const PULSE_MAX = 0.7;
const PULSE_DURATION = 1000; // ms

export function EventCardSkeleton({ testID }: { testID?: string }) {
  const opacity = useRef(new Animated.Value(PULSE_MIN)).current;

  useEffect(() => {
    const animation = Animated.loop(
      Animated.sequence([
        Animated.timing(opacity, {
          toValue: PULSE_MAX,
          duration: PULSE_DURATION / 2,
          useNativeDriver: true,
        }),
        Animated.timing(opacity, {
          toValue: PULSE_MIN,
          duration: PULSE_DURATION / 2,
          useNativeDriver: true,
        }),
      ]),
    );
    animation.start();
    return () => animation.stop();
  }, [opacity]);

  return (
    <Animated.View
      style={[styles.row, { opacity }]}
      testID={testID ?? "event-card-skeleton"}
    >
      <View style={styles.thumb} />
      <View style={styles.body}>
        <View style={styles.lineTitle} />
        <View style={styles.lineMeta} />
        <View style={styles.lineDate} />
      </View>
    </Animated.View>
  );
}

const styles = StyleSheet.create({
  row: {
    flexDirection: "row",
    gap: 12,
    paddingHorizontal: 16,
    paddingVertical: 10,
  },
  thumb: {
    width: 64,
    height: 64,
    borderRadius: 8,
    backgroundColor: SKELETON_COLOR,
  },
  body: {
    flex: 1,
    justifyContent: "center",
    gap: 6,
  },
  lineTitle: {
    height: 14,
    width: "70%",
    borderRadius: 4,
    backgroundColor: SKELETON_COLOR,
  },
  lineMeta: {
    height: 10,
    width: "50%",
    borderRadius: 4,
    backgroundColor: SKELETON_COLOR,
  },
  lineDate: {
    height: 10,
    width: "40%",
    borderRadius: 4,
    backgroundColor: SKELETON_COLOR,
  },
});

export default EventCardSkeleton;