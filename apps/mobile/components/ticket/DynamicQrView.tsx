/**
 * DynamicQrView.tsx — Issue #1179: Offline Ticket Vault
 *
 * Renders a rotating QR code for offline ticket verification. The payload is
 * regenerated every PAYLOAD_WINDOW_S seconds via `generateRotatingPayload()`
 * from `lib/crypto.ts`.
 *
 * Uses react-native-qrcode-svg (backed by react-native-svg) to render the
 * QR code natively on iOS and Android.
 *
 * The component is intentionally dumb: it receives the secretKey as a prop
 * rather than reading from SecureStore directly, so the parent screen
 * controls vault access and biometric gating.
 *
 * ## Anti-screenshot design note
 *
 * The payload rotates every PAYLOAD_WINDOW_S (15s). A screenshot becomes
 * invalid within PAYLOAD_WINDOW_S + SCANNER_CLOCK_DRIFT_S (75s at most) from
 * the time it was taken, because the scanner enforces a ±60s timestamp window
 * around the payload's own window boundary. Gate staff should be instructed to
 * reject QR codes that do not visibly animate/refresh.
 */

import React, { useState, useEffect, useRef } from 'react';
import { View, Text, StyleSheet, ActivityIndicator } from 'react-native';
import QRCode from 'react-native-qrcode-svg';
import { generateRotatingPayload, PAYLOAD_WINDOW_S } from '../lib/crypto';
import Colors from '@/constants/Colors';

// Refresh slightly more often than the window so the display never lags
// behind by a full window when a window boundary is crossed.
const REFRESH_INTERVAL_MS = (PAYLOAD_WINDOW_S * 1000) / 2; // every 7.5s

export interface DynamicQrViewProps {
  /**
   * Ticket identifier to embed in the payload (max 16 UTF-8 bytes).
   * Longer strings are truncated at the crypto layer.
   */
  ticketId: string;
  /**
   * 64-byte Ed25519 secret key derived from the purchase secret.
   * The parent is responsible for biometric-gating access to this value.
   */
  secretKey: Uint8Array;
  /**
   * Called each time a new payload is generated. Useful for testing and for
   * parent screens that want to display the payload text alongside the QR.
   */
  onPayloadChange?: (payload: string) => void;
}

export default function DynamicQrView({
  ticketId,
  secretKey,
  onPayloadChange,
}: DynamicQrViewProps) {
  const [payload, setPayload] = useState<string | null>(null);
  const [error, setError] = useState<string | null>(null);
  const intervalRef = useRef<ReturnType<typeof setInterval> | null>(null);

  const regenerate = () => {
    try {
      const next = generateRotatingPayload(ticketId, secretKey, Date.now());
      setPayload(next);
      setError(null);
      onPayloadChange?.(next);
    } catch (err) {
      setError(err instanceof Error ? err.message : 'Failed to generate payload.');
    }
  };

  useEffect(() => {
    regenerate();
    intervalRef.current = setInterval(regenerate, REFRESH_INTERVAL_MS);
    return () => {
      if (intervalRef.current) clearInterval(intervalRef.current);
    };
    // eslint-disable-next-line react-hooks/exhaustive-deps -- secretKey is a Uint8Array ref; deps intentionally include only stable values
  }, [ticketId, secretKey]);

  if (error) {
    return (
      <View style={styles.container} accessibilityRole="alert">
        <Text style={styles.errorText}>{error}</Text>
      </View>
    );
  }

  if (!payload) {
    return (
      <View style={styles.container}>
        <ActivityIndicator
          size="large"
          color={Colors.primaryYellow}
          accessibilityLabel="Generating QR code…"
        />
      </View>
    );
  }

  return (
    <View style={styles.container} accessibilityRole="image" accessibilityLabel="Ticket QR code">
      <View style={styles.qrWrapper}>
        <QRCode
          value={payload}
          size={240}
          backgroundColor="#FFFFFF"
          color="#000000"
          // Medium error correction (M = ~15% data recovery) balances density
          // and scan reliability; the ~140-char payload fits comfortably.
          ecl="M"
        />
      </View>

      <Text style={styles.refreshHint} accessibilityLiveRegion="polite">
        Refreshes every {PAYLOAD_WINDOW_S}s
      </Text>
    </View>
  );
}

const styles = StyleSheet.create({
  container: {
    alignItems: 'center',
    padding: 20,
    borderWidth: 1,
    borderColor: '#2C2C2E',
    borderRadius: 12,
    backgroundColor: '#1E1E20',
  },
  qrWrapper: {
    padding: 12,
    backgroundColor: '#FFFFFF',
    borderRadius: 8,
    marginBottom: 12,
  },
  refreshHint: {
    fontSize: 11,
    color: Colors.secondaryText,
    marginTop: 4,
    opacity: 0.6,
  },
  errorText: {
    color: '#FF3B30',
    fontSize: 13,
    textAlign: 'center',
  },
});
