import React from 'react';
import { View, Text, StyleSheet, ScrollView } from 'react-native';
import Colors from '@/constants/Colors';

interface TicketItem {
  id: string;
  eventTitle: string;
  date: string;
  seat: string;
  txHash: string;
}

const MOCK_TICKETS: TicketItem[] = [
  {
    id: 'T-1004',
    eventTitle: 'Stellar Meridian 2026',
    date: 'Oct 15, 2026',
    seat: 'General Admission',
    txHash: '0x3f...b82d',
  },
];

export default function TicketsScreen() {
  return (
    <ScrollView style={styles.container} contentContainerStyle={styles.content}>
      <Text style={styles.header}>My Tickets</Text>
      
      {MOCK_TICKETS.length > 0 ? (
        MOCK_TICKETS.map((ticket) => (
          <View key={ticket.id} style={styles.ticketCard}>
            <View style={styles.ticketHeader}>
              <Text style={styles.ticketId}>{ticket.id}</Text>
              <Text style={styles.verifiedBadge}>Verified</Text>
            </View>
            <Text style={styles.eventTitle}>{ticket.eventTitle}</Text>
            <View style={styles.detailsRow}>
              <View>
                <Text style={styles.label}>Date</Text>
                <Text style={styles.value}>{ticket.date}</Text>
              </View>
              <View>
                <Text style={styles.label}>Section/Seat</Text>
                <Text style={styles.value}>{ticket.seat}</Text>
              </View>
            </View>
            <View style={styles.txContainer}>
              <Text style={styles.txLabel}>Transaction Hash</Text>
              <Text style={styles.txValue}>{ticket.txHash}</Text>
            </View>
          </View>
        ))
      ) : (
        <View style={styles.emptyContainer}>
          <Text style={styles.emptyText}>You don't have any tickets yet.</Text>
        </View>
      )}
    </ScrollView>
  );
}

const styles = StyleSheet.create({
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
  ticketCard: {
    backgroundColor: '#1E1E20',
    borderRadius: 12,
    padding: 16,
    marginBottom: 16,
    borderLeftWidth: 4,
    borderLeftColor: Colors.primaryYellow,
    borderWidth: 1,
    borderColor: '#2C2C2E',
  },
  ticketHeader: {
    flexDirection: 'row',
    justifyContent: 'space-between',
    marginBottom: 12,
  },
  ticketId: {
    color: Colors.secondaryText,
    fontSize: 12,
    fontWeight: 'bold',
  },
  verifiedBadge: {
    backgroundColor: '#34C75922',
    color: '#34C759',
    fontSize: 10,
    fontWeight: 'bold',
    paddingHorizontal: 8,
    paddingVertical: 4,
    borderRadius: 4,
    overflow: 'hidden',
  },
  eventTitle: {
    fontSize: 18,
    fontWeight: 'bold',
    color: Colors.primaryText,
    marginBottom: 16,
  },
  detailsRow: {
    flexDirection: 'row',
    justifyContent: 'space-between',
    marginBottom: 16,
  },
  label: {
    fontSize: 12,
    color: Colors.secondaryText,
    marginBottom: 4,
  },
  value: {
    fontSize: 14,
    color: Colors.primaryText,
    fontWeight: '500',
  },
  txContainer: {
    borderTopWidth: 1,
    borderTopColor: '#2C2C2E',
    paddingTop: 12,
  },
  txLabel: {
    fontSize: 11,
    color: Colors.secondaryText,
    marginBottom: 2,
  },
  txValue: {
    fontSize: 12,
    color: Colors.primaryYellow,
    fontFamily: 'SpaceMono',
  },
  emptyContainer: {
    flex: 1,
    justifyContent: 'center',
    alignItems: 'center',
    paddingVertical: 60,
  },
  emptyText: {
    color: Colors.secondaryText,
    fontSize: 16,
    textAlign: 'center',
  },
});
