import * as SecureStore from 'expo-secure-store';
import { WalletType } from './types';

const WALLET_TYPE_KEY = 'agora_wallet_active_type';
const WALLET_PUBKEY_KEY = 'agora_wallet_public_key';
const WALLET_CONNECTED_AT = 'agora_wallet_connected_at';

export interface WalletSession {
  type: WalletType;
  publicKey: string;
  connectedAt: number;
}

export async function saveWalletSession(type: WalletType, publicKey: string): Promise<void> {
  try {
    await SecureStore.setItemAsync(WALLET_TYPE_KEY, type);
    await SecureStore.setItemAsync(WALLET_PUBKEY_KEY, publicKey);
    await SecureStore.setItemAsync(WALLET_CONNECTED_AT, String(Date.now()));
  } catch (err) {
    console.warn('Failed to persist wallet session:', err);
  }
}

export async function getStoredWalletSession(): Promise<WalletSession | null> {
  try {
    const type = (await SecureStore.getItemAsync(WALLET_TYPE_KEY)) as WalletType | null;
    const publicKey = await SecureStore.getItemAsync(WALLET_PUBKEY_KEY);
    const connectedAtStr = await SecureStore.getItemAsync(WALLET_CONNECTED_AT);

    if (type && publicKey) {
      return {
        type,
        publicKey,
        connectedAt: connectedAtStr ? parseInt(connectedAtStr, 10) : Date.now(),
      };
    }
  } catch (err) {
    console.warn('Failed to retrieve stored wallet session:', err);
  }
  return null;
}

export async function clearWalletSession(): Promise<void> {
  try {
    await SecureStore.deleteItemAsync(WALLET_TYPE_KEY);
    await SecureStore.deleteItemAsync(WALLET_PUBKEY_KEY);
    await SecureStore.deleteItemAsync(WALLET_CONNECTED_AT);
  } catch (err) {
    console.warn('Failed to clear stored wallet session:', err);
  }
}

/**
 * Checks if the wallet account has changed since the last stored session.
 * Returns true if the stored public key doesn't match the current wallet state.
 */
export async function hasWalletAccountChanged(): Promise<boolean> {
  try {
    const storedSession = await getStoredWalletSession();
    if (!storedSession) {
      return false; // No session to compare against
    }

    // Import the adapter dynamically to avoid circular dependencies
    const { getWalletAdapter } = require('./index');
    const adapter = getWalletAdapter(storedSession.type);

    if (!adapter.isConnected()) {
      // If adapter reports not connected, but we have a stored session, 
      // the account may have changed or session is stale
      return true;
    }

    const currentPublicKey = await adapter.getPublicKey();
    return currentPublicKey !== storedSession.publicKey;
  } catch (err) {
    console.warn('Failed to check for wallet account change:', err);
    // On error, assume account hasn't changed to avoid breaking functionality
    return false;
  }
}

/**
 * Checks for account changes and clears the session if needed.
 * Returns true if the session was cleared due to account change.
 */
export async function checkAndClearStaleSession(): Promise<boolean> {
  const changed = await hasWalletAccountChanged();
  if (changed) {
    await clearWalletSession();
    console.log('Wallet session cleared due to account change');
    return true;
  }
  return false;
}
