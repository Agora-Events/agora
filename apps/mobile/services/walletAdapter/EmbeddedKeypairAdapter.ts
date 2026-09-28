import * as SecureStore from 'expo-secure-store';
import { Keypair, TransactionBuilder, Networks } from '@stellar/stellar-sdk';
import { MobileWalletAdapter, SignTransactionOptions, WalletType } from './types';

const SECURE_STORE_KEY = 'agora_embedded_stellar_secret';

export class EmbeddedKeypairAdapter implements MobileWalletAdapter {
  id: WalletType = 'embedded';
  name = 'Agora Embedded Keypair';
  icon = 'key-outline';
  description = 'Local non-custodial encrypted keypair for instant onboarding';

  private keypair: Keypair | null = null;

  isConnected(): boolean {
    return Boolean(this.keypair);
  }

  async getPublicKey(): Promise<string | null> {
    if (this.keypair) {
      return this.keypair.publicKey();
    }
    const secret = await SecureStore.getItemAsync(SECURE_STORE_KEY);
    if (secret) {
      try {
        this.keypair = Keypair.fromSecret(secret);
        return this.keypair.publicKey();
      } catch {
        return null;
      }
    }
    return null;
  }

  async connect(): Promise<string> {
    let secret = await SecureStore.getItemAsync(SECURE_STORE_KEY);

    if (!secret) {
      // Generate new non-custodial keypair
      const newKeypair = Keypair.random();
      secret = newKeypair.secret();
      await SecureStore.setItemAsync(SECURE_STORE_KEY, secret);
      this.keypair = newKeypair;
    } else {
      this.keypair = Keypair.fromSecret(secret);
    }

    return this.keypair.publicKey();
  }

  async disconnect(): Promise<void> {
    this.keypair = null;
  }

  async signTransaction(xdr: string, options?: SignTransactionOptions): Promise<string> {
    if (!this.keypair) {
      const pubKey = await this.getPublicKey();
      if (!pubKey || !this.keypair) {
        throw new Error('No embedded keypair available for signing.');
      }
    }

    const networkPassphrase =
      options?.networkPassphrase ||
      (options?.network === 'PUBLIC' ? Networks.PUBLIC : Networks.TESTNET);

    const tx = TransactionBuilder.fromXDR(xdr, networkPassphrase);
    (tx as any).sign(this.keypair);
    return tx.toXDR();
  }

  async signAuthEntry(entry: string, _options?: SignTransactionOptions): Promise<string> {
    if (!this.keypair) {
      throw new Error('No embedded keypair available for signing.');
    }

    const entryBuffer = Buffer.from(entry, 'base64');
    const signature = this.keypair.sign(entryBuffer);
    return signature.toString('base64');
  }

  /**
   * Clears the stored private key permanently.
   */
  async deleteStoredKeypair(): Promise<void> {
    this.keypair = null;
    await SecureStore.deleteItemAsync(SECURE_STORE_KEY);
  }
}
