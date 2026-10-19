import React, { useState } from 'react';
import { View, Text, StyleSheet, Alert } from 'react-native';
import { useLocalSearchParams, useRouter } from 'expo-router';
import Colors from '@/constants/Colors';
import Button from '@/components/ui/Button';

export default function CheckoutScreen() {
  const params = useLocalSearchParams();
  const router = useRouter();
  const [loading, setLoading] = useState(false);

  const eventTitle = params.eventTitle || 'Stellar Meridian 2026';
  const price = '150 XLM';

  const handleCheckout = () => {
    setLoading(true);
    setTimeout(() => {
      setLoading(false);
      Alert.alert(
        'Success',
        'Ticket purchased successfully! Your ticket is now available in the Tickets tab.',
        [{ text: 'OK', onPress: () => router.dismiss() }]
      );
    }, 1500);
  };

  return (
    <View style={styles.container}>
      <View style={styles.content}>
        <View style={styles.ticketSummary}>
          <Text style={styles.label}>Event</Text>
          <Text style={styles.titleText}>{eventTitle}</Text>
          
          <View style={styles.divider} />
          
          <View style={styles.row}>
            <Text style={styles.metaLabel}>Price</Text>
            <Text style={styles.priceText}>{price}</Text>
          </View>
          <View style={styles.row}>
            <Text style={styles.metaLabel}>Fee</Text>
            <Text style={styles.metaValue}>0.1 XLM</Text>
          </View>
          
          <View style={styles.divider} />
          
          <View style={styles.row}>
            <Text style={styles.totalLabel}>Total</Text>
            <Text style={styles.totalValue}>150.1 XLM</Text>
          </View>
        </View>

        <Button
          title={loading ? 'Processing...' : 'Confirm and Pay'}
          onPress={handleCheckout}
          loading={loading}
          style={styles.payButton}
        />
      </View>
    </View>
  );
}

const styles = StyleSheet.create({
  container: {
    flex: 1,
    backgroundColor: Colors.darkBackground,
    padding: 24,
  },
  content: {
    flex: 1,
    justifyContent: 'space-between',
  },
  ticketSummary: {
    backgroundColor: '#1E1E20',
    borderRadius: 12,
    padding: 20,
    borderWidth: 1,
    borderColor: '#2C2C2E',
    marginTop: 20,
  },
  label: {
    color: Colors.secondaryText,
    fontSize: 12,
    fontWeight: '600',
    textTransform: 'uppercase',
    marginBottom: 6,
  },
  titleText: {
    color: Colors.primaryText,
    fontSize: 20,
    fontWeight: 'bold',
  },
  divider: {
    height: 1,
    backgroundColor: '#2C2C2E',
    marginVertical: 16,
  },
  row: {
    flexDirection: 'row',
    justifyContent: 'space-between',
    marginVertical: 4,
  },
  metaLabel: {
    color: Colors.secondaryText,
    fontSize: 14,
  },
  metaValue: {
    color: Colors.primaryText,
    fontSize: 14,
    fontWeight: '500',
  },
  priceText: {
    color: Colors.primaryYellow,
    fontSize: 14,
    fontWeight: 'bold',
  },
  totalLabel: {
    color: Colors.primaryText,
    fontSize: 16,
    fontWeight: 'bold',
  },
  totalValue: {
    color: Colors.primaryYellow,
    fontSize: 18,
    fontWeight: 'bold',
  },
  payButton: {
    marginBottom: 30,
  },
});
