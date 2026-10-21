/**
 * useBiometricAuth.ts — Issue #1179: Offline Ticket Vault
 *
 * React hook encapsulating biometric authentication state and the prompt
 * trigger. Used by the ticket detail screen to gate vault access.
 *
 * ## States
 *
 *   idle        — not yet attempted
 *   checking    — querying hardware availability
 *   unavailable — device has no biometric hardware or no enrolled credentials
 *                 (gate staff UI must show a fallback / passcode path)
 *   prompting   — OS prompt is active
 *   success     — authenticated; vault reads are permitted (Android) /
 *                 will be permitted by the OS prompt (iOS)
 *   failed      — user cancelled or biometric did not match; show an explicit
 *                 error — do NOT silently fall through to "unlocked"
 *   error       — unexpected runtime error (device locked, hardware fault)
 */

import { useState, useCallback, useEffect } from 'react';
import * as LocalAuthentication from 'expo-local-authentication';

export type BiometricState =
  | 'idle'
  | 'checking'
  | 'unavailable'
  | 'prompting'
  | 'success'
  | 'failed'
  | 'error';

export interface BiometricCapabilities {
  /** True if the hardware supports biometric authentication. */
  hasHardware: boolean;
  /** True if the user has enrolled at least one biometric credential. */
  isEnrolled: boolean;
  /** List of biometric types available (Face, Fingerprint, Iris). */
  supportedTypes: LocalAuthentication.AuthenticationType[];
}

export interface UseBiometricAuthResult {
  state: BiometricState;
  /** Populated when state is 'unavailable' or 'failed' or 'error'. */
  errorMessage: string | null;
  capabilities: BiometricCapabilities | null;
  /** Triggers the biometric prompt. Safe to call when state is 'idle' or 'failed'. */
  authenticate: () => Promise<void>;
  /** Resets state back to 'idle' (useful after a transaction completes). */
  reset: () => void;
}

/** Prompt strings shown to the user in the OS native biometric dialog. */
const PROMPT_STRINGS = {
  promptMessage: 'Authenticate to view your ticket',
  cancelLabel: 'Cancel',
  fallbackLabel: 'Use Passcode',
} as const;

export function useBiometricAuth(): UseBiometricAuthResult {
  const [state, setState] = useState<BiometricState>('idle');
  const [errorMessage, setErrorMessage] = useState<string | null>(null);
  const [capabilities, setCapabilities] = useState<BiometricCapabilities | null>(null);

  // Check hardware capabilities once on mount.
  useEffect(() => {
    let cancelled = false;

    async function checkCapabilities() {
      setState('checking');
      try {
        const [hasHardware, isEnrolled, supportedTypes] = await Promise.all([
          LocalAuthentication.hasHardwareAsync(),
          LocalAuthentication.isEnrolledAsync(),
          LocalAuthentication.supportedAuthenticationTypesAsync(),
        ]);

        if (cancelled) return;

        const caps: BiometricCapabilities = { hasHardware, isEnrolled, supportedTypes };
        setCapabilities(caps);

        if (!hasHardware || !isEnrolled) {
          setState('unavailable');
          setErrorMessage(
            !hasHardware
              ? 'This device does not support biometric authentication.'
              : 'No biometric credentials are enrolled. Please set up Face ID, Touch ID, or a passcode in your device settings.'
          );
        } else {
          setState('idle');
        }
      } catch (err) {
        if (cancelled) return;
        setState('error');
        setErrorMessage(
          err instanceof Error
            ? err.message
            : 'An unexpected error occurred while checking biometric availability.'
        );
      }
    }

    checkCapabilities();
    return () => { cancelled = true; };
  }, []);

  const authenticate = useCallback(async () => {
    // Prevent re-entrance while a prompt is already active.
    if (state === 'prompting') return;

    setState('prompting');
    setErrorMessage(null);

    try {
      const result = await LocalAuthentication.authenticateAsync({
        promptMessage: PROMPT_STRINGS.promptMessage,
        cancelLabel: PROMPT_STRINGS.cancelLabel,
        fallbackLabel: PROMPT_STRINGS.fallbackLabel,
        // Require strong biometric (Face ID / Touch ID) but allow passcode
        // as fallback (disableDeviceFallback: false is the default).
        disableDeviceFallback: false,
      });

      if (result.success) {
        setState('success');
      } else {
        // result.error is a string like 'user_cancel', 'lockout', 'user_fallback', etc.
        setState('failed');
        setErrorMessage(describeAuthFailure(result.error));
      }
    } catch (err) {
      setState('error');
      setErrorMessage(
        err instanceof Error ? err.message : 'Biometric authentication failed unexpectedly.'
      );
    }
  }, [state]);

  const reset = useCallback(() => {
    setState('idle');
    setErrorMessage(null);
  }, []);

  return { state, errorMessage, capabilities, authenticate, reset };
}

// ── Error message helpers ─────────────────────────────────────────────────────

function describeAuthFailure(error: string | undefined): string {
  switch (error) {
    case 'user_cancel':
      return 'Authentication was cancelled. Tap to try again.';
    case 'lockout':
    case 'lockout_permanent':
      return 'Too many failed attempts. Please use your passcode or wait and try again.';
    case 'user_fallback':
      return 'Biometric not recognised. Please use your passcode.';
    case 'system_cancel':
      return 'Authentication was interrupted by the system. Please try again.';
    case 'not_enrolled':
      return 'No biometric credentials enrolled. Please configure Face ID or Touch ID.';
    case 'not_available':
      return 'Biometric authentication is not available on this device.';
    default:
      return `Authentication failed${error ? ` (${error})` : ''}. Please try again.`;
  }
}
