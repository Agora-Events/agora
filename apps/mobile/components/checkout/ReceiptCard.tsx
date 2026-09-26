import React from 'react';
import { View, Text, Pressable, StyleSheet } from 'react-native';
import * as Clipboard from 'expo-clipboard';
import Colors from '@/constants/Colors';
import { formatUsdc } from '@/services/ticketPaymentContract';
import { STELLAR_EXPERT_BASE } from '@/services/staking';
import type { CheckoutReceipt } from '@/types/checkout';

interface ReceiptCardProps {
  receipt: CheckoutReceipt;
}

export default function ReceiptCard({ receipt }: ReceiptCardProps) {
  const completedAtLabel = formatTimestamp(receipt.completedAt);

  return (
    <View style={styles.card}>
      <View style={styles.successBadge}>
        <Text style={styles.successBadgeText}>✓ Purchase Confirmed</Text>
      </View>

      <Text style={styles.eventTitle}>{receipt.eventTitle}</Text>
      <Text style={styles.tierName}>
        {receipt.tierName} × {receipt.quantity}
      </Text>

      <View style={styles.divider} />

      <Row label="Ticket ID" value={receipt.ticketId} monospace />
      <Row label="Payment ID" value={receipt.paymentId} monospace />
      <Row label="Purchased" value={completedAtLabel} />
      <Row label="Buyer wallet" value={truncateAddress(receipt.buyerPublicKey)} monospace />

      <View style={styles.divider} />

      <Row label={`Unit price × ${receipt.quantity}`} value={`${formatUsdc(receipt.unitPriceUsdc)} USDC`} />
      <Row
        label="Est. platform fee (from organizer payout)"
        value={`${formatUsdc(receipt.platformFeeUsdc)} USDC`}
        muted
      />
      <Row label="Total paid" value={`${formatUsdc(receipt.totalPaidUsdc)} USDC`} emphasize />

      <View style={styles.divider} />

      <TxHashRow label="Approval tx" hash={receipt.approvalTxHash} />
      <TxHashRow label="Payment tx" hash={receipt.paymentTxHash} />
    </View>
  );
}

function Row({
  label,
  value,
  monospace,
  emphasize,
  muted,
}: {
  label: string;
  value: string;
  monospace?: boolean;
  emphasize?: boolean;
  muted?: boolean;
}) {
  return (
    <View style={styles.row}>
      <Text style={[styles.rowLabel, muted && styles.rowLabelMuted]}>{label}</Text>
      <Text
        style={[
          styles.rowValue,
          monospace && styles.monospace,
          emphasize && styles.rowValueEmphasis,
          muted && styles.rowValueMuted,
        ]}
        numberOfLines={1}
      >
        {value}
      </Text>
    </View>
  );
}

function TxHashRow({ label, hash }: { label: string; hash: string }) {
  const [copied, setCopied] = React.useState(false);

  const handleCopy = async () => {
    await Clipboard.setStringAsync(hash);
    setCopied(true);
    setTimeout(() => setCopied(false), 1500);
  };

  return (
    <Pressable
      testID={`tx-row-${label.toLowerCase().replace(/\s+/g, '-')}`}
      onPress={handleCopy}
      style={styles.txRow}
      accessibilityRole="button"
    >
      <View style={styles.txTextContainer}>
        <Text style={styles.rowLabel}>{label}</Text>
        <Text style={styles.txHash} numberOfLines={1}>
          {truncateHash(hash)}
        </Text>
        <Text style={styles.explorerLink}>
          {`${STELLAR_EXPERT_BASE}/${hash}`}
        </Text>
      </View>
      <Text style={styles.copyLabel}>{copied ? 'Copied!' : 'Copy'}</Text>
    </Pressable>
  );
}

function truncateHash(hash: string): string {
  if (hash.length <= 16) return hash;
  return `${hash.slice(0, 8)}...${hash.slice(-8)}`;
}

function truncateAddress(address: string): string {
  if (address.length <= 12) return address;
  return `${address.slice(0, 6)}...${address.slice(-6)}`;
}

function formatTimestamp(iso: string): string {
  try {
    return new Date(iso).toLocaleString();
  } catch {
    return iso;
  }
}

const styles = StyleSheet.create({
  card: {
    backgroundColor: '#1E1E20',
    borderRadius: 14,
    borderWidth: 1,
    borderColor: '#2C2C2E',
    padding: 20,
  },
  successBadge: {
    alignSelf: 'flex-start',
    backgroundColor: '#34C75922',
    borderRadius: 6,
    paddingHorizontal: 10,
    paddingVertical: 5,
    marginBottom: 14,
  },
  successBadgeText: {
    color: '#34C759',
    fontSize: 12,
    fontWeight: '700',
  },
  eventTitle: {
    color: Colors.primaryText,
    fontSize: 19,
    fontWeight: '800',
  },
  tierName: {
    color: Colors.secondaryText,
    fontSize: 13,
    marginTop: 4,
  },
  divider: {
    height: 1,
    backgroundColor: '#2C2C2E',
    marginVertical: 16,
  },
  row: {
    flexDirection: 'row',
    justifyContent: 'space-between',
    alignItems: 'center',
    marginBottom: 10,
    gap: 12,
  },
  rowLabel: {
    color: Colors.secondaryText,
    fontSize: 12,
    flexShrink: 1,
  },
  rowLabelMuted: {
    fontSize: 11,
  },
  rowValue: {
    color: Colors.primaryText,
    fontSize: 13,
    fontWeight: '600',
    flexShrink: 1,
    textAlign: 'right',
  },
  rowValueMuted: {
    color: Colors.secondaryText,
    fontWeight: '500',
  },
  rowValueEmphasis: {
    color: Colors.primaryYellow,
    fontSize: 17,
    fontWeight: '800',
  },
  monospace: {
    fontFamily: 'SpaceMono',
    fontSize: 12,
  },
  txRow: {
    flexDirection: 'row',
    justifyContent: 'space-between',
    alignItems: 'center',
    paddingVertical: 8,
    gap: 12,
  },
  txTextContainer: {
    flex: 1,
  },
  txHash: {
    color: Colors.primaryText,
    fontFamily: 'SpaceMono',
    fontSize: 12,
    marginTop: 2,
  },
  explorerLink: {
    color: Colors.primaryYellow,
    fontSize: 10,
    marginTop: 2,
  },
  copyLabel: {
    color: Colors.primaryYellow,
    fontSize: 12,
    fontWeight: '700',
  },
});
