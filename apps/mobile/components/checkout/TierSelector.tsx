import React from 'react';
import { View, Text, Pressable, StyleSheet } from 'react-native';
import Colors from '@/constants/Colors';
import { formatUsdc } from '@/services/ticketPaymentContract';
import type { TicketTierOption } from '@/types/checkout';

interface TierSelectorProps {
  tiers: TicketTierOption[];
  selectedTierId: string | null;
  onSelect: (tierId: string) => void;
  disabled?: boolean;
}

export default function TierSelector({
  tiers,
  selectedTierId,
  onSelect,
  disabled = false,
}: TierSelectorProps) {
  return (
    <View style={styles.list}>
      {tiers.map((tier) => {
        const isSelected = tier.id === selectedTierId;
        const isSoldOut = tier.remaining === 0;

        return (
          <Pressable
            key={tier.id}
            testID={`tier-card-${tier.id}`}
            accessibilityRole="button"
            accessibilityState={{ selected: isSelected, disabled: disabled || isSoldOut }}
            disabled={disabled || isSoldOut}
            onPress={() => onSelect(tier.id)}
            style={({ pressed }) => [
              styles.card,
              isSelected && styles.cardSelected,
              isSoldOut && styles.cardDisabled,
              pressed && !isSoldOut && styles.cardPressed,
            ]}
          >
            <View style={styles.cardMain}>
              <View style={styles.cardHeaderRow}>
                <Text style={styles.tierName}>{tier.name}</Text>
                {isSelected && <View style={styles.selectedDot} />}
              </View>
              {tier.description ? (
                <Text style={styles.tierDescription}>{tier.description}</Text>
              ) : null}
              {typeof tier.remaining === 'number' && (
                <Text style={styles.remainingText}>
                  {isSoldOut ? 'Sold out' : `${tier.remaining} remaining`}
                </Text>
              )}
            </View>
            <Text style={styles.tierPrice}>
              {tier.priceUsdc === 0 ? 'Free' : `${formatUsdc(tier.priceUsdc)} USDC`}
            </Text>
          </Pressable>
        );
      })}
    </View>
  );
}

const styles = StyleSheet.create({
  list: {
    gap: 10,
  },
  card: {
    flexDirection: 'row',
    justifyContent: 'space-between',
    alignItems: 'center',
    backgroundColor: '#1E1E20',
    borderRadius: 12,
    borderWidth: 1,
    borderColor: '#2C2C2E',
    padding: 14,
  },
  cardSelected: {
    borderColor: Colors.primaryYellow,
    backgroundColor: '#242019',
  },
  cardDisabled: {
    opacity: 0.4,
  },
  cardPressed: {
    opacity: 0.85,
  },
  cardMain: {
    flex: 1,
    paddingRight: 12,
  },
  cardHeaderRow: {
    flexDirection: 'row',
    alignItems: 'center',
    gap: 8,
  },
  tierName: {
    color: Colors.primaryText,
    fontSize: 15,
    fontWeight: '700',
  },
  selectedDot: {
    width: 8,
    height: 8,
    borderRadius: 4,
    backgroundColor: Colors.primaryYellow,
  },
  tierDescription: {
    color: Colors.secondaryText,
    fontSize: 12,
    marginTop: 4,
  },
  remainingText: {
    color: Colors.secondaryText,
    fontSize: 11,
    marginTop: 6,
  },
  tierPrice: {
    color: Colors.primaryYellow,
    fontSize: 15,
    fontWeight: '700',
  },
});
