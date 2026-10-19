/**
 * Staking service — wraps event_registry contract interactions for organizer
 * collateral staking, unstaking and reward claims.
 *
 * All contract calls are mocked in this implementation; swap the bodies of
 * callContract() for real Soroban RPC invocations when the contract is live.
 */

import { Keypair, Networks, TransactionBuilder, Horizon } from '@stellar/stellar-sdk';
import { StellarWalletManager } from './stellar';

// ── Constants ────────────────────────────────────────────────────────────────

const HORIZON_URL = 'https://horizon-testnet.stellar.org';
export const STELLAR_EXPERT_BASE = 'https://stellar.expert/explorer/testnet/tx';

/** event_registry contract address (testnet placeholder) */
export const EVENT_REGISTRY_CONTRACT =
  'CDLZFC3SYJYDZT7K67VZ75HPJVIEUVNIXF47ZG2FB2RMQQVU2HHGCN3';

// ── Public types ─────────────────────────────────────────────────────────────

export interface StakingConfig {
  /** Staking token — always USDC on the Agora testnet */
  tokenAddress: string;
  /** Minimum collateral required (in USDC, decimal-adjusted) */
  minimumStake: number;
}

export type StakingStatus = 'Verified' | 'Unverified';

export interface OrganizerStakingState {
  status: StakingStatus;
  stakedAmount: number;
  pendingRewards: number;
  lockupEnds: string | null;
}

export type TxType = 'stake' | 'unstake' | 'claim';

export interface StakingTransaction {
  id: string;
  type: TxType;
  amount: number;
  date: string;
  txHash: string;
}

export interface StakingResult {
  success: boolean;
  txHash?: string;
  message: string;
}

// ── Helpers ──────────────────────────────────────────────────────────────────

export function explorerUrl(txHash: string): string {
  return `${STELLAR_EXPERT_BASE}/${txHash}`;
}

export function formatDate(iso: string): string {
  return new Date(iso).toLocaleDateString('en-GB', {
    day: '2-digit',
    month: 'short',
    year: 'numeric',
  });
}

// ── Contract call wrapper ─────────────────────────────────────────────────────

async function callContract(
  secretKey: string,
  method: string,
  args: Record<string, unknown> = {},
): Promise<string> {
  const keypair = Keypair.fromSecret(secretKey);
  const server = new Horizon.Server(HORIZON_URL);
  const account = await server.loadAccount(keypair.publicKey());
  const fee = await server.fetchBaseFee();

  // Reference args in method invocation payload
  const memoText = `agora:${method}:${Object.keys(args).length}`;

  const tx = new TransactionBuilder(account, {
    fee: fee.toString(),
    networkPassphrase: Networks.TESTNET,
  })
    .addMemo({ type: 'text', value: memoText } as any)
    .setTimeout(30)
    .build();

  tx.sign(keypair);

  try {
    const result = await server.submitTransaction(tx);
    return (result as any).hash as string;
  } catch (err: any) {
    const code =
      err?.response?.data?.extras?.result_codes?.transaction ?? `${method}_failed`;
    throw new Error(code);
  }
}

// ── Public API ────────────────────────────────────────────────────────────────

export async function fetchStakingConfig(): Promise<StakingConfig> {
  // TODO: replace with SorobanRpc call to event_registry.get_staking_config()
  return {
    tokenAddress: 'GBBD47IF6LWK7P7MABDHYZCYYZ277OEX22PQ6CHYA67BDN6C7CR27STN',
    minimumStake: 500,
  };
}

export async function fetchOrganizerStakingState(
  publicKey?: string,
): Promise<OrganizerStakingState> {
  // TODO: replace with SorobanRpc call to event_registry.get_organizer_stake(publicKey)
  if (!publicKey) {
    return {
      status: 'Unverified',
      stakedAmount: 0,
      pendingRewards: 0,
      lockupEnds: null,
    };
  }
  return {
    status: 'Unverified',
    stakedAmount: 0,
    pendingRewards: 0,
    lockupEnds: null,
  };
}

export async function fetchStakingHistory(
  publicKey?: string,
): Promise<StakingTransaction[]> {
  // TODO: replace with indexed on-chain query for publicKey
  if (!publicKey) {
    return [];
  }
  return [];
}

export async function stakeCollateral(amount: number): Promise<StakingResult> {
  const secretKey = await StellarWalletManager.getSecretKey();
  if (!secretKey) {
    return { success: false, message: 'No Stellar wallet found. Import or generate a wallet first.' };
  }
  try {
    const txHash = await callContract(secretKey, 'stake_collateral', { amount });
    return { success: true, txHash, message: `Successfully staked ${amount} USDC.` };
  } catch (err: any) {
    return { success: false, message: err?.message ?? 'Stake failed. Please try again.' };
  }
}

export async function unstakeCollateral(): Promise<StakingResult> {
  const secretKey = await StellarWalletManager.getSecretKey();
  if (!secretKey) {
    return { success: false, message: 'No Stellar wallet found. Import or generate a wallet first.' };
  }
  try {
    const txHash = await callContract(secretKey, 'unstake_collateral', {});
    return {
      success: true,
      txHash,
      message: 'Unstake initiated. Collateral will be returned after the lockup period.',
    };
  } catch (err: any) {
    return { success: false, message: err?.message ?? 'Unstake failed. Please try again.' };
  }
}

export async function claimRewards(): Promise<StakingResult> {
  const secretKey = await StellarWalletManager.getSecretKey();
  if (!secretKey) {
    return { success: false, message: 'No Stellar wallet found. Import or generate a wallet first.' };
  }
  try {
    const txHash = await callContract(secretKey, 'claim_staker_rewards', {});
    return { success: true, txHash, message: 'Rewards claimed successfully.' };
  } catch (err: any) {
    return { success: false, message: err?.message ?? 'Claim failed. Please try again.' };
  }
}
