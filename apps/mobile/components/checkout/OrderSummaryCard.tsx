import React from 'react';
import { View, Text, StyleSheet } from 'react-native';
import Colors from '@/constants/Colors';
import { formatUsdc } from '@/services/ticketPaymentContract';
import { computeOrderTotals } from '@/lib/pricing';

interface OrderSummaryCardProps {
  eventTitle: string;
  tierName: string;
  quantity: number;
  unitPriceUsdc: number;
}

/**
 * Detailed pre-purchase summary required by issue #1005's acceptance
 * criteria: ticket count, unit price, platform fees, and total USDC cost.
 */
export default function OrderSummaryCard({
  eventTitle,
  tierName,
  quantity,
  unitPriceUsdc,
}: OrderSummaryCardProps) {
  const totals = computeOrderTotals({ unitPriceUsdc, quantity });

  return (
    <View style={styles.card}>
      <Text style={styles.eventTitle}>{eventTitle}</Text>
      <Text style={styles.tierName}>{tierName}</Text>

      <View style={styles.divider} />

      <SummaryRow
        label={`Unit price × ${quantity}`}
        value={`${formatUsdc(unitPriceUsdc)} USDC`}
      />
      <SummaryRow label="Ticket count" value={String(quantity)} />
      <SummaryRow label="Subtotal" value={`${formatUsdc(totals.subtotalUsdc)} USDC`} />
      <SummaryRow
        label="Est. platform fee (from organizer payout)"
        value={`${formatUsdc(totals.estimatedPlatformFeeUsdc)} USDC`}
        muted
      />

      <View style={styles.divider} />

      <SummaryRow
        label="Total due"
        value={`${formatUsdc(totals.totalUsdc)} USDC`}
        emphasize
      />
    </View>
  );
}

function SummaryRow({
  label,
  value,
  emphasize,
  muted,
}: {
  label: string;
  value: string;
  emphasize?: boolean;
  muted?: boolean;
}) {
  return (
    <View style={styles.row}>
      <Text
        style={[styles.rowLabel, emphasize && styles.rowLabelEmphasis, muted && styles.rowLabelMuted]}
        numberOfLines={2}
      >
        {label}
      </Text>
      <Text style={[styles.rowValue, emphasize && styles.rowValueEmphasis, muted && styles.rowValueMuted]}>
        {value}
      </Text>
    </View>
  );
}

const styles = StyleSheet.create({
  card: {
    backgroundColor: '#1E1E20',
    borderRadius: 12,
    borderWidth: 1,
    borderColor: '#2C2C2E',
    padding: 18,
  },
  eventTitle: {
    color: Colors.primaryText,
    fontSize: 17,
    fontWeight: '700',
  },
  tierName: {
    color: Colors.secondaryText,
    fontSize: 13,
    marginTop: 4,
  },
  divider: {
    height: 1,
    backgroundColor: '#2C2C2E',
    marginVertical: 14,
  },
  row: {
    flexDirection: 'row',
    justifyContent: 'space-between',
    alignItems: 'flex-start',
    marginBottom: 10,
    gap: 12,
  },
  rowLabel: {
    color: Colors.primaryText,
    fontSize: 14,
    flexShrink: 1,
  },
  rowLabelMuted: {
    color: Colors.secondaryText,
    fontSize: 12,
  },
  rowLabelEmphasis: {
    fontSize: 16,
    fontWeight: '700',
  },
  rowValue: {
    color: Colors.primaryText,
    fontSize: 14,
    fontWeight: '600',
  },
  rowValueMuted: {
    color: Colors.secondaryText,
    fontSize: 12,
    fontWeight: '500',
  },
  rowValueEmphasis: {
    color: Colors.primaryYellow,
    fontSize: 18,
    fontWeight: '800',
  },
});
