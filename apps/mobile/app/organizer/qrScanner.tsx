/**
 * qrScanner.tsx
 *
 * Organizer-facing camera scanner for attendee ticket QR codes. Staff pick a
 * Check-in / Check-out mode, point the camera at a ticket's QR code, and get
 * immediate color + haptic feedback while the scan is verified against
 * `/api/v1/tickets/:id/scan`, which prevents double entries server-side.
 */

import React, { useCallback, useRef, useState } from 'react';
import { View, Text, StyleSheet, SafeAreaView, Pressable, ActivityIndicator } from 'react-native';
import { CameraView, useCameraPermissions } from 'expo-camera';
import * as Haptics from 'expo-haptics';
import Colors from '@/constants/Colors';
import Button from '@/components/ui/Button';
import { organizerApi } from '@/services/organizerApi';

/** How long a scan result stays on screen before the scanner re-arms. */
const RESULT_DISPLAY_MS = 2000;
/** Max age of a scanned payload's timestamp before it's treated as a replay. */
const REPLAY_THRESHOLD_MS = 30_000;

type CheckInMode = 'checkin' | 'checkout';
type ScanState = 'scanning' | 'processing' | 'success' | 'duplicate' | 'invalid' | 'expired' | 'error';

interface ScannedQrPayload {
  payload: {
    id: string;
    qr_type: string;
    data: { ticket_id?: string; [key: string]: unknown };
    created_at: string;
    expires_at: string;
    nonce: string;
  };
  signature: string;
  public_key: string;
}

/** Parses the raw QR text into the signed payload shape, or null if malformed. */
function parseScannedQr(raw: string): ScannedQrPayload | null {
  try {
    const parsed = JSON.parse(raw);
    if (
      parsed &&
      typeof parsed.signature === 'string' &&
      typeof parsed.public_key === 'string' &&
      parsed.payload &&
      typeof parsed.payload.id === 'string' &&
      typeof parsed.payload.created_at === 'string' &&
      parsed.payload.data
    ) {
      return parsed as ScannedQrPayload;
    }
    return null;
  } catch {
    return null;
  }
}

function isWithinReplayWindow(createdAt: string, nowMs: number): boolean {
  const createdMs = new Date(createdAt).getTime();
  if (Number.isNaN(createdMs)) return false;
  return Math.abs(nowMs - createdMs) <= REPLAY_THRESHOLD_MS;
}

export default function QrScannerScreen() {
  const [permission, requestPermission] = useCameraPermissions();
  const [mode, setMode] = useState<CheckInMode>('checkin');
  const [state, setState] = useState<ScanState>('scanning');
  const [message, setMessage] = useState<string>('Point the camera at a ticket QR code.');
  const lockedRef = useRef(false);

  const resolveScan = useCallback(async (next: ScanState, nextMessage: string) => {
    setState(next);
    setMessage(nextMessage);

    if (next === 'success') {
      await Haptics.notificationAsync(Haptics.NotificationFeedbackType.Success);
    } else if (next === 'duplicate' || next === 'invalid') {
      await Haptics.notificationAsync(Haptics.NotificationFeedbackType.Error);
    } else if (next === 'expired' || next === 'error') {
      await Haptics.notificationAsync(Haptics.NotificationFeedbackType.Warning);
    }

    setTimeout(() => {
      lockedRef.current = false;
      setState('scanning');
      setMessage('Point the camera at a ticket QR code.');
    }, RESULT_DISPLAY_MS);
  }, []);

  const processScan = useCallback(
    async (raw: string) => {
      const parsed = parseScannedQr(raw);
      if (!parsed) {
        await resolveScan('invalid', 'This QR code is not a valid Agora ticket.');
        return;
      }

      if (!isWithinReplayWindow(parsed.payload.created_at, Date.now())) {
        await resolveScan('expired', 'This QR code has expired. Ask the attendee to refresh it.');
        return;
      }

      const ticketId = parsed.payload.data.ticket_id;
      if (!ticketId) {
        await resolveScan('invalid', 'QR payload is missing a ticket id.');
        return;
      }

      try {
        const { status, message: apiMessage } = await organizerApi.scanTicket(ticketId, {
          payload: parsed.payload,
          signature: parsed.signature,
          public_key: parsed.public_key,
          mode,
        });

        if (status === 200) {
          await resolveScan('success', apiMessage);
        } else if (status === 409) {
          await resolveScan('duplicate', apiMessage);
        } else {
          await resolveScan('invalid', apiMessage);
        }
      } catch {
        await resolveScan('error', 'Could not reach the server. Check your connection and try again.');
      }
    },
    [mode, resolveScan]
  );

  const handleBarcodeScanned = useCallback(
    ({ data }: { data: string }) => {
      if (lockedRef.current) return;
      lockedRef.current = true;
      setState('processing');
      setMessage('Checking ticket…');
      void processScan(data);
    },
    [processScan]
  );

  if (!permission) {
    return (
      <SafeAreaView style={styles.container}>
        <ActivityIndicator color={Colors.primaryText} size="large" />
      </SafeAreaView>
    );
  }

  if (!permission.granted) {
    return (
      <SafeAreaView style={styles.container}>
        <Text style={styles.title}>Camera access needed</Text>
        <Text style={styles.subtitle}>
          Agora needs camera access to scan attendee ticket QR codes at the gate.
        </Text>
        <Button title="Grant Camera Access" onPress={requestPermission} />
      </SafeAreaView>
    );
  }

  return (
    <View style={styles.container}>
      <View style={styles.modeRow}>
        <Pressable
          style={[styles.modeButton, mode === 'checkin' && styles.modeButtonActive]}
          onPress={() => setMode('checkin')}
        >
          <Text style={[styles.modeButtonText, mode === 'checkin' && styles.modeButtonTextActive]}>
            Check-in
          </Text>
        </Pressable>
        <Pressable
          style={[styles.modeButton, mode === 'checkout' && styles.modeButtonActive]}
          onPress={() => setMode('checkout')}
        >
          <Text style={[styles.modeButtonText, mode === 'checkout' && styles.modeButtonTextActive]}>
            Check-out
          </Text>
        </Pressable>
      </View>

      <View style={styles.cameraWrapper}>
        <CameraView
          style={StyleSheet.absoluteFillObject}
          facing="back"
          barcodeScannerSettings={{ barcodeTypes: ['qr'] }}
          onBarcodeScanned={state === 'scanning' ? handleBarcodeScanned : undefined}
        />
        <View style={[styles.target, targetStyles[state]]} />
      </View>

      <View style={styles.footer}>
        <Text style={[styles.resultLabel, resultTextStyles[state]]}>{describeState(state)}</Text>
        <Text style={styles.message}>{message}</Text>
      </View>
    </View>
  );
}

function describeState(state: ScanState): string {
  switch (state) {
    case 'success':
      return 'ADMITTED';
    case 'duplicate':
      return 'DUPLICATE';
    case 'invalid':
      return 'INVALID';
    case 'expired':
      return 'EXPIRED';
    case 'error':
      return 'OFFLINE';
    case 'processing':
      return 'CHECKING…';
    default:
      return 'READY';
  }
}

const styles = StyleSheet.create({
  container: {
    flex: 1,
    backgroundColor: Colors.darkBackground,
  },
  title: { color: Colors.primaryText, fontSize: 22, fontWeight: '700', textAlign: 'center', marginTop: 40 },
  subtitle: {
    color: Colors.secondaryText,
    fontSize: 14,
    textAlign: 'center',
    marginTop: 12,
    marginBottom: 24,
    paddingHorizontal: 32,
  },
  modeRow: {
    flexDirection: 'row',
    padding: 16,
    gap: 8,
  },
  modeButton: {
    flex: 1,
    paddingVertical: 10,
    borderRadius: 8,
    alignItems: 'center',
    backgroundColor: '#1E1E20',
  },
  modeButtonActive: {
    backgroundColor: Colors.primaryYellow,
  },
  modeButtonText: {
    color: Colors.primaryText,
    fontWeight: '600',
  },
  modeButtonTextActive: {
    color: Colors.darkBackground,
  },
  cameraWrapper: {
    flex: 1,
    alignItems: 'center',
    justifyContent: 'center',
    overflow: 'hidden',
  },
  target: {
    width: 240,
    height: 240,
    borderRadius: 16,
    borderWidth: 4,
  },
  footer: {
    padding: 24,
    alignItems: 'center',
  },
  resultLabel: {
    fontSize: 24,
    fontWeight: '800',
    marginBottom: 6,
  },
  message: {
    color: Colors.secondaryText,
    fontSize: 13,
    textAlign: 'center',
  },
});

const targetStyles = StyleSheet.create({
  scanning: { borderColor: Colors.primaryYellow },
  processing: { borderColor: Colors.primaryYellow },
  success: { borderColor: '#34C759' },
  duplicate: { borderColor: Colors.accentRed },
  invalid: { borderColor: Colors.accentRed },
  expired: { borderColor: '#FF9500' },
  error: { borderColor: '#FF9500' },
});

const resultTextStyles = StyleSheet.create({
  scanning: { color: Colors.primaryText },
  processing: { color: Colors.primaryText },
  success: { color: '#34C759' },
  duplicate: { color: Colors.accentRed },
  invalid: { color: Colors.accentRed },
  expired: { color: '#FF9500' },
  error: { color: '#FF9500' },
});
