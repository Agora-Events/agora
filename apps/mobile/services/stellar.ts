import * as SecureStore from 'expo-secure-store';
import { Asset, Keypair, Networks, Operation, TransactionBuilder, Horizon } from '@stellar/stellar-sdk';

const STORAGE_KEY = 'agora_stellar_secret_key';
const HORIZON_URL = 'https://horizon-testnet.stellar.org';
const USDC_ASSET_CODE = 'USDC';
const USDC_ISSUER = 'GBBD47IF6LWK7P7MABDHYZCYYZ277OEX22PQ6CHYA67BDN6C7CR27STN';

export interface StellarBalances {
  accountExists: boolean;
  xlmBalance: string;
  usdcBalance: string;
  hasUSDCTrustline: boolean;
}

export interface TrustlineResult {
  success: boolean;
  message: string;
}

export class StellarWalletManager {
  static async saveSecretKey(secretKey: string): Promise<void> {
    await SecureStore.setItemAsync(STORAGE_KEY, secretKey, {
      keychainService: 'agora_stellar_wallet',
    });
  }

  static async getSecretKey(): Promise<string | null> {
    return SecureStore.getItemAsync(STORAGE_KEY, {
      keychainService: 'agora_stellar_wallet',
    });
  }

  static async deleteSecretKey(): Promise<void> {
    await SecureStore.deleteItemAsync(STORAGE_KEY, {
      keychainService: 'agora_stellar_wallet',
    });
  }

  static generateKeypair(): { secretKey: string; publicKey: string } {
    const keypair = Keypair.random();
    return {
      secretKey: keypair.secret(),
      publicKey: keypair.publicKey(),
    };
  }

  static getPublicKeyFromSecret(secretKey: string): string {
    return Keypair.fromSecret(secretKey).publicKey();
  }

  static async importSecretKey(secretKey: string): Promise<{ publicKey: string }> {
    const publicKey = Keypair.fromSecret(secretKey).publicKey();
    await this.saveSecretKey(secretKey);
    return { publicKey };
  }

  static getUSDCAsset() {
    return new Asset(USDC_ASSET_CODE, USDC_ISSUER);
  }

  static async getBalances(publicKey: string): Promise<StellarBalances> {
    const server = new Horizon.Server(HORIZON_URL);

    try {
      const account = await server.loadAccount(publicKey);
      const xlmBalance =
        account.balances.find((balance: any) => balance.asset_type === 'native')?.balance ?? '0';
      const usdcBalanceEntry = account.balances.find((balance: any) =>
        balance.asset_type === 'credit_alphanum4' &&
        balance.asset_code === USDC_ASSET_CODE &&
        balance.asset_issuer === USDC_ISSUER
      );
      const usdcBalance = usdcBalanceEntry?.balance ?? '0';

      return {
        accountExists: true,
        xlmBalance,
        usdcBalance,
        hasUSDCTrustline: typeof usdcBalanceEntry !== 'undefined',
      };
    } catch (error: any) {
      const status = error?.response?.status ?? error?.status;
      if (status === 404) {
        return {
          accountExists: false,
          xlmBalance: '0',
          usdcBalance: '0',
          hasUSDCTrustline: false,
        };
      }
      throw error;
    }
  }

  static async submitUSDCTrustline(): Promise<TrustlineResult> {
    const secretKey = await this.getSecretKey();
    if (!secretKey) {
      return {
        success: false,
        message: 'No stored Stellar wallet found.',
      };
    }

    const keypair = Keypair.fromSecret(secretKey);
    const server = new Horizon.Server(HORIZON_URL);

    try {
      const account = await server.loadAccount(keypair.publicKey());
      const fee = await server.fetchBaseFee();
      const transaction = new TransactionBuilder(account, {
        fee: fee.toString(),
        networkPassphrase: Networks.TESTNET,
      })
        .addOperation(
          Operation.changeTrust({
            asset: this.getUSDCAsset(),
          })
        )
        .setTimeout(30)
        .build();

      transaction.sign(keypair);
      await server.submitTransaction(transaction);

      return {
        success: true,
        message: 'USDC trustline submitted on Stellar Testnet.',
      };
    } catch (error: any) {
      return {
        success: false,
        message: error?.response?.data?.extras?.result_codes?.operations?.[0] ||
          error?.message ||
          'Unable to submit USDC trustline.',
      };
    }
  }
}
