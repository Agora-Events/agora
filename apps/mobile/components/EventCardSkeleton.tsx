import React, { useEffect, useRef } from 'react';
import { View, StyleSheet, Animated } from 'react-native';

const EventCardSkeleton = () => {
  const pulse = useRef(new Animated.Value(0.3)).current;

  useEffect(() => {
    const animation = Animated.loop(
      Animated.sequence([
        Animated.timing(pulse, {
          toValue: 0.7,
          duration: 800,
          useNativeDriver: true,
        }),
        Animated.timing(pulse, {
          toValue: 0.3,
          duration: 800,
          useNativeDriver: true,
        }),
      ])
    );
    animation.start();

    return () => animation.stop();
  }, [pulse]);

  return (
    <View style={styles.card}>
      <Animated.View style={[styles.imagePlaceholder, { opacity: pulse }]} />
      <View style={styles.details}>
        <Animated.View style={[styles.titleLine, { opacity: pulse }]} />
        <Animated.View style={[styles.metaLine, { opacity: pulse }]} />
        <Animated.View style={[styles.metaLine, { opacity: pulse }]} />
        <View style={styles.footer}>
          <Animated.View style={[styles.priceLine, { opacity: pulse }]} />
          <Animated.View style={[styles.buttonPlaceholder, { opacity: pulse }]} />
        </View>
      </View>
    </View>
  );
};

const styles = StyleSheet.create({
  card: {
    backgroundColor: '#1E1E20',
    borderRadius: 12,
    marginBottom: 20,
    overflow: 'hidden',
    borderWidth: 1,
    borderColor: '#2C2C2E',
  },
  imagePlaceholder: {
    height: 150,
    backgroundColor: '#2C2C2E',
  },
  details: {
    padding: 16,
  },
  titleLine: {
    height: 20,
    backgroundColor: '#2C2C2E',
    borderRadius: 4,
    marginBottom: 8,
  },
  metaLine: {
    height: 14,
    backgroundColor: '#2C2C2E',
    borderRadius: 4,
    marginBottom: 4,
  },
  footer: {
    flexDirection: 'row',
    justifyContent: 'space-between',
    alignItems: 'center',
    marginTop: 12,
    borderTopWidth: 1,
    borderTopColor: '#2C2C2E',
    paddingTop: 12,
  },
  priceLine: {
    height: 16,
    width: 80,
    backgroundColor: '#2C2C2E',
    borderRadius: 4,
  },
  buttonPlaceholder: {
    height: 36,
    width: 110,
    borderRadius: 6,
    backgroundColor: '#2C2C2E',
  },
});

export default EventCardSkeleton;
