import { useState, useEffect, useCallback, useRef } from 'react';
import {
  WalletType,
  MobileWalletAdapter,
  SignTransactionOptions,
  DecodedXDR,
  getWalletAdapter,
  decodeXDR,
  getStoredWalletSession,
  saveWalletSession,
  clearWalletSession,
} from '@/services/walletAdapter';

export type WalletConnectionStatus =
  | 'disconnected'
  | 'connecting'
  | 'connected'
  | 'error';

export interface UseWalletReturn {
  wallet: MobileWalletAdapter | null;
  walletType: WalletType | null;
  publicKey: string | null;
  status: WalletConnectionStatus;
  isConnected: boolean;
  isConnecting: boolean;
  error: string | null;
  connect: (type: WalletType) => Promise<string>;
  disconnect: () => Promise<void>;
  signTransaction: (xdr: string, options?: SignTransactionOptions) => Promise<string>;
  signAuthEntry: (entry: string, options?: SignTransactionOptions) => Promise<string>;
  previewTransaction: (xdr: string) => DecodedXDR;
}

export function useWallet(): UseWalletReturn {
  const [wallet, setWallet] = useState<MobileWalletAdapter | null>(null);
  const [walletType, setWalletType] = useState<WalletType | null>(null);
  const [publicKey, setPublicKey] = useState<string | null>(null);
  const [status, setStatus] = useState<WalletConnectionStatus>('disconnected');
  const [error, setError] = useState<string | null>(null);

  const isMounted = useRef(true);

  useEffect(() => {
    isMounted.current = true;

    // Auto-reconnect from stored session on app startup
    async function restoreSession() {
      try {
        const stored = await getStoredWalletSession();
        if (stored && isMounted.current) {
          const adapter = getWalletAdapter(stored.type);
          setWallet(adapter);
          setWalletType(stored.type);
          setPublicKey(stored.publicKey);
          setStatus('connected');
        }
      } catch (err: any) {
        if (isMounted.current) {
          console.warn('Auto-reconnect error:', err);
        }
      }
    }

    restoreSession();

    return () => {
      isMounted.current = false;
    };
  }, []);

  const connect = useCallback(async (type: WalletType): Promise<string> => {
    setStatus('connecting');
    setError(null);

    try {
      const adapter = getWalletAdapter(type);
      const pubKey = await adapter.connect();

      if (isMounted.current) {
        setWallet(adapter);
        setWalletType(type);
        setPublicKey(pubKey);
        setStatus('connected');
      }

      await saveWalletSession(type, pubKey);
      return pubKey;
    } catch (err: any) {
      const errorMessage = err?.message || 'Failed to connect wallet';
      if (isMounted.current) {
        setError(errorMessage);
        setStatus('error');
      }
      throw err;
    }
  }, []);

  const disconnect = useCallback(async (): Promise<void> => {
    try {
      if (wallet) {
        await wallet.disconnect();
      }
    } catch (err) {
      console.warn('Disconnect error:', err);
    } finally {
      if (isMounted.current) {
        setWallet(null);
        setWalletType(null);
        setPublicKey(null);
        setStatus('disconnected');
        setError(null);
      }
      await clearWalletSession();
    }
  }, [wallet]);

  const signTransaction = useCallback(
    async (xdr: string, options?: SignTransactionOptions): Promise<string> => {
      if (!wallet) {
        throw new Error('No wallet connected for signing');
      }
      return wallet.signTransaction(xdr, options);
    },
    [wallet]
  );

  const signAuthEntry = useCallback(
    async (entry: string, options?: SignTransactionOptions): Promise<string> => {
      if (!wallet) {
        throw new Error('No wallet connected for signing');
      }
      return wallet.signAuthEntry(entry, options);
    },
    [wallet]
  );

  const previewTransaction = useCallback((xdr: string): DecodedXDR => {
    return decodeXDR(xdr);
  }, []);

  return {
    wallet,
    walletType,
    publicKey,
    status,
    isConnected: status === 'connected',
    isConnecting: status === 'connecting',
    error,
    connect,
    disconnect,
    signTransaction,
    signAuthEntry,
    previewTransaction,
  };
}

export default useWallet;
