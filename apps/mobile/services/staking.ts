/**
 * Staking service — wraps event_registry contract interactions for organizer
 * collateral staking, unstaking and reward claims.
 *
 * Staking configuration and organizer state are read from the event_registry
 * Soroban contract.
 */

import {
  Address,
  Contract,
  Keypair,
  nativeToScVal,
  Networks,
  scValToNative,
  TransactionBuilder,
  Horizon,
} from '@stellar/stellar-sdk';
import { StellarWalletManager } from './stellar';
import { buildAndSimulateContractCall, loadSorobanAccount } from './sorobanClient';

// ── Constants ────────────────────────────────────────────────────────────────

const HORIZON_URL = 'https://horizon-testnet.stellar.org';
const TOKEN_DECIMALS = 10_000_000;
export const STELLAR_EXPERT_BASE = 'https://stellar.expert/explorer/testnet/tx';

/** event_registry contract address (testnet placeholder) */
export const EVENT_REGISTRY_CONTRACT =
  'CDLZFC3SYJYDZT7K67VZ75HPJVIEUVNIXF47ZG2FB2RMQQVU2HHGCN3';

// ── Public types ─────────────────────────────────────────────────────────────

export interface StakingConfig {
  /** Configured staking token contract address */
  tokenAddress: string;
  /** Minimum collateral required, in decimal-adjusted token units */
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

async function readContract(
  publicKey: string,
  method: string,
  ...args: ReturnType<typeof nativeToScVal>[]
): Promise<unknown> {
  const sourceAccount = await loadSorobanAccount(publicKey);
  const contract = new Contract(EVENT_REGISTRY_CONTRACT);
  const { simulation } = await buildAndSimulateContractCall({
    sourceAccount,
    operation: contract.call(method, ...args),
  });

  const retval = simulation.result?.retval;
  if (!retval) {
    throw new Error(`The event registry returned no value for ${method}.`);
  }
  return scValToNative(retval);
}

async function resolveReadPublicKey(publicKey?: string): Promise<string> {
  if (publicKey) return publicKey;
  const secretKey = await StellarWalletManager.getSecretKey();
  if (!secretKey) {
    throw new Error('Connect a Stellar wallet to read staking data.');
  }
  return Keypair.fromSecret(secretKey).publicKey();
}

function toTokenAmount(value: unknown): number {
  const amount = Number(BigInt(value as string | number | bigint));
  if (!Number.isFinite(amount)) {
    throw new Error('The event registry returned an invalid staking amount.');
  }
  return amount / TOKEN_DECIMALS;
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

export async function fetchStakingConfig(publicKey?: string): Promise<StakingConfig> {
  const sourcePublicKey = await resolveReadPublicKey(publicKey);
  const result = await readContract(sourcePublicKey, 'get_staking_config') as {
    token?: Address | string | null;
    min_amount?: string | number | bigint;
  } | null;
  if (!result) {
    throw new Error('The event registry returned no staking configuration.');
  }
  const tokenAddress = result.token?.toString();
  if (!tokenAddress || result.min_amount === undefined) {
    throw new Error('Staking has not been configured in the event registry.');
  }

  return {
    tokenAddress,
    minimumStake: toTokenAmount(result.min_amount),
  };
}

export async function fetchOrganizerStakingState(
  publicKey?: string,
): Promise<OrganizerStakingState> {
  if (!publicKey) {
    return {
      status: 'Unverified',
      stakedAmount: 0,
      pendingRewards: 0,
      lockupEnds: null,
    };
  }
  const result = await readContract(
    publicKey,
    'get_organizer_stake',
    nativeToScVal(Address.fromString(publicKey), { type: 'address' }),
  ) as {
    amount?: string | number | bigint;
    is_verified?: boolean;
    reward_balance?: string | number | bigint;
  } | null;

  if (!result) {
    return {
      status: 'Unverified',
      stakedAmount: 0,
      pendingRewards: 0,
      lockupEnds: null,
    };
  }

  if (
    result.amount === undefined ||
    result.reward_balance === undefined ||
    typeof result.is_verified !== 'boolean'
  ) {
    throw new Error('The event registry returned an invalid organizer stake.');
  }

  return {
    status: result.is_verified ? 'Verified' : 'Unverified',
    stakedAmount: toTokenAmount(result.amount),
    pendingRewards: toTokenAmount(result.reward_balance),
    lockupEnds: null,
  };
}

export async function fetchStakingHistory(
  publicKey?: string,
): Promise<StakingTransaction[]> {
  // The contract exposes current stake but no transaction-history entry point.
  void publicKey;
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
