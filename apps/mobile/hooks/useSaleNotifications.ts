/**
 * useSaleNotifications.ts
 *
 * Registers this device for the "your ticket sold" push (issue #1184).
 *
 * A resale can complete while the seller has the app backgrounded — the buyer
 * drives settlement — and the seller has work left to do afterwards: they have
 * to seal the ticket secret to the buyer. Without a push, that handover stalls
 * until they happen to reopen the app, leaving the buyer holding a ticket they
 * cannot check in with.
 *
 * Registration is entirely best-effort. Simulators have no push token, users
 * decline the permission prompt, and Expo Go on Android has its own caveats —
 * none of which should surface as an error in a resale screen. Failures are
 * captured in `error` for optional display and otherwise ignored.
 */

import { useCallback, useEffect, useState } from 'react';
import { Platform } from 'react-native';
import * as Notifications from 'expo-notifications';
import Constants from 'expo-constants';

import { registerPushToken } from '@/services/marketplaceApi';

export interface SaleNotificationsState {
  /** The Expo push token, once registered with the backend. */
  token: string | null;
  /** Why registration did not complete, if it did not. Safe to ignore. */
  error: string | null;
}

/**
 * Resolves the EAS project id Expo needs to mint a push token. Bare
 * `expo-notifications` in SDK 51 requires it explicitly outside of the classic
 * managed workflow.
 */
function getProjectId(): string | undefined {
  return (
    Constants.expoConfig?.extra?.eas?.projectId ??
    (Constants as any)?.easConfig?.projectId
  );
}

export function useSaleNotifications(enabled: boolean = true): SaleNotificationsState {
  const [token, setToken] = useState<string | null>(null);
  const [error, setError] = useState<string | null>(null);

  const register = useCallback(async () => {
    // Push tokens are a device concept; the web build has no equivalent here.
    if (Platform.OS === 'web') {
      setError('Push notifications are not available on web.');
      return;
    }

    const existing = await Notifications.getPermissionsAsync();
    let status = existing.status;

    // Only prompt if we have not been answered yet — re-prompting a user who
    // said no is both futile and annoying.
    if (status !== 'granted' && existing.canAskAgain) {
      status = (await Notifications.requestPermissionsAsync()).status;
    }

    if (status !== 'granted') {
      setError('Notifications are turned off, so we cannot alert you when your ticket sells.');
      return;
    }

    const projectId = getProjectId();
    const { data } = await Notifications.getExpoPushTokenAsync(
      projectId ? { projectId } : undefined
    );

    await registerPushToken(data, Platform.OS === 'ios' ? 'ios' : 'android');
    setToken(data);
    setError(null);
  }, []);

  useEffect(() => {
    if (!enabled) return;

    let cancelled = false;
    register().catch((e: unknown) => {
      if (cancelled) return;
      setError(e instanceof Error ? e.message : 'Could not register for sale alerts.');
    });

    return () => {
      cancelled = true;
    };
  }, [enabled, register]);

  return { token, error };
}
