import React from 'react';
import { View, Text, ScrollView, StyleSheet } from 'react-native';
import { useTheme } from '@/hooks/useTheme';
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
  const { theme, palette } = useTheme();

  return (
    <ScrollView
      style={{ flex: 1, backgroundColor: theme.background }}
      contentContainerStyle={styles.content}
    >
      <Text style={[styles.header, { color: theme.text }]}>My Tickets</Text>

      {MOCK_TICKETS.length > 0 ? (
        MOCK_TICKETS.map((ticket) => (
          <View
            key={ticket.id}
            style={[
              styles.ticketCard,
              {
                backgroundColor: theme.cardBackground,
                borderColor: theme.border,
                borderLeftColor: palette.primaryYellow,
              },
            ]}
          >
            <View style={styles.ticketHeader}>
              <Text style={[styles.ticketId, { color: theme.icon }]}>{ticket.id}</Text>
              <Text
                style={[
                  styles.verifiedBadge,
                  {
                    backgroundColor: theme.successBadgeBackground,
                    color: theme.successBadgeText,
                  },
                ]}
              >
                Verified
              </Text>
            </View>

            <Text style={[styles.eventTitle, { color: theme.text }]}>
              {ticket.eventTitle}
            </Text>

            <View style={styles.detailsRow}>
              <View>
                <Text style={[styles.label, { color: theme.icon }]}>Date</Text>
                <Text style={[styles.value, { color: theme.text }]}>{ticket.date}</Text>
              </View>
              <View>
                <Text style={[styles.label, { color: theme.icon }]}>Section/Seat</Text>
                <Text style={[styles.value, { color: theme.text }]}>{ticket.seat}</Text>
              </View>
            </View>

            <View style={[styles.txContainer, { borderTopColor: theme.border }]}>
              <Text style={[styles.txLabel, { color: theme.icon }]}>Transaction Hash</Text>
              <Text style={[styles.txValue, { color: palette.primaryYellow }]}>
                {ticket.txHash}
              </Text>
            </View>
          </View>
        ))
      ) : (
        <View style={styles.emptyContainer}>
          <Text style={[styles.emptyText, { color: theme.icon }]}>
            You don't have any tickets yet.
          </Text>
        </View>
      )}
    </ScrollView>
  );
}

// Only theme-independent structural styles remain here.
const styles = StyleSheet.create({
  content: {
    padding: 16,
  },
  header: {
    fontSize: 24,
    fontWeight: 'bold',
    marginBottom: 20,
  },
  ticketCard: {
    borderRadius: 12,
    padding: 16,
    marginBottom: 16,
    borderLeftWidth: 4,
    borderWidth: 1,
  },
  ticketHeader: {
    flexDirection: 'row',
    justifyContent: 'space-between',
    marginBottom: 12,
  },
  ticketId: {
    fontSize: 12,
    fontWeight: 'bold',
  },
  verifiedBadge: {
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
    marginBottom: 16,
  },
  detailsRow: {
    flexDirection: 'row',
    justifyContent: 'space-between',
    marginBottom: 16,
  },
  label: {
    fontSize: 12,
    marginBottom: 4,
  },
  value: {
    fontSize: 14,
    fontWeight: '500',
  },
  txContainer: {
    borderTopWidth: 1,
    paddingTop: 12,
  },
  txLabel: {
    fontSize: 11,
    marginBottom: 2,
  },
  txValue: {
    fontSize: 12,
    fontFamily: 'SpaceMono',
  },
  emptyContainer: {
    flex: 1,
    justifyContent: 'center',
    alignItems: 'center',
    paddingVertical: 60,
  },
  emptyText: {
    fontSize: 16,
    textAlign: 'center',
  },
});
