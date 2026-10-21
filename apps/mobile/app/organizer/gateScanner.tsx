/**
 * gateScanner.tsx
 *
 * Organizer-facing gate screen. Waits for an NFC tap, validates the
 * resulting ticket token, records the result into the local BLE mesh log
 * so it survives an offline gate, and gives the organizer immediate
 * haptic + audio feedback on approve/deny.
 */

import React, { useCallback, useEffect, useRef, useState } from 'react';
import { View, Text, StyleSheet, SafeAreaView, ActivityIndicator } from 'react-native';
import { useLocalSearchParams } from 'expo-router';
import * as Haptics from 'expo-haptics';
import { Audio } from 'expo-av';
import Colors from '@/constants/Colors';
import Button from '@/components/ui/Button';
import { readTicketTap, NfcTicketToken } from '@/services/nfcService';
import { BleMeshService, CheckInResult } from '@/services/bleMeshService';

/** How long the last scan result stays on screen before we re-arm the reader. */
const RESULT_DISPLAY_MS = 1500;

type ScanState = 'idle' | 'waiting' | 'approved' | 'denied';

/** Stand-in for real signature/expiry validation against the event roster. */
function validateTicketToken(token: NfcTicketToken): boolean {
  const ageSeconds = Math.floor(Date.now() / 1000) - token.timestamp;
  return Boolean(token.ticketId) && Boolean(token.signature) && ageSeconds < 60;
}

export default function GateScannerScreen() {
  const { gateId } = useLocalSearchParams<{ gateId?: string }>();
  const resolvedGateId = gateId ?? 'gate-1';

  const [state, setState] = useState<ScanState>('idle');
  const [lastTicketId, setLastTicketId] = useState<string | null>(null);
  const meshRef = useRef<BleMeshService | null>(null);

  // Each organizer device gets a stable-for-session scanner id for gossip attribution.
  useEffect(() => {
    const scannerId = `scanner-${Math.random().toString(36).slice(2, 10)}`;
    const mesh = new BleMeshService(scannerId);
    meshRef.current = mesh;
    mesh.start(resolvedGateId).catch((e) => console.warn('BLE mesh failed to start', e));

    return () => {
      mesh.stop().catch(() => {});
    };
  }, [resolvedGateId]);

  const playFeedback = useCallback(async (result: CheckInResult) => {
    if (result === 'approved') {
      // Double haptic tap reads as a distinct "good" pattern at arm's length.
      await Haptics.notificationAsync(Haptics.NotificationFeedbackType.Success);
      await new Promise((r) => setTimeout(r, 120));
      await Haptics.impactAsync(Haptics.ImpactFeedbackStyle.Light);
    } else {
      await Haptics.notificationAsync(Haptics.NotificationFeedbackType.Error);
    }

    const asset =
      result === 'approved'
        ? // eslint-disable-next-line @typescript-eslint/no-require-imports -- Metro needs a static require() for bundled assets
          require('@/assets/sounds/approved-chirp.wav')
        : // eslint-disable-next-line @typescript-eslint/no-require-imports -- Metro needs a static require() for bundled assets
          require('@/assets/sounds/denied-alert.wav');
    const { sound } = await Audio.Sound.createAsync(asset);
    sound.playAsync().catch(() => {});
    sound.setOnPlaybackStatusUpdate((status) => {
      if ('didJustFinish' in status && status.didJustFinish) sound.unloadAsync().catch(() => {});
    });
  }, []);

  const handleScan = useCallback(async () => {
    setState('waiting');
    try {
      const token = await readTicketTap();
      const approved = validateTicketToken(token);
      const result: CheckInResult = approved ? 'approved' : 'denied';

      meshRef.current?.recordCheckIn(token.ticketId, resolvedGateId, result);
      setLastTicketId(token.ticketId);
      setState(result);
      await playFeedback(result);
    } catch (e) {
      console.warn('Gate tap read failed', e);
      setState('denied');
      await playFeedback('denied');
    } finally {
      setTimeout(() => setState('idle'), RESULT_DISPLAY_MS);
    }
  }, [playFeedback, resolvedGateId]);

  const pendingCount = meshRef.current?.pendingCount ?? 0;

  return (
    <SafeAreaView style={styles.container}>
      <Text style={styles.title}>Gate {resolvedGateId}</Text>
      <Text style={styles.subtitle}>{pendingCount} check-ins queued for mesh sync</Text>

      <View style={[styles.statusCircle, statusStyles[state]]}>
        {state === 'waiting' ? (
          <ActivityIndicator color={Colors.primaryText} size="large" />
        ) : (
          <Text style={styles.statusText}>{statusLabel(state)}</Text>
        )}
      </View>

      {lastTicketId && <Text style={styles.ticketId}>Last: {lastTicketId}</Text>}

      <Button title="Tap to Scan" onPress={handleScan} disabled={state === 'waiting'} />
    </SafeAreaView>
  );
}

function statusLabel(state: ScanState): string {
  if (state === 'approved') return 'APPROVED';
  if (state === 'denied') return 'DENIED';
  return 'Ready';
}

const styles = StyleSheet.create({
  container: {
    flex: 1,
    backgroundColor: Colors.darkBackground,
    alignItems: 'center',
    justifyContent: 'center',
    padding: 24,
  },
  title: { color: Colors.primaryText, fontSize: 22, fontWeight: '700' },
  subtitle: { color: Colors.secondaryText, fontSize: 13, marginTop: 4, marginBottom: 32 },
  statusCircle: {
    width: 200,
    height: 200,
    borderRadius: 100,
    alignItems: 'center',
    justifyContent: 'center',
    marginBottom: 32,
  },
  statusText: { color: Colors.primaryText, fontSize: 24, fontWeight: '800' },
  ticketId: { color: Colors.secondaryText, marginBottom: 16 },
});

const statusStyles = StyleSheet.create({
  idle: { backgroundColor: '#2C2C2E' },
  waiting: { backgroundColor: '#2C2C2E' },
  approved: { backgroundColor: '#34C759' },
  denied: { backgroundColor: Colors.accentRed },
});
