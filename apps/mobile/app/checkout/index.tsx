import React, { useMemo, useState } from 'react';
import { View, Text, ScrollView, StyleSheet, Alert } from 'react-native';
import { useLocalSearchParams, useRouter } from 'expo-router';
import Colors from '@/constants/Colors';
import Button from '@/components/ui/Button';
import TierSelector from '@/components/checkout/TierSelector';
import QuantityStepper from '@/components/checkout/QuantityStepper';
import OrderSummaryCard from '@/components/checkout/OrderSummaryCard';
import CheckoutProgressModal from '@/components/checkout/CheckoutProgressModal';
import { useAuth } from '@/hooks/useAuth';
import { useTicketCheckout } from '@/hooks/useTicketCheckout';
import type { TicketTierOption } from '@/types/checkout';

const MAX_TICKETS_PER_ORDER = 10;

/**
 * Ticket tier catalogue keyed by event id. The event/pricing endpoints this
 * would normally come from aren't wired up in the mobile client yet, so this
 * mirrors the mock-data convention already used by `app/event/[id].tsx` and
 * `app/ticket/[id].tsx` until a real `/api/events/:id/tiers` call replaces it.
 */
const MOCK_TIERS_BY_EVENT: Record<string, TicketTierOption[]> = {
  '1': [
    { id: 'tier-ga', name: 'General Admission', priceUsdc: 150, remaining: 340 },
    { id: 'tier-vip', name: 'VIP', description: 'Front-row seating + backstage pass', priceUsdc: 450, remaining: 12 },
  ],
  '2': [
    { id: 'tier-ga', name: 'General Admission', priceUsdc: 80, remaining: 500 },
    { id: 'tier-pro', name: 'Pro Pass', description: 'Includes workshop access', priceUsdc: 220, remaining: 45 },
  ],
  '3': [
    { id: 'tier-ga', name: 'General Admission', priceUsdc: 60, remaining: 800 },
    { id: 'tier-early', name: 'Early Bird', priceUsdc: 45, remaining: 0 },
  ],
};

const DEFAULT_TIERS: TicketTierOption[] = [
  { id: 'tier-ga', name: 'General Admission', priceUsdc: 100, remaining: 200 },
];

function getTiersForEvent(eventId: string | undefined): TicketTierOption[] {
  if (!eventId) return DEFAULT_TIERS;
  return MOCK_TIERS_BY_EVENT[eventId] ?? DEFAULT_TIERS;
}

export default function CheckoutScreen() {
  const params = useLocalSearchParams<{ eventId?: string; eventTitle?: string }>();
  const router = useRouter();
  const { user } = useAuth();
  const checkout = useTicketCheckout();

  const eventId = params.eventId ?? '1';
  const eventTitle = params.eventTitle ?? 'Agora Event';

  const tiers = useMemo(() => getTiersForEvent(eventId), [eventId]);
  const firstAvailableTier = tiers.find((t) => t.remaining !== 0) ?? tiers[0];

  const [selectedTierId, setSelectedTierId] = useState<string>(firstAvailableTier?.id ?? '');
  const [quantity, setQuantity] = useState(1);

  const selectedTier = tiers.find((t) => t.id === selectedTierId) ?? null;
  const maxQuantity = selectedTier?.remaining != null
    ? Math.max(1, Math.min(MAX_TICKETS_PER_ORDER, selectedTier.remaining))
    : MAX_TICKETS_PER_ORDER;

  const isModalVisible = checkout.phase === 'in-progress' || checkout.phase === 'error';

  const handleTierSelect = (tierId: string) => {
    setSelectedTierId(tierId);
    setQuantity(1);
  };

  const handleConfirm = async () => {
    if (!selectedTier) {
      Alert.alert('Select a ticket tier', 'Please choose a ticket tier before continuing.');
      return;
    }
    if (!user?.walletAddress || user.walletAddress === 'GDAGORA...') {
      Alert.alert(
        'Wallet required',
        'Set up your Stellar wallet in Settings before purchasing tickets.'
      );
      return;
    }

    await checkout.startCheckout({
      eventId,
      eventTitle,
      tierId: selectedTier.id,
      tierName: selectedTier.name,
      unitPriceUsdc: selectedTier.priceUsdc,
      quantity,
      buyerPublicKey: user.walletAddress,
    });
  };

  // Navigate to the receipt screen once the hook reports success.
  React.useEffect(() => {
    if (checkout.phase === 'success' && checkout.receipt) {
      router.replace({
        pathname: '/checkout/complete',
        params: { receipt: JSON.stringify(checkout.receipt) },
      });
      checkout.reset();
    }
  }, [checkout.phase, checkout.receipt]); // eslint-disable-line react-hooks/exhaustive-deps

  return (
    <ScrollView style={styles.container} contentContainerStyle={styles.content}>
      <Text style={styles.heading}>Select Tickets</Text>
      <Text style={styles.subheading}>{eventTitle}</Text>

      <View style={styles.section}>
        <Text style={styles.sectionLabel}>Ticket Tier</Text>
        <TierSelector
          tiers={tiers}
          selectedTierId={selectedTierId}
          onSelect={handleTierSelect}
          disabled={checkout.isSubmitting}
        />
      </View>

      {selectedTier ? (
        <View style={styles.section}>
          <View style={styles.quantityRow}>
            <Text style={styles.sectionLabel}>Quantity</Text>
            <QuantityStepper
              value={quantity}
              onChange={setQuantity}
              min={1}
              max={maxQuantity}
              disabled={checkout.isSubmitting}
            />
          </View>
        </View>
      ) : null}

      {selectedTier ? (
        <View style={styles.section}>
          <Text style={styles.sectionLabel}>Order Summary</Text>
          <OrderSummaryCard
            eventTitle={eventTitle}
            tierName={selectedTier.name}
            quantity={quantity}
            unitPriceUsdc={selectedTier.priceUsdc}
          />
        </View>
      ) : null}

      <Button
        testID="checkout-confirm-button"
        title={checkout.isSubmitting ? 'Processing...' : 'Confirm & Pay with USDC'}
        onPress={handleConfirm}
        disabled={!selectedTier || checkout.isSubmitting}
        loading={checkout.isSubmitting}
        style={styles.confirmButton}
      />

      <Text style={styles.disclaimer}>
        You will be asked to approve a USDC spending allowance, then submit the ticket purchase.
        Both transactions run on Stellar Testnet via {'\n'}soroban-testnet.stellar.org.
      </Text>

      <CheckoutProgressModal
        visible={isModalVisible}
        steps={checkout.steps}
        phase={checkout.phase}
        errorMessage={checkout.errorMessage}
        onRetry={handleConfirm}
        onDismiss={checkout.reset}
      />
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
  },
  heading: {
    fontSize: 22,
    fontWeight: '700',
    color: Colors.primaryText,
  },
  subheading: {
    fontSize: 14,
    color: Colors.secondaryText,
    marginTop: 4,
    marginBottom: 24,
  },
  section: {
    marginBottom: 24,
  },
  sectionLabel: {
    fontSize: 13,
    fontWeight: '700',
    textTransform: 'uppercase',
    letterSpacing: 0.5,
    color: Colors.secondaryText,
    marginBottom: 10,
  },
  quantityRow: {
    flexDirection: 'row',
    justifyContent: 'space-between',
    alignItems: 'center',
  },
  confirmButton: {
    marginTop: 4,
  },
  disclaimer: {
    color: Colors.secondaryText,
    fontSize: 11,
    textAlign: 'center',
    marginTop: 16,
    lineHeight: 16,
  },
});
