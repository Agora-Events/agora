import React, { useEffect } from 'react';
import { ActivityIndicator, StyleSheet, Text, View } from 'react-native';
import { useLocalSearchParams, useRouter } from 'expo-router';
import Colors from '@/constants/Colors';
import Button from '@/components/ui/Button';
import { useAuth } from '@/hooks/useAuth';
import { useWaitingRoom } from '@/hooks/useWaitingRoom';

/**
 * Virtual waiting room screen (Issue #1187).
 *
 * Gates checkout during high-traffic ticket releases:
 *   1. Solves the server's SHA-256 proof-of-work challenge (anti-bot).
 *   2. Joins the Redis-backed FIFO queue and streams its live position via
 *      SSE ("You are #142 in line", "Estimated wait: 45s").
 *   3. On receiving the cryptographically signed checkout access grant, it
 *      automatically redirects to `/checkout` with the grant token attached.
 */
export default function WaitingRoomScreen() {
  const params = useLocalSearchParams<{ eventId?: string; eventTitle?: string }>();
  const router = useRouter();
  const { user } = useAuth();

  const eventId = params.eventId ?? '';
  const eventTitle = params.eventTitle ?? 'Agora Event';
  const clientId = user?.walletAddress || 'guest';

  const { phase, position, queueSize, estimatedWaitSeconds, grantToken, errorMessage, retry } =
    useWaitingRoom(eventId, clientId);

  // Auto-redirect to checkout the moment the signed grant arrives.
  useEffect(() => {
    if (phase === 'admitted' && grantToken && eventId) {
      router.replace({
        pathname: '/checkout',
        params: { eventId, eventTitle, grantToken },
      });
    }
  }, [phase, grantToken, eventId, eventTitle, router]);

  const isBusy = phase === 'fetching-challenge' || phase === 'solving' || phase === 'joining';
  const progress = queueSize > 0 && position != null ? Math.max(0, Math.min(1, 1 - position / queueSize)) : 0;

  return (
    <View style={styles.container}>
      <View style={styles.content}>
        <Text style={styles.eyebrow}>VIRTUAL WAITING ROOM</Text>
        <Text style={styles.title}>{eventTitle}</Text>

        {isBusy ? (
          <View style={styles.centered}>
            <ActivityIndicator color={Colors.primaryYellow} size="large" />
            <Text style={styles.statusText}>
              {phase === 'fetching-challenge' && 'Contacting the queue...'}
              {phase === 'solving' && 'Verifying you are human...'}
              {phase === 'joining' && 'Joining the queue...'}
            </Text>
            <Text style={styles.statusHint}>
              This one-time check stops automated bots from cutting the line.
            </Text>
          </View>
        ) : null}

        {phase === 'waiting' && position != null ? (
          <View style={styles.queueCard}>
            <Text style={styles.positionLabel}>You are</Text>
            <Text style={styles.positionValue} testID="queue-position">
              #{position}
            </Text>
            <Text style={styles.positionLabel}>in line</Text>

            <View style={styles.progressTrack}>
              <View style={[styles.progressFill, { width: `${Math.round(progress * 100)}%` }]} />
            </View>

            <Text style={styles.estimatedWait} testID="estimated-wait">
              Estimated wait:{' '}
              {estimatedWaitSeconds != null
                ? formatWait(estimatedWaitSeconds)
                : 'calculating...'}
            </Text>
            {queueSize > 0 ? (
              <Text style={styles.queueSize}>{queueSize.toLocaleString()} people ahead of you</Text>
            ) : null}

            <View style={styles.keepAliveRow}>
              <View style={styles.liveDot} />
              <Text style={styles.keepAliveText}>
                Your place is reserved — keep this screen open. You will be sent to checkout
                automatically.
              </Text>
            </View>
          </View>
        ) : null}

        {phase === 'admitted' ? (
          <View style={styles.centered}>
            <Text style={styles.admittedTitle}>You are in! 🎉</Text>
            <Text style={styles.statusText}>Taking you to checkout...</Text>
          </View>
        ) : null}

        {phase === 'error' ? (
          <View style={styles.centered}>
            <Text style={styles.errorTitle}>Could not join the queue</Text>
            <Text style={styles.errorMessage}>{errorMessage}</Text>
            <Button testID="waiting-room-retry" title="Try again" onPress={retry} variant="primary" />
          </View>
        ) : null}
      </View>
    </View>
  );
}

function formatWait(seconds: number): string {
  const rounded = Math.max(1, Math.round(seconds));
  const minutes = Math.floor(rounded / 60);
  const secs = rounded % 60;
  if (minutes === 0) return `${secs}s`;
  return `${minutes}m ${secs}s`;
}

const styles = StyleSheet.create({
  container: {
    flex: 1,
    backgroundColor: Colors.darkBackground,
  },
  content: {
    flex: 1,
    padding: 24,
    justifyContent: 'center',
  },
  centered: {
    alignItems: 'center',
    marginTop: 32,
    gap: 12,
  },
  eyebrow: {
    color: Colors.primaryYellow,
    fontSize: 12,
    fontWeight: '700',
    letterSpacing: 1.5,
    textAlign: 'center',
  },
  title: {
    color: Colors.primaryText,
    fontSize: 22,
    fontWeight: '700',
    textAlign: 'center',
    marginTop: 8,
  },
  statusText: {
    color: Colors.primaryText,
    fontSize: 16,
    fontWeight: '600',
    textAlign: 'center',
    marginTop: 16,
  },
  statusHint: {
    color: Colors.secondaryText,
    fontSize: 13,
    textAlign: 'center',
    marginTop: 4,
  },
  queueCard: {
    backgroundColor: '#1A1A1C',
    borderRadius: 16,
    padding: 24,
    marginTop: 32,
    alignItems: 'center',
  },
  positionLabel: {
    color: Colors.secondaryText,
    fontSize: 13,
  },
  positionValue: {
    color: Colors.primaryText,
    fontSize: 48,
    fontWeight: '800',
    marginVertical: 4,
  },
  progressTrack: {
    width: '100%',
    height: 6,
    borderRadius: 3,
    backgroundColor: '#2C2C2E',
    marginVertical: 16,
    overflow: 'hidden',
  },
  progressFill: {
    height: '100%',
    borderRadius: 3,
    backgroundColor: Colors.primaryYellow,
  },
  estimatedWait: {
    color: Colors.primaryText,
    fontSize: 15,
    fontWeight: '600',
  },
  queueSize: {
    color: Colors.secondaryText,
    fontSize: 12,
    marginTop: 4,
  },
  keepAliveRow: {
    flexDirection: 'row',
    alignItems: 'flex-start',
    gap: 8,
    marginTop: 20,
  },
  liveDot: {
    width: 8,
    height: 8,
    borderRadius: 4,
    backgroundColor: Colors.primaryYellow,
    marginTop: 4,
  },
  keepAliveText: {
    color: Colors.secondaryText,
    fontSize: 12,
    lineHeight: 17,
    flex: 1,
  },
  admittedTitle: {
    color: Colors.primaryText,
    fontSize: 22,
    fontWeight: '700',
  },
  errorTitle: {
    color: Colors.primaryText,
    fontSize: 18,
    fontWeight: '700',
  },
  errorMessage: {
    color: Colors.secondaryText,
    fontSize: 13,
    textAlign: 'center',
    marginBottom: 8,
  },
});
