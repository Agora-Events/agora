import * as Linking from 'expo-linking';
import { MobileWalletAdapter, SignTransactionOptions, WalletType } from './types';

export class FreighterMobileAdapter implements MobileWalletAdapter {
  id: WalletType = 'freighter';
  name = 'Freighter Mobile';
  icon = 'wallet-outline';
  description = 'Connect via Freighter Mobile deep link with hardware-backed keys';

  private connectedPublicKey: string | null = null;
  private pendingResolvers = new Map<
    string,
    { resolve: (val: any) => void; reject: (err: any) => void }
  >();
  private linkingSubscription: any = null;

  constructor() {
    this.initLinkingListener();
  }

  private initLinkingListener() {
    this.linkingSubscription = Linking.addEventListener('url', this.handleDeepLinkReturn);
  }

  private handleDeepLinkReturn = (event: { url: string }) => {
    try {
      const parsed = Linking.parse(event.url);
      const query = parsed.queryParams || {};

      const requestId = (query.requestId as string) || (query.req as string) || 'default';
      const resolver = this.pendingResolvers.get(requestId);

      if (resolver) {
        if (query.error) {
          resolver.reject(new Error(String(query.error)));
        } else if (query.publicKey) {
          this.connectedPublicKey = String(query.publicKey);
          resolver.resolve(String(query.publicKey));
        } else if (query.signedXdr || query.xdr) {
          resolver.resolve(String(query.signedXdr || query.xdr));
        } else if (query.signedEntry || query.entry) {
          resolver.resolve(String(query.signedEntry || query.entry));
        } else {
          resolver.resolve(query);
        }
        this.pendingResolvers.delete(requestId);
      }
    } catch {
      // Ignore unhandled deep links
    }
  };

  isConnected(): boolean {
    return Boolean(this.connectedPublicKey);
  }

  async getPublicKey(): Promise<string | null> {
    return this.connectedPublicKey;
  }

  async connect(): Promise<string> {
    const requestId = `req_${Date.now()}`;
    const redirectUrl = Linking.createURL('wallet-callback', {
      queryParams: { requestId },
    });

    const freighterUrl = `web+stellar:connect?callback=${encodeURIComponent(
      redirectUrl
    )}&requestId=${requestId}&app_name=Agora`;

    return new Promise<string>(async (resolve, reject) => {
      this.pendingResolvers.set(requestId, { resolve, reject });

      // Fallback timeout
      const timeout = setTimeout(() => {
        if (this.pendingResolvers.has(requestId)) {
          this.pendingResolvers.delete(requestId);
          // If in development/testing or wallet not responding, provide simulation or timeout
          if (__DEV__) {
            const devKey = 'GDEVFREIGHTERDEMO7TESTNETACCOUNTFORAGORAWALLET00000000000';
            this.connectedPublicKey = devKey;
            resolve(devKey);
          } else {
            reject(new Error('Freighter Mobile connection timed out.'));
          }
        }
      }, 30000);

      try {
        const canOpen = await Linking.canOpenURL(freighterUrl);
        if (canOpen) {
          await Linking.openURL(freighterUrl);
        } else {
          // If scheme not directly supported, try alternative freighter:// scheme
          const altUrl = `freighter://connect?callback=${encodeURIComponent(
            redirectUrl
          )}&requestId=${requestId}`;
          const canOpenAlt = await Linking.canOpenURL(altUrl);
          if (canOpenAlt) {
            await Linking.openURL(altUrl);
          } else {
            clearTimeout(timeout);
            this.pendingResolvers.delete(requestId);
            throw new Error('Freighter Mobile is not installed on this device.');
          }
        }
      } catch (err) {
        clearTimeout(timeout);
        this.pendingResolvers.delete(requestId);
        reject(err);
      }
    });
  }

  async disconnect(): Promise<void> {
    this.connectedPublicKey = null;
    this.pendingResolvers.clear();
  }

  async signTransaction(xdr: string, options?: SignTransactionOptions): Promise<string> {
    if (!this.connectedPublicKey) {
      throw new Error('Freighter wallet is not connected');
    }

    const requestId = `sign_${Date.now()}`;
    const redirectUrl = Linking.createURL('wallet-callback', {
      queryParams: { requestId },
    });

    const network = options?.network || 'TESTNET';
    const signUrl = `web+stellar:signXdr?xdr=${encodeURIComponent(
      xdr
    )}&network=${network}&callback=${encodeURIComponent(
      redirectUrl
    )}&requestId=${requestId}`;

    return new Promise<string>(async (resolve, reject) => {
      this.pendingResolvers.set(requestId, { resolve, reject });

      const timeout = setTimeout(() => {
        if (this.pendingResolvers.has(requestId)) {
          this.pendingResolvers.delete(requestId);
          reject(new Error('Signing request timed out.'));
        }
      }, 45000);

      try {
        await Linking.openURL(signUrl);
      } catch (err) {
        clearTimeout(timeout);
        this.pendingResolvers.delete(requestId);
        reject(err);
      }
    });
  }

  async signAuthEntry(entry: string, options?: SignTransactionOptions): Promise<string> {
    if (!this.connectedPublicKey) {
      throw new Error('Freighter wallet is not connected');
    }

    const requestId = `auth_${Date.now()}`;
    const redirectUrl = Linking.createURL('wallet-callback', {
      queryParams: { requestId },
    });

    const signUrl = `web+stellar:signAuthEntry?entry=${encodeURIComponent(
      entry
    )}&callback=${encodeURIComponent(redirectUrl)}&requestId=${requestId}`;

    return new Promise<string>(async (resolve, reject) => {
      this.pendingResolvers.set(requestId, { resolve, reject });

      try {
        await Linking.openURL(signUrl);
      } catch (err) {
        this.pendingResolvers.delete(requestId);
        reject(err);
      }
    });
  }
}
