import React from 'react';
import { View, Text, StyleSheet, ScrollView } from 'react-native';
import { useLocalSearchParams, useRouter } from 'expo-router';
import Colors from '@/constants/Colors';
import Button from '@/components/ui/Button';

export default function EventDetailsScreen() {
  const { id } = useLocalSearchParams();
  const router = useRouter();

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

  return (
    <ScrollView style={styles.container} contentContainerStyle={styles.content}>
      <View style={styles.heroPlaceholder}>
        <Text style={styles.heroText}>AGORA EVENTS</Text>
      </View>

      <View style={styles.detailsContainer}>
        <Text style={styles.title}>{title}</Text>
        <Text style={styles.price}>150 XLM</Text>
        
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
          title="Buy Tickets Now"
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
  actionButton: {
    marginTop: 20,
  },
});
