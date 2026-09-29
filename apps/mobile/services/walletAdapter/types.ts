/**
 * Unified Mobile Wallet Adapter Types for Stellar & Soroban
 */

export type WalletType =
  | 'freighter'
  | 'walletconnect'
  | 'embedded'
  | 'lobstr'
  | 'albedo';

export interface WalletMetadata {
  id: WalletType;
  name: string;
  icon: string;
  description: string;
  isDeepLink: boolean;
}

export interface SignTransactionOptions {
  networkPassphrase?: string;
  network?: 'PUBLIC' | 'TESTNET';
  accountToSign?: string;
}

export interface DecodedOperation {
  type: string;
  destination?: string;
  amount?: string;
  asset?: string;
  contractId?: string;
  functionName?: string;
  args?: any[];
  [key: string]: any;
}

export interface DecodedResourceLimits {
  instructions?: number;
  readBytes?: number;
  writeBytes?: number;
}

export interface DecodedXDR {
  sourceAccount: string;
  fee: string;
  sequenceNumber: string;
  operationCount: number;
  operations: DecodedOperation[];
  contractAddress?: string;
  functionName?: string;
  functionArgs?: any[];
  resourceLimits?: DecodedResourceLimits;
  rawXdr: string;
}

export interface MobileWalletAdapter {
  id: WalletType;
  name: string;
  icon: string;
  description: string;

  /**
   * Checks if this wallet provider is currently connected.
   */
  isConnected(): boolean;

  /**
   * Retrieves the current connected public key.
   */
  getPublicKey(): Promise<string | null>;

  /**
   * Initiates wallet connection / handshake.
   * Returns the user's public Stellar address.
   */
  connect(): Promise<string>;

  /**
   * Disconnects the wallet session.
   */
  disconnect(): Promise<void>;

  /**
   * Signs a Stellar / Soroban transaction envelope encoded in base64 XDR.
   * Returns the signed transaction XDR.
   */
  signTransaction(xdr: string, options?: SignTransactionOptions): Promise<string>;

  /**
   * Signs a Soroban authorization entry (for multi-sig or sub-invocations).
   * Returns the signed entry.
   */
  signAuthEntry(entry: string, options?: SignTransactionOptions): Promise<string>;
}
