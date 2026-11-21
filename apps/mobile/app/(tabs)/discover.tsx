import React, { useRef, useState, useEffect } from 'react';
import { View, Text, StyleSheet, ScrollView, Pressable, Animated } from 'react-native';
import { useRouter } from 'expo-router';
import Colors from '@/constants/Colors';
import Button from '@/components/ui/Button';
import { useAuth } from '@/hooks/useAuth';
import { useEventCreationForm } from '@/hooks/useEventCreationForm';

interface EventItem { id: string; title: string; date: string; location: string; price: string; image: string; }

const MOCK_EVENTS: EventItem[] = [
  { id: '1', title: 'Stellar Meridian 2026', date: 'Oct 15 - Oct 17, 2026', location: 'London, UK', price: '150 XLM', image: 'https://images.unsplash.com/photo-1511578314322-379afb476865?w=500&auto=format&fit=crop&q=60' },
  { id: '2', title: 'Agora Blockchain Summit', date: 'Nov 05, 2026', location: 'Paris, France', price: 'Free', image: 'https://images.unsplash.com/photo-1540575466703-178a50c2df87?w=500&auto=format&fit=crop&q=60' },
  { id: '3', title: 'Decentralized Music Festival', date: 'Dec 12, 2026', location: 'Miami, USA', price: '50 USDC', image: 'https://images.unsplash.com/photo-1506157786151-b8491531f063?w=500&auto=format&fit=crop&q=60' },
];

const EventCardSkeleton: React.FC = () => {
  const opacity = useRef(new Animated.Value(0.3)).current;

  useEffect(() => {
    const animation = Animated.loop(
      Animated.sequence([
        Animated.timing(opacity, { toValue: 0.7, duration: 800, useNativeDriver: true }),
        Animated.timing(opacity, { toValue: 0.3, duration: 800, useNativeDriver: true }),
      ])
    );
    animation.start();
    return () => animation.stop();
  }, [opacity]);

/**
 * Event discovery screen (issue #1004).
 *
 * List and map render from the same filtered array, so a category tap refines
 * both in the same frame — the acceptance criterion — rather than relying on
 * two code paths applying the same filter consistently.
 *
 * This is the canonical `/discover` screen. It lives under the `(tabs)` group
 * because that's what `(tabs)/_layout.tsx` registers as the Discover tab and
 * what `router.replace('/(tabs)/discover')` targets; the previous top-level
 * `app/discover.tsx` resolved to the same `/discover` URL (route groups don't
 * add a path segment) and has been removed to resolve the conflict (#1787).
 */

type ViewMode = "list" | "map";

function EventRow({ event }: { event: DiscoverEvent }) {
  return (
    <View style={styles.row}>
      {event.imageUrl ? (
        <Image source={{ uri: event.imageUrl }} style={styles.thumb} />
      ) : (
        <View style={[styles.thumb, styles.thumbPlaceholder]} />
      )}
      <View style={styles.rowBody}>
        <Text style={styles.rowTitle} numberOfLines={1}>
          {event.title}
        </Text>
        <Text style={styles.rowMeta} numberOfLines={1}>
          {event.category}
          {event.venue ? ` · ${event.venue}` : ""}
        </Text>
        <Text style={styles.rowMeta}>
          {new Date(event.startsAt).toLocaleDateString()}
        </Text>
      </View>
    </View>
  );
}

export default function DiscoverScreen() {
  const router = useRouter();
  const { isAuthenticated } = useAuth();
  const resetForm = useEventCreationForm((s) => s.resetForm);
  const goToStep = useEventCreationForm((s) => s.goToStep);
  const [loading, setLoading] = useState(true);
