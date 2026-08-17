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
import { useLiveTicketInventory } from '@/hooks/useLiveTicketInventory';
import { getTiersForEvent } from '@/lib/ticketTiers';

const MAX_TICKETS_PER_ORDER = 10;

export default function CheckoutScreen() {
  const params = useLocalSearchParams<{
    eventId?: string;
    eventTitle?: string;
    /** Signed checkout access grant issued by the waiting room (Issue #1187). */
    grantToken?: string;
  }>();
  const router = useRouter();
  const { user, token } = useAuth();
  const checkout = useTicketCheckout();

  const eventId = params.eventId ?? '1';
  const eventTitle = params.eventTitle ?? 'Agora Event';
  // Received when the user was admitted through the virtual waiting room.
  // Server-side checkout endpoints can verify it with the waiting-room API.
  const grantToken = params.grantToken;

  const baseTiers = useMemo(() => getTiersForEvent(eventId), [eventId]);

  // Live inventory: `tiers` is `baseTiers` with any remaining counts pushed
  // by the server applied on top (issue #1010).
  const { tiers, isLive } = useLiveTicketInventory({
    eventId,
    tiers: baseTiers,
    token: token ?? undefined,
  });

  const firstAvailableTier = baseTiers.find((t) => t.remaining !== 0) ?? baseTiers[0];

  const [selectedTierId, setSelectedTierId] = useState<string>(firstAvailableTier?.id ?? '');
  const [quantity, setQuantity] = useState(1);

  const selectedTier = tiers.find((t) => t.id === selectedTierId) ?? null;
  const isSelectedSoldOut = selectedTier?.remaining === 0;
  const maxQuantity = selectedTier?.remaining != null
    ? Math.max(1, Math.min(MAX_TICKETS_PER_ORDER, selectedTier.remaining))
    : MAX_TICKETS_PER_ORDER;

  // A tier can sell out while the user is choosing a quantity; clamp rather
  // than letting them submit an order larger than what is left.
  React.useEffect(() => {
    setQuantity((current) => Math.min(current, maxQuantity));
  }, [maxQuantity]);

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
    if (isSelectedSoldOut) {
      Alert.alert('Tier sold out', 'This ticket tier just sold out. Please choose another tier.');
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
      grantToken,
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

      {isLive ? (
        <Text testID="live-inventory-indicator" style={styles.liveBadge}>
          ● Live availability
        </Text>
      ) : null}

      {isSelectedSoldOut ? (
        <Text testID="tier-sold-out-notice" style={styles.soldOutNotice}>
          This tier just sold out. Choose another tier to continue.
        </Text>
      ) : null}

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
              disabled={checkout.isSubmitting || isSelectedSoldOut}
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
        title={
          isSelectedSoldOut
            ? 'Sold Out'
            : checkout.isSubmitting
              ? 'Processing...'
              : 'Confirm & Pay with USDC'
        }
        onPress={handleConfirm}
        disabled={!selectedTier || checkout.isSubmitting || isSelectedSoldOut}
        loading={checkout.isSubmitting}
        style={styles.confirmButton}
      />

      <Button
        testID="checkout-queue-link"
        title="High demand? Join the virtual queue"
        variant="outline"
        onPress={() =>
          router.push({
            pathname: '/checkout/waiting-room',
            params: { eventId, eventTitle },
          })
        }
        disabled={checkout.isSubmitting}
        style={styles.queueLink}
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
  queueLink: {
    marginTop: 12,
  },
  liveBadge: {
    fontSize: 11,
    color: Colors.primaryText,
    marginTop: -18,
    marginBottom: 18,
  },
  soldOutNotice: {
    fontSize: 12,
    color: Colors.primaryText,
    marginBottom: 16,
  },
  disclaimer: {
    color: Colors.secondaryText,
    fontSize: 11,
    textAlign: 'center',
    marginTop: 16,
    lineHeight: 16,
  },
});
