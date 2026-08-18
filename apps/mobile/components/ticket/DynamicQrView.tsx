/**
 * DynamicQrView.tsx — Issue #1179: Offline Ticket Vault
 *
 * Renders a rotating QR code for offline ticket verification. The payload is
 * regenerated every PAYLOAD_WINDOW_S seconds via `generateRotatingPayload()`
 * from `lib/crypto.ts`.
 *
 * The QR library is not yet installed in this app (no QR library in
 * package.json at the time of this feature). This component renders the
 * base64url payload string directly for now, with a clearly marked TODO to
 * swap in a QR code rendering library (e.g. react-native-qrcode-svg) once
 * one is added to the project.
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
      {/*
       * TODO: Replace this placeholder with a proper QR code rendering component
       * once a QR library is added to the project (e.g. react-native-qrcode-svg).
       *
       * The payload string is a base64url-encoded binary ticket proof ~140
       * characters long, well within QR code capacity even at medium error
       * correction.
       *
       * Example drop-in replacement:
       *
       *   import QRCode from 'react-native-qrcode-svg';
       *   <QRCode value={payload} size={240} />
       */}
      <View style={styles.qrPlaceholderBox} accessibilityHidden>
        <Text style={styles.qrPlaceholderLabel}>QR Code</Text>
        <Text style={styles.qrPlaceholderSub}>(QR library not yet installed)</Text>
      </View>

      {/* Payload string — shown for development/debugging and for gate staff
          on devices that cannot render the QR. In production this should be
          hidden or shown only on request. */}
      <Text
        style={styles.payloadText}
        numberOfLines={3}
        accessibilityLabel="Ticket payload"
      >
        {payload}
      </Text>

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
  qrPlaceholderBox: {
    width: 200,
    height: 200,
    backgroundColor: '#2C2C2E',
    borderRadius: 8,
    alignItems: 'center',
    justifyContent: 'center',
    marginBottom: 12,
  },
  qrPlaceholderLabel: {
    color: Colors.primaryText,
    fontWeight: 'bold',
    fontSize: 16,
  },
  qrPlaceholderSub: {
    color: Colors.secondaryText,
    fontSize: 11,
    marginTop: 4,
    textAlign: 'center',
    paddingHorizontal: 8,
  },
  payloadText: {
    fontSize: 9,
    color: Colors.secondaryText,
    textAlign: 'center',
    marginTop: 4,
    fontFamily: 'SpaceMono',
  },
  refreshHint: {
    fontSize: 11,
    color: Colors.secondaryText,
    marginTop: 8,
    opacity: 0.6,
  },
  errorText: {
    color: '#FF3B30',
    fontSize: 13,
    textAlign: 'center',
  },
});
