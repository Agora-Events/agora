import React, { useCallback, useEffect, useMemo, useState } from 'react';
import { View, Text, StyleSheet, ScrollView } from 'react-native';
import { useLocalSearchParams, useRouter } from 'expo-router';
import AsyncStorage from '@react-native-async-storage/async-storage';
import Colors from '@/constants/Colors';
import Button from '@/components/ui/Button';
import LikeableBanner from '@/components/LikeableBanner';
import { useAuth } from '@/hooks/useAuth';
import { useLiveTicketInventory } from '@/hooks/useLiveTicketInventory';
import { getTiersForEvent, totalRemaining } from '@/lib/ticketTiers';

/** AsyncStorage key for an event's liked/saved status (issue #1552). */
const likedStorageKey = (eventId: string) => `agora.event.liked.${eventId}`;

export default function EventDetailsScreen() {
  const { id } = useLocalSearchParams();
  const router = useRouter();
  const { token } = useAuth();

  // Find simulated event details
  const getEventTitle = (eventId: string) => {
    switch (eventId) {
      case '1': return 'Stellar Meridian 2026';
      case '2': return 'Agora Blockchain Summit';
      case '3': return 'Decentralized Music Festival';
      default: return 'Agora Event';
    }
  };

  const title = getEventTitle(id as string);
  const eventId = (id as string) ?? '1';

  // Live inventory so availability shown here matches checkout (issue #1010).
  const baseTiers = useMemo(() => getTiersForEvent(eventId), [eventId]);
  const { tiers, isLive } = useLiveTicketInventory({
    eventId,
    tiers: baseTiers,
    token: token ?? undefined,
  });

  const remaining = totalRemaining(tiers);
  const isSoldOut = remaining === 0;

  // Issue #1552: double-tap-to-like on the event banner, persisted locally
  // per event so the "save/like" status survives navigating away and back.
  const [liked, setLiked] = useState(false);

  useEffect(() => {
    let isMounted = true;
    AsyncStorage.getItem(likedStorageKey(eventId))
      .then((value) => {
        if (isMounted) setLiked(value === 'true');
      })
      .catch(() => {
        // No persisted preference (or storage unavailable) — default to not liked.
      });
    return () => {
      isMounted = false;
    };
  }, [eventId]);

  const handleLikeChange = useCallback(
    (next: boolean) => {
      setLiked(next);
      AsyncStorage.setItem(likedStorageKey(eventId), next ? 'true' : 'false').catch(() => {
        // Best-effort persistence; in-memory state still reflects the change.
      });
    },
    [eventId]
  );

  return (
    <ScrollView style={styles.container} contentContainerStyle={styles.content}>
      <LikeableBanner
        liked={liked}
        onChange={handleLikeChange}
        style={styles.heroPlaceholder}
        testID="event-banner"
      >
        <Text style={styles.heroText}>AGORA EVENTS</Text>
      </LikeableBanner>

      <View style={styles.detailsContainer}>
        <Text style={styles.title}>{title}</Text>
        <Text style={styles.price}>150 XLM</Text>

        {remaining !== undefined ? (
          <Text testID="event-remaining-tickets" style={styles.availability}>
            {isSoldOut ? 'Sold out' : `${remaining} tickets remaining`}
            {isLive ? ' • live' : ''}
          </Text>
        ) : null}
        
        <View style={styles.infoBlock}>
          <Text style={styles.label}>Date & Time</Text>
          <Text style={styles.value}>Thursday, Oct 15, 2026 • 9:00 AM</Text>
        </View>

        <View style={styles.infoBlock}>
          <Text style={styles.label}>Location</Text>
          <Text style={styles.value}>London Science Center, London, UK</Text>
        </View>

        <View style={styles.infoBlock}>
          <Text style={styles.label}>About the Event</Text>
          <Text style={styles.description}>
            Join developers, founders, and community members from around the world for three days of keynotes, tech talks, panel discussions, and networking opportunities. Learn about the latest developments on Stellar, Soroban, and the open financial stack.
          </Text>
        </View>

        <Button
          testID="event-buy-tickets-button"
          title={isSoldOut ? 'Sold Out' : 'Buy Tickets Now'}
          disabled={isSoldOut}
          onPress={() => {
            router.push({
              pathname: '/checkout',
              params: { eventId: id, eventTitle: title }
            });
          }}
          style={styles.actionButton}
        />
      </View>
    </ScrollView>
  );
}

const styles = StyleSheet.create({
  container: {
    flex: 1,
    backgroundColor: Colors.darkBackground,
  },
  content: {
    paddingBottom: 40,
  },
  heroPlaceholder: {
    height: 200,
    backgroundColor: '#1E1E20',
    justifyContent: 'center',
    alignItems: 'center',
    borderBottomWidth: 1,
    borderBottomColor: '#2C2C2E',
  },
  heroText: {
    fontSize: 24,
    color: Colors.primaryYellow,
    fontWeight: '900',
    letterSpacing: 2,
  },
  detailsContainer: {
    padding: 20,
  },
  title: {
    fontSize: 24,
    fontWeight: 'bold',
    color: Colors.primaryText,
    marginBottom: 8,
  },
  price: {
    fontSize: 20,
    fontWeight: 'bold',
    color: Colors.primaryYellow,
    marginBottom: 24,
  },
  infoBlock: {
    marginBottom: 20,
  },
  label: {
    fontSize: 12,
    fontWeight: 'bold',
    textTransform: 'uppercase',
    color: Colors.secondaryText,
    marginBottom: 6,
  },
  value: {
    fontSize: 16,
    color: Colors.primaryText,
    fontWeight: '500',
  },
  description: {
    fontSize: 14,
    color: Colors.secondaryText,
    lineHeight: 20,
  },
  availability: {
    fontSize: 13,
    color: Colors.secondaryText,
    marginTop: 6,
  },
  actionButton: {
    marginTop: 20,
  },
});
