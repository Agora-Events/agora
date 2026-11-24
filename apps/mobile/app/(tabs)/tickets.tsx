import React, { useState, useCallback, useEffect } from 'react';
import {
  View,
  Text,
  ScrollView,
  StyleSheet,
  RefreshControl,
  TouchableOpacity,
  ActivityIndicator,
} from 'react-native';
import NetInfo, { NetInfoState } from '@react-native-community/netinfo';
import { Ionicons } from '@expo/vector-icons';
import { useTheme } from '@/hooks/useTheme';
import Colors from '@/constants/Colors';
import {
  AttendeeTicket,
  FALLBACK_TICKETS,
  fetchAttendeeTickets,
  getCachedAttendeeTickets,
} from '@/services/ticketService';

export default function TicketsScreen() {
  const { theme, palette } = useTheme();
  const [tickets, setTickets] = useState<AttendeeTicket[]>(FALLBACK_TICKETS);
  const [refreshing, setRefreshing] = useState(false);
  const [isRetrying, setIsRetrying] = useState(false);
  const [networkError, setNetworkError] = useState<string | null>(null);

  const loadTickets = useCallback(async (isRefresh = false) => {
    if (isRefresh) {
      setRefreshing(true);
    }
    try {
      const fetchedTickets = await fetchAttendeeTickets();
      setTickets(fetchedTickets);
      setNetworkError(null);
    } catch (err: unknown) {
      const message =
        err instanceof Error
          ? err.message
          : 'Unable to refresh tickets. Please check your network connection.';
      setNetworkError(message);

      // Attempt to preserve previously cached tickets on error
      const cached = await getCachedAttendeeTickets();
      if (cached && cached.length > 0) {
        setTickets(cached);
      }
    } finally {
      if (isRefresh) {
        setRefreshing(false);
      }
    }
  }, []);

  useEffect(() => {
    // Initial fetch on mount
    void loadTickets(false);

    // Subscribe to network connectivity changes
    const unsubscribe = NetInfo.addEventListener((state: NetInfoState) => {
      const isOffline = state.isConnected === false || state.isInternetReachable === false;
      if (isOffline) {
        setNetworkError('You are currently offline. Check your network connection.');
      } else if (state.isConnected === true && state.isInternetReachable !== false) {
        // Auto-clear network error if connectivity is restored
        setNetworkError(null);
      }
    });

    return () => {
      unsubscribe();
    };
  }, [loadTickets]);

  const onRefresh = useCallback(async () => {
    await loadTickets(true);
  }, [loadTickets]);

  const handleRetry = useCallback(async () => {
    setIsRetrying(true);
    try {
      await loadTickets(false);
    } finally {
      setIsRetrying(false);
    }
  }, [loadTickets]);

  return (
    <View style={[styles.container, { backgroundColor: theme.background }]}>
      {/* Network Error Retry Banner */}
      {networkError && (
        <View
          style={[styles.bannerContainer, { backgroundColor: '#DC2626' }]}
          testID="network-error-banner"
          accessibilityRole="alert"
          accessibilityLiveRegion="assertive"
        >
          <View style={styles.bannerContent}>
            <Ionicons
              name="cloud-offline-outline"
              size={20}
              color="#FFFFFF"
              style={styles.bannerIcon}
            />
            <Text style={styles.bannerText} numberOfLines={2}>
              {networkError}
            </Text>
          </View>
          <TouchableOpacity
            style={styles.retryButton}
            onPress={handleRetry}
            disabled={isRetrying || refreshing}
            testID="retry-button"
            accessibilityRole="button"
            accessibilityLabel="Retry ticket refresh"
          >
            {isRetrying ? (
              <ActivityIndicator size="small" color="#FFFFFF" />
            ) : (
              <View style={styles.retryButtonContent}>
                <Ionicons name="refresh" size={14} color="#FFFFFF" style={styles.retryIcon} />
                <Text style={styles.retryButtonText}>Retry</Text>
              </View>
            )}
          </TouchableOpacity>
        </View>
      )}

      <ScrollView
        style={{ flex: 1 }}
        contentContainerStyle={styles.content}
        testID="tickets-scroll-view"
        refreshControl={
          <RefreshControl
            testID="tickets-refresh-control"
            refreshing={refreshing}
            onRefresh={onRefresh}
            tintColor={Colors.palette.primaryYellow}
            colors={[Colors.palette.primaryYellow]}
          />
        }
      >
        <Text style={[styles.header, { color: theme.text }]}>My Tickets</Text>

        {tickets.length > 0 ? (
          tickets.map((ticket) => (
            <View
              key={ticket.id}
              testID="ticket-card"
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
              You don&apos;t have any tickets yet.
            </Text>
          </View>
        )}
      </ScrollView>
    </View>
  );
}

const styles = StyleSheet.create({
  container: {
    flex: 1,
  },
  content: {
    padding: 16,
  },
  bannerContainer: {
    paddingHorizontal: 16,
    paddingVertical: 12,
    flexDirection: 'row',
    alignItems: 'center',
    justifyContent: 'space-between',
    zIndex: 10,
    elevation: 4,
    shadowColor: '#000000',
    shadowOffset: { width: 0, height: 2 },
    shadowOpacity: 0.25,
    shadowRadius: 3,
  },
  bannerContent: {
    flex: 1,
    flexDirection: 'row',
    alignItems: 'center',
    marginRight: 12,
  },
  bannerIcon: {
    marginRight: 8,
  },
  bannerText: {
    color: '#FFFFFF',
    fontSize: 13,
    fontWeight: '600',
    flexShrink: 1,
  },
  retryButton: {
    backgroundColor: 'rgba(255, 255, 255, 0.25)',
    paddingHorizontal: 12,
    paddingVertical: 6,
    borderRadius: 6,
    borderWidth: 1,
    borderColor: 'rgba(255, 255, 255, 0.4)',
    alignItems: 'center',
    justifyContent: 'center',
  },
  retryButtonContent: {
    flexDirection: 'row',
    alignItems: 'center',
  },
  retryIcon: {
    marginRight: 4,
  },
  retryButtonText: {
    color: '#FFFFFF',
    fontSize: 12,
    fontWeight: 'bold',
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
