import React, { useMemo } from 'react';
import { View, Text, ScrollView, StyleSheet } from 'react-native';
import { useLocalSearchParams, useRouter } from 'expo-router';
import Colors from '@/constants/Colors';
import Button from '@/components/ui/Button';
import ReceiptCard from '@/components/checkout/ReceiptCard';
import type { CheckoutReceipt } from '@/types/checkout';

function parseReceipt(raw: string | string[] | undefined): CheckoutReceipt | null {
  if (!raw || Array.isArray(raw)) return null;
  try {
    const parsed = JSON.parse(raw);
    if (parsed && typeof parsed === 'object' && parsed.ticketId) {
      return parsed as CheckoutReceipt;
    }
    return null;
  } catch {
    return null;
  }
}

/**
 * Completion / receipt screen navigated to on a successful checkout — the
 * last acceptance criterion of issue #1005.
 */
export default function CheckoutCompleteScreen() {
  const params = useLocalSearchParams<{ receipt?: string }>();
  const router = useRouter();

  const receipt = useMemo(() => parseReceipt(params.receipt), [params.receipt]);

  if (!receipt) {
    return (
      <View style={styles.emptyContainer}>
        <Text style={styles.emptyText}>We couldn't find your receipt.</Text>
        <Button
          testID="checkout-complete-back-to-events"
          title="Back to Events"
          onPress={() => router.replace('/(tabs)/discover')}
          style={styles.emptyButton}
        />
      </View>
    );
  }

  return (
    <ScrollView style={styles.container} contentContainerStyle={styles.content}>
      <View style={styles.iconCircle}>
        <Text style={styles.iconCheck}>✓</Text>
      </View>

      <Text style={styles.heading}>You're going!</Text>
      <Text style={styles.subheading}>
        {receipt.quantity > 1
          ? `Your ${receipt.quantity} tickets have been secured on-chain.`
          : 'Your ticket has been secured on-chain.'}
      </Text>

      <ReceiptCard receipt={receipt} />

      <View style={styles.actions}>
        <Button
          testID="checkout-complete-view-tickets"
          title="View My Tickets"
          onPress={() => router.replace({ pathname: '/ticket/[id]', params: { id: receipt.ticketId } })}
          style={styles.actionButton}
        />
        <Button
          testID="checkout-complete-back-to-events"
          title="Back to Events"
          variant="secondary"
          onPress={() => router.replace('/(tabs)/discover')}
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
    padding: 20,
    paddingBottom: 48,
    alignItems: 'stretch',
  },
  iconCircle: {
    alignSelf: 'center',
    width: 64,
    height: 64,
    borderRadius: 32,
    backgroundColor: '#34C75922',
    justifyContent: 'center',
    alignItems: 'center',
    marginTop: 12,
    marginBottom: 20,
  },
  iconCheck: {
    color: '#34C759',
    fontSize: 32,
    fontWeight: '900',
  },
  heading: {
    fontSize: 24,
    fontWeight: '800',
    color: Colors.primaryText,
    textAlign: 'center',
  },
  subheading: {
    fontSize: 14,
    color: Colors.secondaryText,
    textAlign: 'center',
    marginTop: 8,
    marginBottom: 24,
  },
  actions: {
    marginTop: 24,
    gap: 12,
  },
  actionButton: {
    width: '100%',
  },
  emptyContainer: {
    flex: 1,
    backgroundColor: Colors.darkBackground,
    justifyContent: 'center',
    alignItems: 'center',
    padding: 24,
  },
  emptyText: {
    color: Colors.secondaryText,
    fontSize: 15,
    marginBottom: 20,
    textAlign: 'center',
  },
  emptyButton: {
    width: '100%',
  },
});
