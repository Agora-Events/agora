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
