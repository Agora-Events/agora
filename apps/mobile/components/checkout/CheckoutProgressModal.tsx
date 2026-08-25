import React from 'react';
import { View, Text, Modal, ActivityIndicator, StyleSheet, ScrollView } from 'react-native';
import Colors from '@/constants/Colors';
import Button from '@/components/ui/Button';
import type { CheckoutStep, CheckoutStepStatus } from '@/types/checkout';

interface CheckoutProgressModalProps {
  visible: boolean;
  steps: CheckoutStep[];
  phase: 'in-progress' | 'error' | 'success' | 'idle';
  errorMessage: string | null;
  onRetry: () => void;
  onDismiss: () => void;
}

/**
 * Full-screen step-by-step progress indicator shown while the checkout hook
 * walks through: generate → sign → submit → confirm → record. Required by
 * issue #1005's acceptance criteria.
 */
export default function CheckoutProgressModal({
  visible,
  steps,
  phase,
  errorMessage,
  onRetry,
  onDismiss,
}: CheckoutProgressModalProps) {
  return (
    <Modal visible={visible} transparent animationType="fade" statusBarTranslucent>
      <View style={styles.overlay}>
        <View style={styles.card}>
          <Text style={styles.title}>
            {phase === 'error' ? 'Checkout Failed' : 'Processing Your Purchase'}
          </Text>

          <ScrollView style={styles.stepList} contentContainerStyle={styles.stepListContent}>
            {steps.map((step) => (
              <StepRow key={step.id} step={step} />
            ))}
          </ScrollView>

          {phase === 'error' && errorMessage ? (
            <View style={styles.errorBox}>
              <Text style={styles.errorText}>{errorMessage}</Text>
            </View>
          ) : null}

          {phase === 'error' ? (
            <View style={styles.actions}>
              <Button
                testID="checkout-progress-dismiss"
                title="Dismiss"
                variant="secondary"
                onPress={onDismiss}
                style={styles.actionButton}
              />
              <Button
                testID="checkout-progress-retry"
                title="Retry"
                onPress={onRetry}
                style={styles.actionButton}
              />
            </View>
          ) : (
            <Text style={styles.hint}>Please don't close the app while this completes.</Text>
          )}
        </View>
      </View>
    </Modal>
  );
}

function StepRow({ step }: { step: CheckoutStep }) {
  return (
    <View style={styles.stepRow}>
      <StepIndicator status={step.status} />
      <View style={styles.stepTextContainer}>
        <Text style={[styles.stepLabel, step.status === 'pending' && styles.stepLabelPending]}>
          {step.label}
        </Text>
        {step.detail ? <Text style={styles.stepDetail}>{step.detail}</Text> : null}
      </View>
    </View>
  );
}

function StepIndicator({ status }: { status: CheckoutStepStatus }) {
  if (status === 'active') {
    return <ActivityIndicator size="small" color={Colors.primaryYellow} style={styles.indicator} />;
  }
  if (status === 'done') {
    return (
      <View style={[styles.indicator, styles.dot, styles.dotDone]}>
        <Text style={styles.dotDoneCheck}>✓</Text>
      </View>
    );
  }
  if (status === 'error') {
    return (
      <View style={[styles.indicator, styles.dot, styles.dotError]}>
        <Text style={styles.dotErrorMark}>!</Text>
      </View>
    );
  }
  return <View style={[styles.indicator, styles.dot, styles.dotPending]} />;
}

const styles = StyleSheet.create({
  overlay: {
    flex: 1,
    backgroundColor: 'rgba(0,0,0,0.75)',
    justifyContent: 'center',
    padding: 20,
  },
  card: {
    backgroundColor: '#1E1E20',
    borderRadius: 16,
    borderWidth: 1,
    borderColor: '#2C2C2E',
    padding: 22,
    maxHeight: '80%',
  },
  title: {
    color: Colors.primaryText,
    fontSize: 18,
    fontWeight: '700',
    marginBottom: 16,
    textAlign: 'center',
  },
  stepList: {
    maxHeight: 320,
  },
  stepListContent: {
    gap: 14,
  },
  stepRow: {
    flexDirection: 'row',
    alignItems: 'flex-start',
    gap: 12,
  },
  indicator: {
    width: 22,
    height: 22,
    marginTop: 1,
  },
  dot: {
    borderRadius: 11,
    justifyContent: 'center',
    alignItems: 'center',
  },
  dotPending: {
    backgroundColor: 'transparent',
    borderWidth: 1.5,
    borderColor: '#3A3A3C',
  },
  dotDone: {
    backgroundColor: '#34C759',
  },
  dotDoneCheck: {
    color: '#0F0F10',
    fontSize: 12,
    fontWeight: '900',
  },
  dotError: {
    backgroundColor: Colors.accentRed,
  },
  dotErrorMark: {
    color: '#FFFFFF',
    fontSize: 12,
    fontWeight: '900',
  },
  stepTextContainer: {
    flex: 1,
  },
  stepLabel: {
    color: Colors.primaryText,
    fontSize: 14,
    fontWeight: '600',
  },
  stepLabelPending: {
    color: Colors.secondaryText,
    fontWeight: '500',
  },
  stepDetail: {
    color: Colors.secondaryText,
    fontSize: 12,
    marginTop: 3,
  },
  errorBox: {
    backgroundColor: '#FF3B3022',
    borderWidth: 1,
    borderColor: '#FF3B3055',
    borderRadius: 10,
    padding: 12,
    marginTop: 16,
  },
  errorText: {
    color: Colors.accentRed,
    fontSize: 13,
    lineHeight: 18,
  },
  hint: {
    color: Colors.secondaryText,
    fontSize: 12,
    textAlign: 'center',
    marginTop: 18,
  },
  actions: {
    flexDirection: 'row',
    gap: 12,
    marginTop: 18,
  },
  actionButton: {
    flex: 1,
  },
});
