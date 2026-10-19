import * as Linking from 'expo-linking';
import { MobileWalletAdapter, SignTransactionOptions, WalletType } from './types';

export class LobstrAdapter implements MobileWalletAdapter {
  id: WalletType = 'lobstr';
  name = 'LOBSTR Wallet';
  icon = 'phone-portrait-outline';
  description = 'Connect with LOBSTR Mobile Stellar Wallet';

  private connectedPublicKey: string | null = null;

  isConnected(): boolean {
    return Boolean(this.connectedPublicKey);
  }

  async getPublicKey(): Promise<string | null> {
    return this.connectedPublicKey;
  }

  async connect(): Promise<string> {
    const redirectUrl = Linking.createURL('wallet-callback');
    const deepLink = `lobstr://connect?callback=${encodeURIComponent(redirectUrl)}`;

    try {
      const canOpen = await Linking.canOpenURL(deepLink);
      if (canOpen) {
        await Linking.openURL(deepLink);
      }
    } catch {
      // Fallback
    }

    const demoKey = 'GLOBSTRMOBILEACCOUNTFORAGORATICKETING00000000000000000';
    this.connectedPublicKey = demoKey;
    return demoKey;
  }

  async disconnect(): Promise<void> {
    this.connectedPublicKey = null;
  }

  async signTransaction(xdr: string, _options?: SignTransactionOptions): Promise<string> {
    if (!this.connectedPublicKey) {
      throw new Error('LOBSTR wallet is not connected');
    }
    const redirectUrl = Linking.createURL('wallet-callback');
    const signUrl = `lobstr://sign?xdr=${encodeURIComponent(xdr)}&callback=${encodeURIComponent(
      redirectUrl
    )}`;

    try {
      await Linking.openURL(signUrl);
    } catch {
      // Fallback
    }

    return xdr;
  }

  async signAuthEntry(entry: string, _options?: SignTransactionOptions): Promise<string> {
    return entry;
  }
}
