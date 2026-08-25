import React from 'react';
import { View, Text, StyleSheet, ScrollView, Pressable } from 'react-native';
import { useRouter } from 'expo-router';
import Colors from '@/constants/Colors';
import Button from '@/components/ui/Button';
import { useAuth } from '@/hooks/useAuth';
import { useEventCreationForm } from '@/hooks/useEventCreationForm';

interface EventItem {
  id: string;
  title: string;
  date: string;
  location: string;
  price: string;
  image: string;
}

const MOCK_EVENTS: EventItem[] = [
  {
    id: '1',
    title: 'Stellar Meridian 2026',
    date: 'Oct 15 - Oct 17, 2026',
    location: 'London, UK',
    price: '150 XLM',
    image: 'https://images.unsplash.com/photo-1511578314322-379afb476865?w=500&auto=format&fit=crop&q=60',
  },
  {
    id: '2',
    title: 'Agora Blockchain Summit',
    date: 'Nov 05, 2026',
    location: 'Paris, France',
    price: 'Free',
    image: 'https://images.unsplash.com/photo-1540575467063-178a50c2df87?w=500&auto=format&fit=crop&q=60',
  },
  {
    id: '3',
    title: 'Decentralized Music Festival',
    date: 'Dec 12, 2026',
    location: 'Miami, USA',
    price: '50 USDC',
    image: 'https://images.unsplash.com/photo-1506157786151-b8491531f063?w=500&auto=format&fit=crop&q=60',
  },
];

export default function DiscoverScreen() {
  const router = useRouter();
  const { isAuthenticated } = useAuth();
  const resetForm = useEventCreationForm((s) => s.resetForm);
  const goToStep = useEventCreationForm((s) => s.goToStep);

  const handleCreateEvent = () => {
    resetForm();
    goToStep(1);
    router.push('/create-event/step-1-basics');
  };

  return (
    <View style={styles.wrapper}>
      <ScrollView style={styles.container} contentContainerStyle={styles.content}>
      <Text style={styles.header}>Featured Events</Text>
      
      {MOCK_EVENTS.map((event) => (
        <Pressable
          key={event.id}
          style={({ pressed }) => [styles.card, pressed && styles.cardPressed]}
          onPress={() => router.push(`/event/${event.id}`)}
        >
          <View style={styles.cardImagePlaceholder}>
            <Text style={styles.placeholderText}>Agora Event</Text>
          </View>
          <View style={styles.cardDetails}>
            <Text style={styles.cardTitle}>{event.title}</Text>
            <Text style={styles.cardMeta}>{event.date}</Text>
            <Text style={styles.cardMeta}>{event.location}</Text>
            <View style={styles.cardFooter}>
              <Text style={styles.cardPrice}>{event.price}</Text>
              <Button
                title="Buy Ticket"
                onPress={() => router.push({ pathname: '/checkout', params: { eventId: event.id, eventTitle: event.title } })}
                style={styles.buyButton}
                textStyle={styles.buyButtonText}
              />
            </View>
          </View>
        </Pressable>
      ))}
      </ScrollView>

      {/* Floating Action Button for organizers */}
      {isAuthenticated && (
        <Pressable
          style={({ pressed }) => [styles.fab, pressed && styles.fabPressed]}
          onPress={handleCreateEvent}
        >
          <Text style={styles.fabText}>＋ Create Event</Text>
        </Pressable>
      )}
    </View>
  );
}

const styles = StyleSheet.create({
  wrapper: {
    flex: 1,
    backgroundColor: Colors.darkBackground,
  },
  container: {
    flex: 1,
    backgroundColor: Colors.darkBackground,
  },
  content: {
    padding: 16,
  },
  header: {
    fontSize: 24,
    fontWeight: 'bold',
    color: Colors.primaryText,
    marginBottom: 20,
  },
  card: {
    backgroundColor: '#1E1E20',
    borderRadius: 12,
    marginBottom: 20,
    overflow: 'hidden',
    borderWidth: 1,
    borderColor: '#2C2C2E',
  },
  cardPressed: {
    opacity: 0.95,
  },
  cardImagePlaceholder: {
    height: 150,
    backgroundColor: '#2C2C2E',
    justifyContent: 'center',
    alignItems: 'center',
    borderBottomWidth: 1,
    borderBottomColor: '#3A3A3C',
  },
  placeholderText: {
    color: Colors.primaryYellow,
    fontSize: 18,
    fontWeight: 'bold',
    letterSpacing: 1.5,
  },
  cardDetails: {
    padding: 16,
  },
  cardTitle: {
    fontSize: 18,
    fontWeight: 'bold',
    color: Colors.primaryText,
    marginBottom: 8,
  },
  cardMeta: {
    fontSize: 14,
    color: Colors.secondaryText,
    marginBottom: 4,
  },
  cardFooter: {
    flexDirection: 'row',
    justifyContent: 'space-between',
    alignItems: 'center',
    marginTop: 12,
    borderTopWidth: 1,
    borderTopColor: '#2C2C2E',
    paddingTop: 12,
  },
  cardPrice: {
    fontSize: 16,
    fontWeight: 'bold',
    color: Colors.primaryYellow,
  },
  buyButton: {
    height: 36,
    width: 110,
    borderRadius: 6,
  },
  buyButtonText: {
    fontSize: 14,
  },
  fab: {
    position: 'absolute',
    bottom: 24,
    right: 20,
    backgroundColor: Colors.primaryYellow,
    borderRadius: 28,
    paddingVertical: 12,
    paddingHorizontal: 20,
    shadowColor: Colors.primaryYellow,
    shadowOffset: { width: 0, height: 4 },
    shadowOpacity: 0.4,
    shadowRadius: 8,
    elevation: 6,
  },
  fabPressed: {
    opacity: 0.85,
    transform: [{ scale: 0.97 }],
  },
  fabText: {
    color: Colors.darkBackground,
    fontSize: 15,
    fontWeight: '700',
  },
});
